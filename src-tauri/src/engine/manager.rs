//! Engine Manager：进程生命周期 + UCI 编解码 + 请求串行化 + 崩溃恢复。
//!
//! - React 不直接管理 Pikafish；所有引擎交互都经过本管理器。
//! - 单任务事件循环：stdout 逐行异步解析，命令经 mpsc 串行化，避免并发写 stdin。
//! - 处理：启动失败、崩溃（stdout EOF）、停止超时、restart、quit、分析期间切换局面。

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::timeout;

use super::types::{EngineConfig, EngineEvent, EngineStatus, GoParams, UciOption};
use super::uci;

/// 停止搜索后待执行的挂起动作（分析期间切换局面时排队）。
enum Pending {
    PositionOnly {
        fen: Option<String>,
        moves: Vec<String>,
    },
    PositionAndGo {
        fen: Option<String>,
        moves: Vec<String>,
        params: GoParams,
    },
}

enum Cmd {
    SetOption {
        name: String,
        value: Option<String>,
    },
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },
    Go(GoParams),
    SetPositionAndGo {
        fen: Option<String>,
        moves: Vec<String>,
        params: GoParams,
    },
    Stop(oneshot::Sender<()>),
    IsReady(oneshot::Sender<Result<(), String>>),
    Quit(oneshot::Sender<()>),
    /// 定时器到期（token 用于丢弃过期定时器）。
    Tick(u64),
}

const STOP_TIMEOUT: Duration = Duration::from_secs(3);
const READY_TIMEOUT: Duration = Duration::from_secs(5);

/// 当前进程相关状态（restart 时整体替换）。
struct ProcessState {
    cmd_tx: mpsc::Sender<Cmd>,
    task: tokio::task::JoinHandle<()>,
    pid: u32,
}

/// 引擎管理器（单文档会话；命令层持有一个全局实例）。
pub struct EngineManager {
    state: tokio::sync::Mutex<Option<ProcessState>>,
    events: broadcast::Sender<EngineEvent>,
    status: Arc<Mutex<EngineStatus>>,
    options: Arc<Mutex<Vec<UciOption>>>,
    engine_id: Arc<Mutex<Option<String>>>,
    config: EngineConfig,
}

impl EngineManager {
    /// 构造并启动引擎进程（完成 uci/isready 握手）。
    pub async fn spawn(config: EngineConfig) -> Result<Self, String> {
        let (events, _) = broadcast::channel::<EngineEvent>(256);
        let manager = EngineManager {
            state: tokio::sync::Mutex::new(None),
            events,
            status: Arc::new(Mutex::new(EngineStatus::Stopped)),
            options: Arc::new(Mutex::new(Vec::new())),
            engine_id: Arc::new(Mutex::new(None)),
            config,
        };
        manager.start().await?;
        Ok(manager)
    }

    /// （重新）启动引擎进程。
    pub async fn start(&self) -> Result<(), String> {
        let (child, stdin, reader, engine_id, options) = spawn_process(&self.config).await?;
        let pid = child
            .id()
            .ok_or_else(|| "无法获取引擎进程 PID".to_string())?;
        let (cmd_tx, cmd_rx) = mpsc::channel::<Cmd>(32);
        let events = self.events.clone();
        let status = self.status.clone();
        let options_arc = self.options.clone();
        let task = tokio::spawn(run_process(
            child,
            stdin,
            reader,
            cmd_tx.clone(),
            cmd_rx,
            events.clone(),
            status.clone(),
            options_arc,
        ));
        *self
            .options
            .lock()
            .map_err(|_| "选项锁已损坏".to_string())? = options;
        *self
            .engine_id
            .lock()
            .map_err(|_| "引擎标识锁已损坏".to_string())? = Some(engine_id);
        *self.status.lock().map_err(|_| "状态锁已损坏".to_string())? = EngineStatus::Ready;
        *self.state.lock().await = Some(ProcessState { cmd_tx, task, pid });
        let _ = self.events.send(EngineEvent::Started);
        Ok(())
    }

    /// 停止并回收当前进程（restart/quit 的内部实现）。
    async fn stop_process(&self) {
        let state = self.state.lock().await.take();
        if let Some(state) = state {
            let (ack, rx) = oneshot::channel();
            let _ = state.cmd_tx.send(Cmd::Quit(ack)).await;
            let _ = timeout(Duration::from_secs(5), rx).await;
            let _ = timeout(Duration::from_secs(2), state.task).await;
        }
        *self.status.lock().unwrap_or_else(|e| e.into_inner()) = EngineStatus::Stopped;
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EngineEvent> {
        self.events.subscribe()
    }

    pub fn status(&self) -> EngineStatus {
        self.status
            .lock()
            .map(|g| *g)
            .unwrap_or(EngineStatus::Crashed)
    }

    pub fn options(&self) -> Vec<UciOption> {
        self.options.lock().map(|o| o.clone()).unwrap_or_default()
    }

    pub fn engine_id(&self) -> Option<String> {
        self.engine_id.lock().ok().and_then(|i| i.clone())
    }

    pub async fn set_option(&self, name: &str, value: Option<&str>) -> Result<(), String> {
        self.send(Cmd::SetOption {
            name: name.to_string(),
            value: value.map(|s| s.to_string()),
        })
        .await
    }

    pub async fn set_position(&self, fen: Option<&str>, moves: &[String]) -> Result<(), String> {
        self.send(Cmd::Position {
            fen: fen.map(|s| s.to_string()),
            moves: moves.to_vec(),
        })
        .await
    }

    pub async fn go(&self, params: GoParams) -> Result<(), String> {
        self.send(Cmd::Go(params)).await
    }

    /// 分析期间切换局面：自动先 stop（若在搜索），再 position + go。
    pub async fn set_position_and_go(
        &self,
        fen: Option<&str>,
        moves: &[String],
        params: GoParams,
    ) -> Result<(), String> {
        self.send(Cmd::SetPositionAndGo {
            fen: fen.map(|s| s.to_string()),
            moves: moves.to_vec(),
            params,
        })
        .await
    }

    pub async fn stop(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send(Cmd::Stop(tx)).await?;
        timeout(STOP_TIMEOUT + Duration::from_secs(2), rx)
            .await
            .map_err(|_| "等待引擎停止超时".to_string())?
            .map_err(|_| "停止确认通道已关闭".to_string())
    }

    pub async fn is_ready(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send(Cmd::IsReady(tx)).await?;
        timeout(READY_TIMEOUT + Duration::from_secs(2), rx)
            .await
            .map_err(|_| "等待 readyok 超时".to_string())?
            .map_err(|_| "isready 通道已关闭".to_string())?
    }

    /// 崩溃/无响应后重启：回收旧进程并重新拉起。
    pub async fn restart(&self) -> Result<(), String> {
        self.stop_process().await;
        self.start().await
    }

    /// 优雅退出并回收进程。
    pub async fn quit(&self) -> Result<(), String> {
        self.stop_process().await;
        Ok(())
    }

    async fn send(&self, cmd: Cmd) -> Result<(), String> {
        let state = self.state.lock().await;
        let Some(state) = state.as_ref() else {
            return Err("引擎未运行".to_string());
        };
        state
            .cmd_tx
            .send(cmd)
            .await
            .map_err(|_| "引擎已停止或崩溃".to_string())
    }
}

impl Drop for EngineManager {
    fn drop(&mut self) {
        // 应用退出兜底：强制结束引擎进程树（Windows）。
        #[cfg(target_os = "windows")]
        {
            let pid = match self.state.try_lock() {
                Ok(guard) => guard.as_ref().map(|s| s.pid),
                Err(_) => None,
            };
            if let Some(pid) = pid {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .status();
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = ();
    }
}

/// 启动进程并完成 uci/isready 握手。
async fn spawn_process(
    config: &EngineConfig,
) -> Result<
    (
        Child,
        ChildStdin,
        BufReader<ChildStdout>,
        String,
        Vec<UciOption>,
    ),
    String,
> {
    let mut cmd = Command::new(&config.program);
    cmd.args(&config.args).envs(&config.env);
    if let Some(dir) = &config.cwd {
        cmd.current_dir(dir);
    }
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());
    // 任务结束/句柄丢弃时自动终止进程（兜底）
    cmd.kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|e| format!("启动引擎失败：{e}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "引擎 stdin 不可用".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "引擎 stdout 不可用".to_string())?;
    let mut reader = BufReader::new(stdout);

    let (id, options) = handshake(&mut stdin, &mut reader, config.handshake_timeout).await?;
    Ok((child, stdin, reader, id, options))
}

/// uci → uciok（收集 id/option），isready → readyok。任何超时/EOF 视为启动失败。
async fn handshake(
    stdin: &mut ChildStdin,
    reader: &mut BufReader<ChildStdout>,
    timeout_dur: Duration,
) -> Result<(String, Vec<UciOption>), String> {
    write_line(stdin, uci::uci()).await;
    let mut id = String::new();
    let mut options = Vec::new();
    let mut buf = String::new();
    loop {
        buf.clear();
        match timeout(timeout_dur, reader.read_line(&mut buf)).await {
            Ok(Ok(0)) => return Err("引擎在握手期间退出".to_string()),
            Ok(Ok(_)) => {
                let line = buf.trim();
                if let Some(v) = line.strip_prefix("id name ") {
                    id = v.to_string();
                } else if let Some(o) = uci::parse_option(line) {
                    options.push(o);
                } else if line == "uciok" {
                    break;
                }
            }
            Ok(Err(e)) => return Err(format!("读取引擎输出失败：{e}")),
            Err(_) => return Err("引擎握手超时（未收到 uciok）".to_string()),
        }
    }

    write_line(stdin, uci::isready()).await;
    loop {
        buf.clear();
        match timeout(timeout_dur, reader.read_line(&mut buf)).await {
            Ok(Ok(0)) => return Err("引擎在 isready 期间退出".to_string()),
            Ok(Ok(_)) => {
                if buf.trim() == "readyok" {
                    break;
                }
            }
            Ok(Err(e)) => return Err(format!("读取引擎输出失败：{e}")),
            Err(_) => return Err("引擎握手超时（未收到 readyok）".to_string()),
        }
    }
    Ok((id, options))
}

async fn write_line(stdin: &mut ChildStdin, line: &str) {
    let _ = stdin.write_all(format!("{line}\n").as_bytes()).await;
    let _ = stdin.flush().await;
}

fn schedule_timer(cmd_tx: mpsc::Sender<Cmd>, dur: Duration, token: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(dur).await;
        let _ = cmd_tx.send(Cmd::Tick(token)).await;
    });
}

#[allow(clippy::too_many_arguments)]
/// 单个引擎进程的事件循环（退出/崩溃时返回，由外层决定 restart）。
async fn run_process(
    mut child: Child,
    mut stdin: ChildStdin,
    mut reader: BufReader<ChildStdout>,
    cmd_tx: mpsc::Sender<Cmd>,
    mut cmd_rx: mpsc::Receiver<Cmd>,
    events: broadcast::Sender<EngineEvent>,
    status: Arc<Mutex<EngineStatus>>,
    _options: Arc<Mutex<Vec<UciOption>>>,
) {
    let mut line = String::new();
    let mut searching = false;
    let mut stopping = false;
    let mut pending: Option<Pending> = None;
    let mut stop_ack: Option<oneshot::Sender<()>> = None;
    let mut ready_ack: Option<oneshot::Sender<Result<(), String>>> = None;
    let mut wait_token: u64 = 0;
    let mut stop_deadline: Option<u64> = None;
    let mut ready_deadline: Option<u64> = None;
    let mut go_guard: Option<u64> = None;

    macro_rules! set_status {
        ($s:expr) => {
            *status.lock().unwrap_or_else(|e| e.into_inner()) = $s;
        };
    }

    loop {
        line.clear();
        tokio::select! {
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else { break };
                match cmd {
                    Cmd::SetOption { name, value } => {
                        write_line(&mut stdin, &uci::setoption(&name, value.as_deref())).await;
                        let _ = events.send(EngineEvent::OptionSet { name, value });
                    }
                    Cmd::Position { fen, moves } => {
                        if searching {
                            write_line(&mut stdin, uci::stop()).await;
                            searching = false;
                            stopping = true;
                            let token = next_token(&mut wait_token);
                            stop_deadline = Some(token);
                            pending = Some(Pending::PositionOnly { fen, moves });
                            schedule_timer(cmd_tx.clone(), STOP_TIMEOUT, token);
                        } else {
                            write_line(&mut stdin, &uci::position(fen.as_deref(), &moves)).await;
                        }
                    }
                    Cmd::Go(params) => {
                        if searching {
                            write_line(&mut stdin, uci::stop()).await;
                            searching = false;
                            stopping = true;
                            let token = next_token(&mut wait_token);
                            stop_deadline = Some(token);
                            pending = Some(Pending::PositionAndGo { fen: None, moves: Vec::new(), params });
                            schedule_timer(cmd_tx.clone(), STOP_TIMEOUT, token);
                        } else if stopping {
                            pending = Some(Pending::PositionAndGo { fen: None, moves: Vec::new(), params });
                        } else {
                            start_go(&mut stdin, &params, &mut searching, &mut go_guard, &mut wait_token, cmd_tx.clone(), &events).await;
                            set_status!(EngineStatus::Searching);
                        }
                    }
                    Cmd::SetPositionAndGo { fen, moves, params } => {
                        if searching {
                            write_line(&mut stdin, uci::stop()).await;
                            searching = false;
                            stopping = true;
                            let token = next_token(&mut wait_token);
                            stop_deadline = Some(token);
                            pending = Some(Pending::PositionAndGo { fen, moves, params });
                            schedule_timer(cmd_tx.clone(), STOP_TIMEOUT, token);
                        } else if stopping {
                            pending = Some(Pending::PositionAndGo { fen, moves, params });
                        } else {
                            write_line(&mut stdin, &uci::position(fen.as_deref(), &moves)).await;
                            start_go(&mut stdin, &params, &mut searching, &mut go_guard, &mut wait_token, cmd_tx.clone(), &events).await;
                            set_status!(EngineStatus::Searching);
                        }
                    }
                    Cmd::Stop(ack) => {
                        if searching {
                            write_line(&mut stdin, uci::stop()).await;
                            searching = false;
                            stopping = true;
                            let token = next_token(&mut wait_token);
                            stop_deadline = Some(token);
                            schedule_timer(cmd_tx.clone(), STOP_TIMEOUT, token);
                            stop_ack = Some(ack);
                        } else if stopping {
                            stop_ack = Some(ack);
                        } else {
                            let _ = ack.send(());
                            let _ = events.send(EngineEvent::Stopped);
                        }
                    }
                    Cmd::IsReady(ack) => {
                        write_line(&mut stdin, uci::isready()).await;
                        ready_ack = Some(ack);
                        let token = next_token(&mut wait_token);
                        ready_deadline = Some(token);
                        schedule_timer(cmd_tx.clone(), READY_TIMEOUT, token);
                    }
                    Cmd::Quit(ack) => {
                        write_line(&mut stdin, uci::quit()).await;
                        drop(stdin);
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        set_status!(EngineStatus::Stopped);
                        let _ = ack.send(());
                        break;
                    }
                    Cmd::Tick(token) => {
                        if stop_deadline == Some(token) {
                            stop_deadline = None;
                            if let Some(ack) = stop_ack.take() {
                                let _ = ack.send(());
                            }
                            finish_stop(
                                &mut stopping, &mut pending, &mut stdin,
                                &mut searching, &mut go_guard, &mut wait_token,
                                cmd_tx.clone(), &events, &status,
                            ).await;
                        }
                        if ready_deadline == Some(token) {
                            ready_deadline = None;
                            if let Some(ack) = ready_ack.take() {
                                let _ = ack.send(Err("isready 超时".to_string()));
                            }
                        }
                        if go_guard == Some(token) {
                            go_guard = None;
                            if searching && !stopping {
                                write_line(&mut stdin, uci::stop()).await;
                                searching = false;
                                stopping = true;
                                let t = next_token(&mut wait_token);
                                stop_deadline = Some(t);
                                schedule_timer(cmd_tx.clone(), STOP_TIMEOUT, t);
                            }
                        }
                    }
                }
            }
            read = reader.read_line(&mut line) => {
                match read {
                    Ok(0) => {
                        // stdout EOF：进程退出
                        let code = child.try_wait().ok().flatten().and_then(|s| s.code());
                        if let Some(ack) = stop_ack.take() { let _ = ack.send(()); }
                        if let Some(ack) = ready_ack.take() { let _ = ack.send(Err("引擎已退出".to_string())); }
                        set_status!(EngineStatus::Crashed);
                        let _ = events.send(EngineEvent::Crashed { code });
                        break;
                    }
                    Ok(_) => {
                        let text = line.trim().to_string();
                        if text.starts_with("bestmove") {
                            if let Some(bm) = uci::parse_bestmove(&text) {
                                let _ = events.send(EngineEvent::BestMove(bm));
                            }
                            if stopping {
                                finish_stop(
                                    &mut stopping, &mut pending, &mut stdin,
                                    &mut searching, &mut go_guard, &mut wait_token,
                                    cmd_tx.clone(), &events, &status,
                                ).await;
                            } else if searching {
                                searching = false;
                                go_guard = None;
                                set_status!(EngineStatus::Ready);
                                let _ = events.send(EngineEvent::Stopped);
                            }
                        } else if text == "readyok" {
                            ready_deadline = None;
                            if let Some(ack) = ready_ack.take() { let _ = ack.send(Ok(())); }
                            let _ = events.send(EngineEvent::Ready);
                        } else if let Some(msg) = text.strip_prefix("info string ") {
                            let _ = events.send(EngineEvent::InfoString(msg.to_string()));
                        } else if text.starts_with("info ") {
                            if let Some(info) = uci::parse_info(&text) {
                                let _ = events.send(EngineEvent::Info(info));
                            }
                        }
                    }
                    Err(e) => {
                        set_status!(EngineStatus::Crashed);
                        let _ = events.send(EngineEvent::Error(format!("读取引擎输出失败：{e}")));
                        break;
                    }
                }
            }
        }
    }
}

fn next_token(wait_token: &mut u64) -> u64 {
    *wait_token += 1;
    *wait_token
}

async fn start_go(
    stdin: &mut ChildStdin,
    params: &GoParams,
    searching: &mut bool,
    go_guard: &mut Option<u64>,
    wait_token: &mut u64,
    cmd_tx: mpsc::Sender<Cmd>,
    events: &broadcast::Sender<EngineEvent>,
) {
    write_line(stdin, &uci::go(params)).await;
    *searching = true;
    let _ = events.send(EngineEvent::Searching);
    if let Some(ms) = params.movetime_ms {
        let token = next_token(wait_token);
        *go_guard = Some(token);
        schedule_timer(cmd_tx, Duration::from_millis(ms + 150), token);
    }
}

#[allow(clippy::too_many_arguments)]
async fn finish_stop(
    stopping: &mut bool,
    pending: &mut Option<Pending>,
    stdin: &mut ChildStdin,
    searching: &mut bool,
    go_guard: &mut Option<u64>,
    wait_token: &mut u64,
    cmd_tx: mpsc::Sender<Cmd>,
    events: &broadcast::Sender<EngineEvent>,
    status: &Arc<Mutex<EngineStatus>>,
) {
    *stopping = false;
    *go_guard = None;
    let _ = events.send(EngineEvent::Stopped);
    match pending.take() {
        Some(Pending::PositionOnly { fen, moves }) => {
            write_line(stdin, &uci::position(fen.as_deref(), &moves)).await;
            *status.lock().unwrap_or_else(|e| e.into_inner()) = EngineStatus::Ready;
        }
        Some(Pending::PositionAndGo { fen, moves, params }) => {
            write_line(stdin, &uci::position(fen.as_deref(), &moves)).await;
            start_go(
                stdin, &params, searching, go_guard, wait_token, cmd_tx, events,
            )
            .await;
            *status.lock().unwrap_or_else(|e| e.into_inner()) = EngineStatus::Searching;
        }
        None => {
            *status.lock().unwrap_or_else(|e| e.into_inner()) = EngineStatus::Ready;
        }
    }
}
