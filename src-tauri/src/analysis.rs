//! 自动复盘（Automatic Game Analysis）。
//!
//! 目标：分析完整棋谱主线，对每一步记录：实际着法 / 最佳着法 / 走前评价 / 走后评价 /
//! 评价损失 / 深度 / PV，并按「评价损失」分类（Best/Excellent/Good/Inaccuracy/Mistake/Blunder）。
//!
//! 设计要点：
//! - **分类阈值可配置**（`ClassificationConfig`，不硬编码）；评价损失 = 走子方视角的
//!   前后评价差（厘兵，非负）。
//! - 分数约定：引擎 `score cp` 视为**红方视角**（docs/engine.md §3.2）。
//!   `NEEDS_VERIFICATION`：真实 Pikafish 的分数视角（红方 vs 行棋方）需在冒烟测试中核实；
//!   若为行棋方视角，仅需调整 `score_cp_red` 的符号逻辑。
//! - 不阻塞 UI：整个分析循环在 tokio 任务中异步执行，命令层只读写状态；
//!   支持 停止（暂停）/ 继续 / 重新分析。
//! - 每局面一次有限深度搜索：局面 i 的搜索既是「第 i-1 步的走后评价」也是「第 i 步的走前评价」，
//!   因此 n 步棋只需 n+1 次搜索。

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::engine::{EngineEvent, EngineManager, GoParams, InfoLine, Score};

/// 着法评价分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl Category {
    pub fn name(self) -> &'static str {
        match self {
            Category::Best => "best",
            Category::Excellent => "excellent",
            Category::Good => "good",
            Category::Inaccuracy => "inaccuracy",
            Category::Mistake => "mistake",
            Category::Blunder => "blunder",
        }
    }
}

/// 分类阈值（厘兵，**可配置**；默认值为经验起点，不硬编码进分类逻辑）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationConfig {
    /// 与最佳着法差距 ≤ `best_cp` → Best
    pub best_cp: i32,
    pub excellent_cp: i32,
    pub good_cp: i32,
    pub inaccuracy_cp: i32,
    /// 超过 `mistake_cp` → Blunder
    pub mistake_cp: i32,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        ClassificationConfig {
            best_cp: 10,
            excellent_cp: 30,
            good_cp: 60,
            inaccuracy_cp: 120,
            mistake_cp: 250,
        }
    }
}

impl ClassificationConfig {
    /// 按损失（厘兵）分类。损失应为非负。
    pub fn classify(&self, loss_cp: i32) -> Category {
        let loss = loss_cp.max(0);
        if loss <= self.best_cp {
            Category::Best
        } else if loss <= self.excellent_cp {
            Category::Excellent
        } else if loss <= self.good_cp {
            Category::Good
        } else if loss <= self.inaccuracy_cp {
            Category::Inaccuracy
        } else if loss <= self.mistake_cp {
            Category::Mistake
        } else {
            Category::Blunder
        }
    }
}

/// 自动复盘配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisConfig {
    /// 每局面搜索深度（与 `movetime_ms` 二选一；均 None 时报错）。
    pub depth: Option<u32>,
    /// 每局面搜索时间（毫秒）。
    pub movetime_ms: Option<u64>,
    /// 单个局面等待超时（毫秒）。
    pub per_move_timeout_ms: u64,
    /// 分类阈值。
    pub classification: ClassificationConfig,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        AnalysisConfig {
            depth: Some(12),
            movetime_ms: None,
            per_move_timeout_ms: 15_000,
            classification: ClassificationConfig::default(),
        }
    }
}

impl AnalysisConfig {
    fn go_params(&self) -> GoParams {
        GoParams {
            infinite: false,
            depth: self.depth,
            movetime_ms: self.movetime_ms,
            ..GoParams::default()
        }
    }
}

/// 计划分析的一步（棋谱主线节点）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedMove {
    pub node_id: u64,
    /// 走到该节点的着法（UCI）。
    pub mv: String,
    /// 该着法是否红方所走（决定损失视角）。
    pub is_red: bool,
}

/// 单步评估结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveAssessment {
    pub node_id: u64,
    /// 实际着法（UCI）。
    pub mv: String,
    /// 最佳着法（UCI）。
    pub best_move: String,
    /// 走前评价（红方视角，厘兵）。
    pub eval_before_cp: i32,
    /// 走后评价（红方视角，厘兵）。
    pub eval_after_cp: i32,
    /// 评价损失（走子方视角，厘兵，非负）。
    pub loss_cp: i32,
    pub depth: u32,
    pub pv: Vec<String>,
    pub category: Category,
}

/// 一次搜索的输出（评估下一步需要）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchOutcome {
    best_move: String,
    score_cp_red: i32,
    depth: u32,
    pv: Vec<String>,
}

/// 分析状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisStatus {
    Idle,
    Running,
    Paused,
    Done,
    Failed(String),
}

impl serde::Serialize for AnalysisStatus {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.name())
    }
}

impl AnalysisStatus {
    pub fn name(&self) -> &'static str {
        match self {
            AnalysisStatus::Idle => "idle",
            AnalysisStatus::Running => "running",
            AnalysisStatus::Paused => "paused",
            AnalysisStatus::Done => "done",
            AnalysisStatus::Failed(_) => "failed",
        }
    }
}

/// 分析进度事件（推送给前端）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AnalysisEvent {
    StatusChanged {
        status: AnalysisStatus,
    },
    Progress {
        done: usize,
        total: usize,
        current_node: Option<u64>,
    },
    Assessment(MoveAssessment),
    Finished(Vec<MoveAssessment>),
}

/// 分析器内部状态（std Mutex：访问都是短临界区，不跨 await 持锁）。
struct AnalyzerState {
    mgr: Option<Arc<EngineManager>>,
    status: AnalysisStatus,
    startpos: String,
    moves: Vec<PlannedMove>,
    assessments: Vec<MoveAssessment>,
    /// 已完成评估的步数（= 已分析位置数 - 1）。
    progress: usize,
    current_node: Option<u64>,
    /// 最近一个已分析局面的搜索结果（继续时需要）。
    last_outcome: Option<SearchOutcome>,
    config: AnalysisConfig,
}

/// 自动复盘器（单文档会话；命令层持有一个全局实例）。
pub struct AutoAnalyzer {
    state: Arc<Mutex<AnalyzerState>>,
    events: broadcast::Sender<AnalysisEvent>,
    /// 唤醒运行任务（暂停/恢复/开始）。
    notify: Arc<tokio::sync::Notify>,
    /// 唯一的运行任务（持久存活，暂停时等待 notify）。
    runner: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Default for AutoAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoAnalyzer {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel::<AnalysisEvent>(256);
        AutoAnalyzer {
            state: Arc::new(Mutex::new(AnalyzerState {
                mgr: None,
                status: AnalysisStatus::Idle,
                startpos: String::new(),
                moves: Vec::new(),
                assessments: Vec::new(),
                progress: 0,
                current_node: None,
                last_outcome: None,
                config: AnalysisConfig::default(),
            })),
            events,
            notify: Arc::new(tokio::sync::Notify::new()),
            runner: Mutex::new(None),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AnalysisEvent> {
        self.events.subscribe()
    }

    /// 只读快照（供命令层 DTO）。
    pub fn snapshot(&self) -> (AnalysisStatus, usize, usize, Vec<MoveAssessment>) {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        (
            s.status.clone(),
            s.progress,
            s.moves.len(),
            s.assessments.clone(),
        )
    }

    pub fn status(&self) -> AnalysisStatus {
        self.state
            .lock()
            .map(|s| s.status.clone())
            .unwrap_or(AnalysisStatus::Failed("状态锁损坏".to_string()))
    }

    /// 开始/重新分析：重置状态、记录引擎并唤醒运行任务。
    pub fn start(
        &self,
        mgr: Arc<EngineManager>,
        startpos: String,
        moves: Vec<PlannedMove>,
        config: AnalysisConfig,
    ) {
        {
            let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
            s.mgr = Some(mgr);
            s.startpos = startpos;
            s.moves = moves;
            s.config = config;
            s.assessments.clear();
            s.progress = 0;
            s.current_node = None;
            s.last_outcome = None;
            s.status = AnalysisStatus::Running;
        }
        let _ = self.events.send(AnalysisEvent::StatusChanged {
            status: AnalysisStatus::Running,
        });
        self.ensure_runner();
        self.notify.notify_one();
    }

    /// 停止（暂停）：完成当前局面搜索后运行任务会等待 notify。
    pub fn stop(&self) {
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.status == AnalysisStatus::Running {
            s.status = AnalysisStatus::Paused;
            let _ = self.events.send(AnalysisEvent::StatusChanged {
                status: AnalysisStatus::Paused,
            });
        }
    }

    /// 继续：从上次进度恢复（同一运行任务被唤醒）。
    pub fn resume(&self, mgr: Arc<EngineManager>) {
        {
            let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if s.status == AnalysisStatus::Done {
                return;
            }
            s.mgr = Some(mgr);
            s.status = AnalysisStatus::Running;
            let _ = self.events.send(AnalysisEvent::StatusChanged {
                status: AnalysisStatus::Running,
            });
        }
        self.ensure_runner();
        self.notify.notify_one();
    }

    /// 确保唯一的持久运行任务存在。
    fn ensure_runner(&self) {
        let mut guard = self.runner.lock().unwrap_or_else(|e| e.into_inner());
        let alive = guard.as_ref().map(|h| !h.is_finished()).unwrap_or(false);
        if alive {
            return;
        }
        let state = self.state.clone();
        let events = self.events.clone();
        let notify = self.notify.clone();
        *guard = Some(tokio::spawn(async move {
            runner_loop(state, events, notify).await;
        }));
    }
}

/// 分数 → 红方视角厘兵（Mate 转大分数）。
fn score_cp_red(score: Option<Score>) -> i32 {
    match score {
        None => 0,
        Some(Score::Cp(v)) => v,
        // 约定：`mate m`（m>0 红胜）→ 100000 - m；m<0 黑胜 → -(100000 + m)
        Some(Score::Mate(m)) => {
            if m > 0 {
                100_000 - m
            } else {
                -(100_000 + m)
            }
        }
    }
}

/// 持久运行任务：Running 时逐步分析，否则等待 notify。
async fn runner_loop(
    state: Arc<Mutex<AnalyzerState>>,
    events: broadcast::Sender<AnalysisEvent>,
    notify: Arc<tokio::sync::Notify>,
) {
    loop {
        // 取当前状态与进度（不跨 await 持锁）
        let (running, startpos, moves, config, start_i, prev, mgr) = {
            let s = state.lock().unwrap_or_else(|e| e.into_inner());
            let fresh = s.last_outcome.is_none() && s.progress == 0;
            (
                s.status == AnalysisStatus::Running,
                s.startpos.clone(),
                s.moves.clone(),
                s.config.clone(),
                if fresh { 0 } else { s.progress + 1 },
                s.last_outcome.clone(),
                s.mgr.clone(),
            )
        };
        if !running {
            notify.notified().await;
            continue;
        }
        let Some(mgr) = mgr else {
            notify.notified().await;
            continue;
        };
        let positions_total = moves.len() + 1;
        if start_i >= positions_total {
            // 完成
            let assessments = {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.status = AnalysisStatus::Done;
                s.assessments.clone()
            };
            let _ = events.send(AnalysisEvent::StatusChanged {
                status: AnalysisStatus::Done,
            });
            let _ = events.send(AnalysisEvent::Finished(assessments));
            notify.notified().await;
            continue;
        }

        let prefix: Vec<String> = moves.iter().take(start_i).map(|m| m.mv.clone()).collect();
        let outcome = analyze_one(&mgr, &startpos, &prefix, &config).await;

        match outcome {
            Ok(outcome) => {
                if start_i >= 1 {
                    // 用 位置 start_i-1（prev）与 位置 start_i（outcome）构建第 start_i-1 步评估
                    if let Some(before) = prev.as_ref() {
                        let plan = &moves[start_i - 1];
                        let assessment = build_assessment(plan, before, &outcome, &config);
                        {
                            let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                            s.assessments.push(assessment.clone());
                            s.progress = start_i;
                            s.current_node = Some(plan.node_id);
                            s.last_outcome = Some(outcome.clone());
                        }
                        let _ = events.send(AnalysisEvent::Assessment(assessment));
                        let _ = events.send(AnalysisEvent::Progress {
                            done: start_i,
                            total: moves.len(),
                            current_node: Some(plan.node_id),
                        });
                    }
                } else {
                    {
                        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                        s.last_outcome = Some(outcome.clone());
                    }
                }
            }
            Err(e) => {
                {
                    let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                    s.status = AnalysisStatus::Failed(e.clone());
                }
                let _ = events.send(AnalysisEvent::StatusChanged {
                    status: AnalysisStatus::Failed(e),
                });
                notify.notified().await;
                continue;
            }
        }
    }
}

/// 对单个局面做一次有限搜索，等待 bestmove，返回最佳着法 + 最高深度 info。
async fn analyze_one(
    mgr: &EngineManager,
    startpos: &str,
    moves: &[String],
    config: &AnalysisConfig,
) -> Result<SearchOutcome, String> {
    let mut rx = mgr.subscribe();
    let params = config.go_params();
    mgr.set_position_and_go(Some(startpos), moves, params)
        .await?;

    let timeout = Duration::from_millis(config.per_move_timeout_ms.max(1000));
    let mut got_searching = false;
    let mut best_info: Option<InfoLine> = None;

    let best_move = loop {
        let recv = tokio::time::timeout(timeout, rx.recv()).await;
        match recv {
            Ok(Ok(EngineEvent::Searching)) => {
                got_searching = true;
            }
            Ok(Ok(EngineEvent::Info(info))) if got_searching && info.multipv == 1 => {
                // 保留深度最大的 info（分数/PV 随深度稳定）
                let replace = match &best_info {
                    None => true,
                    Some(prev) => info.depth.unwrap_or(0) > prev.depth.unwrap_or(0),
                };
                if replace {
                    best_info = Some(info);
                }
            }
            Ok(Ok(EngineEvent::BestMove(bm))) if got_searching => break bm.mv,
            Ok(Ok(EngineEvent::Error(e))) => return Err(e),
            Ok(Ok(_)) => {}
            Ok(Err(_)) => return Err("引擎事件通道已关闭".to_string()),
            Err(_) => return Err("等待引擎搜索结果超时".to_string()),
        }
    };

    let info = best_info.unwrap_or_default();
    Ok(SearchOutcome {
        best_move,
        score_cp_red: score_cp_red(info.score),
        depth: info.depth.unwrap_or(0),
        pv: info.pv.clone(),
    })
}

/// 构建第 `plan` 步的评估。
fn build_assessment(
    plan: &PlannedMove,
    before: &SearchOutcome,
    after: &SearchOutcome,
    config: &AnalysisConfig,
) -> MoveAssessment {
    // 损失 = 走子方视角：红方 loss = before - after；黑方 loss = after - before
    let loss = if plan.is_red {
        before.score_cp_red - after.score_cp_red
    } else {
        after.score_cp_red - before.score_cp_red
    }
    .max(0);
    let category = config.classification.classify(loss);
    MoveAssessment {
        node_id: plan.node_id,
        mv: plan.mv.clone(),
        best_move: before.best_move.clone(),
        eval_before_cp: before.score_cp_red,
        eval_after_cp: after.score_cp_red,
        loss_cp: loss,
        depth: before.depth,
        pv: before.pv.clone(),
        category,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(
        best: i32,
        excellent: i32,
        good: i32,
        inaccuracy: i32,
        mistake: i32,
    ) -> ClassificationConfig {
        ClassificationConfig {
            best_cp: best,
            excellent_cp: excellent,
            good_cp: good,
            inaccuracy_cp: inaccuracy,
            mistake_cp: mistake,
        }
    }

    #[test]
    fn classification_thresholds_are_configurable() {
        // 默认阈值
        let d = ClassificationConfig::default();
        assert_eq!(d.classify(0), Category::Best);
        assert_eq!(d.classify(10), Category::Best);
        assert_eq!(d.classify(15), Category::Excellent);
        assert_eq!(d.classify(40), Category::Good);
        assert_eq!(d.classify(90), Category::Inaccuracy);
        assert_eq!(d.classify(180), Category::Mistake);
        assert_eq!(d.classify(400), Category::Blunder);
        // 宽松阈值（大棋力差距场景）→ 同样损失归入更优类别
        let loose = cfg(50, 100, 200, 400, 800);
        assert_eq!(loose.classify(40), Category::Best);
        assert_eq!(loose.classify(90), Category::Excellent);
        assert_eq!(loose.classify(150), Category::Good);
        assert_eq!(loose.classify(300), Category::Inaccuracy);
        assert_eq!(loose.classify(600), Category::Mistake);
        // 严格阈值
        let strict = cfg(5, 10, 20, 40, 80);
        assert_eq!(strict.classify(6), Category::Excellent);
        assert_eq!(strict.classify(100), Category::Blunder);
        // 负损失视为 0（Best）
        assert_eq!(d.classify(-5), Category::Best);
    }

    #[test]
    fn loss_perspective_and_category() {
        // 红方走：before=50, after=-30 → 红方损失 80 → Inaccuracy（默认）
        let red = PlannedMove {
            node_id: 1,
            mv: "h2e2".into(),
            is_red: true,
        };
        let before = SearchOutcome {
            best_move: "b0c2".into(),
            score_cp_red: 50,
            depth: 12,
            pv: vec!["b0c2".into()],
        };
        let after = SearchOutcome {
            best_move: "h7e7".into(),
            score_cp_red: -30,
            depth: 12,
            pv: vec!["h7e7".into()],
        };
        let a = build_assessment(&red, &before, &after, &AnalysisConfig::default());
        assert_eq!(a.loss_cp, 80);
        assert_eq!(a.category, Category::Inaccuracy);
        assert_eq!(a.eval_before_cp, 50);
        assert_eq!(a.eval_after_cp, -30);
        assert_eq!(a.best_move, "b0c2");
        assert_eq!(a.mv, "h2e2");

        // 黑方走：红方视角 before=50 → after=-30，红方亏 80 = 黑方赚 80 → 黑方损失 0
        let black = PlannedMove {
            node_id: 2,
            mv: "h7e7".into(),
            is_red: false,
        };
        let b = build_assessment(&black, &before, &after, &AnalysisConfig::default());
        assert_eq!(b.loss_cp, 0);
        assert_eq!(b.category, Category::Best);
        // 红方视角下黑方收益为正（after > before）→ 黑方损失为正
        let before2 = SearchOutcome {
            best_move: "x".into(),
            score_cp_red: -20,
            depth: 10,
            pv: vec![],
        };
        let after2 = SearchOutcome {
            best_move: "y".into(),
            score_cp_red: 60,
            depth: 10,
            pv: vec![],
        };
        let b2 = build_assessment(&black, &before2, &after2, &AnalysisConfig::default());
        assert_eq!(b2.loss_cp, 80);
        assert_eq!(b2.category, Category::Inaccuracy);
        // 黑方赚分（红方视角下降）→ 黑方损失为 0
        let before3 = SearchOutcome {
            best_move: "x".into(),
            score_cp_red: 60,
            depth: 10,
            pv: vec![],
        };
        let after3 = SearchOutcome {
            best_move: "y".into(),
            score_cp_red: -20,
            depth: 10,
            pv: vec![],
        };
        let b3 = build_assessment(&black, &before3, &after3, &AnalysisConfig::default());
        assert_eq!(b3.loss_cp, 0);
        assert_eq!(b3.category, Category::Best);
    }

    #[test]
    fn score_mate_conversion() {
        assert_eq!(score_cp_red(Some(Score::Cp(35))), 35);
        assert_eq!(score_cp_red(Some(Score::Mate(3))), 100_000 - 3);
        assert_eq!(score_cp_red(Some(Score::Mate(-2))), -(100_000 - 2)); // = -99998
        assert_eq!(score_cp_red(None), 0);
    }
}
