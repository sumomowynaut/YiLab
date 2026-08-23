import { useEffect, useMemo, useState } from "react";
import { useShortcuts } from "./hooks/useShortcuts";
import { Board } from "./components/board/Board";
import { MoveTree } from "./components/board/MoveTree";
import { PiecePalette } from "./components/board/PiecePalette";
import { Button } from "./components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { AnalysisPanel } from "./components/engine/AnalysisPanel";
import { EvalCurve } from "./components/engine/EvalCurve";
import { AnalysisReport } from "./components/analysis/AnalysisReport";
import { GifExportPanel } from "./components/gif/GifExportPanel";
import { GameCodec } from "./components/io/GameCodec";
import { OcrPanel } from "./components/ocr/OcrPanel";
import { BookPanel } from "./components/book/BookPanel";
import { SettingsPanel } from "./components/settings/SettingsPanel";
import { getDefaultBoardApi } from "./lib/board/api";
import { sideToColor } from "./lib/board/notation";
import { getDefaultEngineApi } from "./lib/engine/api";
import { getDefaultGameApi } from "./lib/game/api";
import { getDefaultIoApi } from "./lib/io/api";
import { getDefaultOcrApi } from "./lib/ocr/api";
import { getDefaultAnalysisApi } from "./lib/analysis/api";
import { getDefaultGifApi } from "./lib/gif/api";
import { getDefaultBookApi } from "./lib/book/api";
import { useEngineStore } from "./stores/useEngineStore";
import { useThemeStore } from "./stores/useThemeStore";
import { useCurveStore } from "./stores/useCurveStore";
import { useAnalysisStore } from "./stores/useAnalysisStore";
import { selectDisplayPosition, useGameStore } from "./stores/useGameStore";
import type { VariationOption } from "./lib/gif/types";
import type { TreeNodeDto } from "./lib/game/types";

type TabKey = "game" | "analysis" | "book" | "io" | "settings";

const TABS: { key: TabKey; label: string }[] = [
  { key: "game", label: "棋谱" },
  { key: "analysis", label: "分析" },
  { key: "book", label: "开局库" },
  { key: "io", label: "导入导出" },
  { key: "settings", label: "设置" },
];

function App() {
  const snapshot = useGameStore((state) => state.snapshot);
  const position = useGameStore(selectDisplayPosition);
  const validation = useGameStore((state) => state.validation);
  const selected = useGameStore((state) => state.selected);
  const legalTargets = useGameStore((state) => state.legalTargets);
  const view = useGameStore((state) => state.view);
  const editing = useGameStore((state) => state.editing);
  const tool = useGameStore((state) => state.tool);
  const message = useGameStore((state) => state.message);
  const expandedVariations = useGameStore((state) => state.expandedVariations);
  const init = useGameStore((state) => state.init);
  const handleSquareClick = useGameStore((state) => state.handleSquareClick);
  const toggleEditing = useGameStore((state) => state.toggleEditing);
  const setTool = useGameStore((state) => state.setTool);
  const clearAll = useGameStore((state) => state.clearAll);
  const toggleSide = useGameStore((state) => state.toggleSide);
  const rotateView = useGameStore((state) => state.rotateView);
  const mirrorView = useGameStore((state) => state.mirrorView);
  const loadFen = useGameStore((state) => state.loadFen);
  const navigate = useGameStore((state) => state.navigate);
  const previous = useGameStore((state) => state.previous);
  const next = useGameStore((state) => state.next);
  const undo = useGameStore((state) => state.undo);
  const redo = useGameStore((state) => state.redo);
  const goToStart = useGameStore((state) => state.goToStart);
  const goToEnd = useGameStore((state) => state.goToEnd);
  const deleteVariation = useGameStore((state) => state.deleteVariation);
  const promoteVariation = useGameStore((state) => state.promoteVariation);
  const reorderVariation = useGameStore((state) => state.reorderVariation);
  const setComment = useGameStore((state) => state.setComment);
  const setNag = useGameStore((state) => state.setNag);
  const toggleVariation = useGameStore((state) => state.toggleVariation);
  const adoptSnapshot = useGameStore((state) => state.adoptSnapshot);
  const saveGame = useGameStore((state) => state.saveGame);
  const loadGame = useGameStore((state) => state.loadGame);

  // 收集变例节点（GIF「指定变例」来源）
  const variations = useMemo(() => {
    const out: VariationOption[] = [];
    function walk(node: TreeNodeDto): void {
      if (node.isVariation) {
        out.push({
          nodeId: node.id,
          label: `${node.moveNumber}${node.isRed ? "." : "…"} ${node.mv ?? ""}`,
        });
      }
      for (const child of node.children) {
        walk(child);
      }
    }
    if (snapshot) {
      walk(snapshot.tree);
    }
    return out;
  }, [snapshot]);
  const theme = useThemeStore((state) => state.theme);
  const initTheme = useThemeStore((state) => state.initTheme);
  const toggleTheme = useThemeStore((state) => state.toggleTheme);
  const curvePoints = useCurveStore((state) => state.points);
  const curveRecord = useCurveStore((state) => state.record);
  const curveClear = useCurveStore((state) => state.clear);
  const analysisStatus = useAnalysisStore((state) => state.status);
  const analysisProgress = useAnalysisStore((state) => state.progress);
  const analysisTotal = useAnalysisStore((state) => state.total);
  const analysisAssessments = useAnalysisStore((state) => state.assessments);
  const analysisInit = useAnalysisStore((state) => state.init);
  const analysisStart = useAnalysisStore((state) => state.start);
  const analysisStop = useAnalysisStore((state) => state.stop);
  const analysisContinue = useAnalysisStore((state) => state.continue);
  const engineStatus = useEngineStore((state) => state.status);
  const engineId = useEngineStore((state) => state.engineId);
  const engineLines = useEngineStore((state) => state.lines);
  const engineBestMove = useEngineStore((state) => state.bestMove);
  const engineMessage = useEngineStore((state) => state.message);
  const analysisEnabled = useEngineStore((state) => state.analysisEnabled);
  const preview = useEngineStore((state) => state.preview);
  const engineInit = useEngineStore((state) => state.init);
  const engineStart = useEngineStore((state) => state.start);
  const engineStop = useEngineStore((state) => state.stop);
  const engineRestart = useEngineStore((state) => state.restart);
  const engineStartAnalysis = useEngineStore((state) => state.startAnalysis);
  const enginePreviewPv = useEngineStore((state) => state.previewPv);
  const engineClearPreview = useEngineStore((state) => state.clearPreview);
  const [tab, setTab] = useState<TabKey>("game");
  const [fenInput, setFenInput] = useState("");
  const [commentDraft, setCommentDraft] = useState("");
  const [ioApi] = useState(() => getDefaultIoApi());
  const [ocrApi] = useState(() => getDefaultOcrApi());
  const [gifApi] = useState(() => getDefaultGifApi());
  const [bookApi] = useState(() => getDefaultBookApi());

  useEffect(() => {
    void init(getDefaultGameApi(), getDefaultBoardApi());
  }, [init]);

  // 主题初始化（localStorage / 系统偏好）
  useEffect(() => {
    initTheme();
  }, [initTheme]);

  // 自动复盘初始化（订阅事件）
  useEffect(() => {
    analysisInit(getDefaultAnalysisApi());
  }, [analysisInit]);

  // 引擎初始化（订阅事件）
  useEffect(() => {
    engineInit(getDefaultEngineApi(), getDefaultBoardApi());
  }, [engineInit]);

  // 分析开启时：切换棋步 → 对新局面发起分析（引擎内部 stop→position→go）
  const currentFen = snapshot?.currentFen ?? null;
  useEffect(() => {
    if (analysisEnabled && currentFen) {
      void engineStartAnalysis(currentFen);
    }
  }, [currentFen, analysisEnabled, engineStartAnalysis]);

  // 同步注释草稿
  useEffect(() => {
    setCommentDraft(snapshot?.comment ?? "");
  }, [snapshot?.comment]);

  // 评价曲线：分析开启时，把每个分析局面的主变（multipv=1）分数记录为红方视角
  useEffect(() => {
    if (!analysisEnabled || !currentFen || !position) {
      return;
    }
    const line = engineLines[1];
    const score = line?.score;
    if (!score || !("cp" in score)) {
      return;
    }
    const fromRed = position.sideToMove === "w" ? score.cp : -score.cp;
    curveRecord(currentFen, fromRed);
  }, [analysisEnabled, currentFen, position, engineLines, curveRecord]);

  // 全局快捷键：←/→ 导航、Home/End 首尾、F/M 翻转/镜像、Space 分析启停、Ctrl+Z/Y 悔棋/重做
  useShortcuts({
    onAction: (action) => {
      switch (action) {
        case "previous":
          void previous();
          break;
        case "next":
          void next();
          break;
        case "goToStart":
          void goToStart();
          break;
        case "goToEnd":
          void goToEnd();
          break;
        case "undo":
          void undo();
          break;
        case "redo":
          void redo();
          break;
        case "flip":
          rotateView();
          break;
        case "mirror":
          mirrorView();
          break;
        case "toggleAnalysis":
          if (analysisEnabled) {
            void engineStop();
          } else {
            void engineStart();
          }
          break;
      }
    },
    shouldIgnore: () => {
      const el = document.activeElement;
      return !!el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA");
    },
  });

  if (!snapshot || !position) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-background p-6 text-foreground">
        <p data-testid="loading">加载中…</p>
      </main>
    );
  }

  const sideLabel = sideToColor(position.sideToMove) === "red" ? "红方" : "黑方";

  return (
    <main className="flex min-h-screen items-start justify-center bg-background p-3 text-foreground sm:p-6">
      <Card className="w-full max-w-7xl">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>PikaXiangqi</CardTitle>
            <CardDescription>中国象棋复盘与分析 — 本地优先 · 引擎分析 · 自动复盘</CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            data-testid="theme-toggle"
            onClick={toggleTheme}
            aria-label="切换深浅色主题"
          >
            {theme === "dark" ? "☀ 浅色" : "🌙 深色"}
          </Button>
        </CardHeader>

        <CardContent className="flex flex-col gap-5 xl:flex-row">
          {/* 左列：棋盘为视觉中心 */}
          <div className="flex flex-col gap-3 xl:w-[440px] xl:shrink-0">
            {preview && (
              <div className="flex items-center gap-2 rounded border border-blue-300 bg-blue-50 px-2 py-1 text-xs text-blue-700">
                <span>
                  预览变化：<span className="font-mono">{preview.moves.join(" ")}</span>
                </span>
                <Button variant="outline" size="sm" onClick={engineClearPreview}>
                  退出预览
                </Button>
              </div>
            )}
            <Board
              position={preview?.position ?? position}
              selected={selected}
              legalTargets={legalTargets}
              view={view}
              onSquareClick={(sq) => {
                if (preview) {
                  engineClearPreview();
                  return;
                }
                void handleSquareClick(sq);
              }}
            />

            <div className="flex items-center gap-2 text-sm">
              <span>行棋方：</span>
              <span
                className={
                  sideToColor(position.sideToMove) === "red"
                    ? "font-bold text-red-600"
                    : "font-bold text-gray-700"
                }
              >
                {sideLabel}
              </span>
              {snapshot.nags.length > 0 && (
                <span className="font-bold text-amber-500">{snapshot.nags.join("")}</span>
              )}
            </div>

            {/* 导航工具栏 */}
            <div className="flex flex-wrap items-center gap-1">
              <Button
                variant="outline"
                size="sm"
                title="回到起点 (Home)"
                aria-label="回到起点"
                onClick={() => void goToStart()}
                disabled={!snapshot.hasParent}
              >
                ⏮
              </Button>
              <Button
                variant="outline"
                size="sm"
                title="上一步 (←)"
                aria-label="上一步"
                onClick={() => void previous()}
                disabled={!snapshot.hasParent}
              >
                ←
              </Button>
              <Button
                variant="outline"
                size="sm"
                title="下一步 (→)"
                aria-label="下一步"
                onClick={() => void next()}
                disabled={!snapshot.nextMainId}
              >
                →
              </Button>
              <Button
                variant="outline"
                size="sm"
                title="走到终点 (End)"
                aria-label="走到终点"
                onClick={() => void goToEnd()}
                disabled={!snapshot.nextMainId}
              >
                ⏭
              </Button>
              <span className="mx-1 h-4 w-px bg-border" aria-hidden />
              <Button
                variant="outline"
                size="sm"
                title="悔棋 (Ctrl+Z)"
                aria-label="悔棋"
                onClick={() => void undo()}
                disabled={!snapshot.undoAvailable}
              >
                ↶
              </Button>
              <Button
                variant="outline"
                size="sm"
                title="重做 (Ctrl+Y)"
                aria-label="重做"
                onClick={() => void redo()}
                disabled={!snapshot.redoAvailable}
              >
                ↷
              </Button>
              <span className="mx-1 h-4 w-px bg-border" aria-hidden />
              <Button variant="outline" size="sm" title="翻转棋盘 (F)" onClick={rotateView}>
                ⇅
              </Button>
              <Button variant="outline" size="sm" title="左右镜像 (M)" onClick={mirrorView}>
                ⇄
              </Button>
              <Button
                variant={editing ? "default" : "outline"}
                size="sm"
                title="编辑局面"
                onClick={() => void toggleEditing()}
              >
                {editing ? "完成编辑" : "编辑局面"}
              </Button>
            </div>

            {editing && (
              <div className="flex flex-col gap-2 rounded border p-3">
                <PiecePalette tool={tool} onSelect={setTool} />
                <div className="flex gap-2">
                  <Button variant="outline" size="sm" onClick={() => void toggleSide()}>
                    切换先手方
                  </Button>
                  <Button variant="destructive" size="sm" onClick={() => void clearAll()}>
                    清空棋盘
                  </Button>
                </div>
              </div>
            )}
          </div>

          {/* 右列：功能标签页 */}
          <div className="flex min-w-0 flex-1 flex-col gap-3">
            <div className="flex flex-wrap gap-1 border-b pb-1" role="tablist">
              {TABS.map((t) => (
                <button
                  key={t.key}
                  type="button"
                  role="tab"
                  aria-selected={tab === t.key}
                  data-testid={`tab-${t.key}`}
                  onClick={() => setTab(t.key)}
                  className={`rounded-t px-3 py-1.5 text-sm transition-colors ${
                    tab === t.key
                      ? "border-b-2 border-primary font-semibold text-foreground"
                      : "text-muted-foreground hover:bg-accent"
                  }`}
                >
                  {t.label}
                </button>
              ))}
            </div>

            {tab === "game" && (
              <div className="flex flex-col gap-3">
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground">本地存档</span>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    data-testid="game-save"
                    onClick={() => void saveGame()}
                  >
                    保存棋局
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    data-testid="game-load"
                    onClick={() => void loadGame()}
                  >
                    载入棋局
                  </Button>
                </div>

                <MoveTree
                  tree={snapshot.tree}
                  currentId={snapshot.currentId}
                  expanded={expandedVariations}
                  onNavigate={(id) => void navigate(id)}
                  onToggleVariation={toggleVariation}
                  onDeleteVariation={(id) => void deleteVariation(id)}
                  onPromoteVariation={(id) => void promoteVariation(id)}
                  onReorderVariation={(parentId, from, to) =>
                    void reorderVariation(parentId, from, to)
                  }
                />

                <div className="flex flex-col gap-1">
                  <label htmlFor="comment" className="text-xs text-muted-foreground">
                    注释（当前棋步）
                  </label>
                  <textarea
                    id="comment"
                    value={commentDraft}
                    onChange={(event) => setCommentDraft(event.currentTarget.value)}
                    onBlur={() => void setComment(snapshot.currentId, commentDraft)}
                    rows={2}
                    placeholder="为当前棋步添加注释…"
                    className="rounded-md border border-input bg-background px-3 py-2 text-sm"
                  />
                  <div className="flex gap-1">
                    {["!", "?", "!!", "??", "!?", "?!"].map((nag) => {
                      const active = snapshot.nags.includes(nag);
                      return (
                        <Button
                          key={nag}
                          variant={active ? "default" : "outline"}
                          size="sm"
                          onClick={() => void setNag(snapshot.currentId, nag, !active)}
                        >
                          {nag}
                        </Button>
                      );
                    })}
                  </div>
                </div>

                <div className="flex flex-col gap-1">
                  <label htmlFor="fen" className="text-xs text-muted-foreground">
                    FEN
                  </label>
                  <div className="flex gap-2">
                    <input
                      id="fen"
                      value={fenInput}
                      onChange={(event) => setFenInput(event.currentTarget.value)}
                      placeholder="粘贴 FEN…"
                      className="h-9 flex-1 rounded-md border border-input bg-background px-3 text-sm"
                    />
                    <Button variant="secondary" size="sm" onClick={() => void loadFen(fenInput)}>
                      载入
                    </Button>
                  </div>
                  <p className="break-all font-mono text-xs text-muted-foreground">
                    {position.fen}
                  </p>
                </div>

                {validation &&
                  (validation.ok ? (
                    <p className="text-sm text-green-600">✓ 局面合法</p>
                  ) : (
                    <ul className="list-inside list-disc text-sm text-amber-600">
                      {validation.issues.map((issue, index) => (
                        <li key={index}>{issue}</li>
                      ))}
                    </ul>
                  ))}

                {message && <p className="text-sm text-amber-600">{message}</p>}
              </div>
            )}

            {tab === "analysis" && (
              <div className="flex flex-col gap-3">
                <AnalysisPanel
                  status={engineStatus}
                  engineId={engineId}
                  lines={engineLines}
                  bestMove={engineBestMove}
                  message={engineMessage}
                  onStart={() => void engineStart()}
                  onStop={() => void engineStop()}
                  onRestart={() => void engineRestart()}
                  onPreview={(pv) => {
                    if (snapshot) {
                      void enginePreviewPv(pv, snapshot.currentFen);
                    }
                  }}
                />
                <EvalCurve points={curvePoints} onClear={curveClear} />
                <AnalysisReport
                  status={analysisStatus}
                  progress={analysisProgress}
                  total={analysisTotal}
                  assessments={analysisAssessments}
                  onStart={() => void analysisStart(16, null)}
                  onStop={() => void analysisStop()}
                  onContinue={() => void analysisContinue()}
                  onRestart={() => void analysisStart(16, null)}
                  onNavigate={(nodeId) => void navigate(nodeId)}
                />
              </div>
            )}

            {tab === "book" && (
              <BookPanel
                bookApi={bookApi}
                currentFen={currentFen}
                onAutoMove={(snap) => adoptSnapshot(snap)}
              />
            )}

            {tab === "io" && (
              <div className="flex flex-col gap-3">
                <GameCodec ioApi={ioApi} onImported={(snap) => adoptSnapshot(snap)} />
                <OcrPanel ocrApi={ocrApi} onLoaded={(fen) => void loadFen(fen)} />
                <GifExportPanel gifApi={gifApi} variations={variations} />
              </div>
            )}

            {tab === "settings" && <SettingsPanel />}
          </div>
        </CardContent>
      </Card>
    </main>
  );
}

export default App;
