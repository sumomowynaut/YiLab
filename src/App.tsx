import { useEffect, useState } from "react";
import { Board } from "./components/board/Board";
import { MoveTree } from "./components/board/MoveTree";
import { PiecePalette } from "./components/board/PiecePalette";
import { Button } from "./components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { getDefaultBoardApi } from "./lib/board/api";
import { sideToColor } from "./lib/board/notation";
import { getDefaultGameApi } from "./lib/game/api";
import { useGameStore } from "./stores/useGameStore";

function App() {
  const snapshot = useGameStore((state) => state.snapshot);
  const position = useGameStore((state) => state.position);
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
  const setComment = useGameStore((state) => state.setComment);
  const setNag = useGameStore((state) => state.setNag);
  const toggleVariation = useGameStore((state) => state.toggleVariation);
  const [fenInput, setFenInput] = useState("");
  const [commentDraft, setCommentDraft] = useState("");

  useEffect(() => {
    void init(getDefaultGameApi(), getDefaultBoardApi());
  }, [init]);

  // 同步注释草稿
  useEffect(() => {
    setCommentDraft(snapshot?.comment ?? "");
  }, [snapshot?.comment]);

  // 键盘导航：←/→ 上一步/下一步，Ctrl+Z / Ctrl+Y 悔棋/重做
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const el = document.activeElement;
      if (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA")) {
        return;
      }
      if (event.key === "ArrowLeft") {
        event.preventDefault();
        void previous();
      } else if (event.key === "ArrowRight") {
        event.preventDefault();
        void next();
      } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "z") {
        event.preventDefault();
        if (event.shiftKey) {
          void redo();
        } else {
          void undo();
        }
      } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "y") {
        event.preventDefault();
        void redo();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [previous, next, undo, redo]);

  if (!snapshot || !position) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-background p-6 text-foreground">
        <p>加载中…</p>
      </main>
    );
  }

  const sideLabel = sideToColor(position.sideToMove) === "red" ? "红方" : "黑方";

  return (
    <main className="flex min-h-screen items-start justify-center bg-background p-6 text-foreground">
      <Card className="w-full max-w-5xl">
        <CardHeader>
          <CardTitle>PikaXiangqi</CardTitle>
          <CardDescription>中国象棋复盘与分析 — 棋谱树（Phase 2）</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-6">
          <Board
            position={position}
            selected={selected}
            legalTargets={legalTargets}
            view={view}
            onSquareClick={(sq) => void handleSquareClick(sq)}
          />

          <div className="flex min-w-72 flex-1 flex-col gap-3">
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

            <div className="flex flex-wrap gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => void goToStart()}
                disabled={!snapshot.hasParent}
              >
                ⏮
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void previous()}
                disabled={!snapshot.hasParent}
              >
                ←
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void next()}
                disabled={snapshot.nextMainId === null}
              >
                →
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void goToEnd()}
                disabled={snapshot.nextMainId === null}
              >
                ⏭
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void undo()}
                disabled={!snapshot.undoAvailable}
              >
                悔棋
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void redo()}
                disabled={!snapshot.redoAvailable}
              >
                重做
              </Button>
            </div>

            <MoveTree
              tree={snapshot.tree}
              currentId={snapshot.currentId}
              expanded={expandedVariations}
              onNavigate={(id) => void navigate(id)}
              onToggleVariation={toggleVariation}
              onDeleteVariation={(id) => void deleteVariation(id)}
            />

            <div className="flex flex-col gap-1">
              <label htmlFor="comment" className="text-xs text-muted-foreground">
                注释（当前棋步）
              </label>
              <textarea
                id="comment"
                value={commentDraft}
                onChange={(event) => setCommentDraft(event.currentTarget.value)}
                onBlur={() => void setComment(commentDraft)}
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
                      onClick={() => void setNag(nag, !active)}
                    >
                      {nag}
                    </Button>
                  );
                })}
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              <Button variant="outline" size="sm" onClick={rotateView}>
                翻转棋盘
              </Button>
              <Button variant="outline" size="sm" onClick={mirrorView}>
                左右镜像
              </Button>
              <Button
                variant={editing ? "default" : "outline"}
                size="sm"
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
              <p className="break-all font-mono text-xs text-muted-foreground">{position.fen}</p>
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
        </CardContent>
      </Card>
    </main>
  );
}

export default App;
