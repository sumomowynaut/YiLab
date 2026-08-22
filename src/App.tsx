import { useEffect, useState } from "react";
import { Board } from "./components/board/Board";
import { PiecePalette } from "./components/board/PiecePalette";
import { Button } from "./components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { getDefaultBoardApi } from "./lib/board/api";
import { sideToColor } from "./lib/board/notation";
import { useBoardStore } from "./stores/useBoardStore";

function App() {
  const position = useBoardStore((state) => state.position);
  const validation = useBoardStore((state) => state.validation);
  const selected = useBoardStore((state) => state.selected);
  const legalTargets = useBoardStore((state) => state.legalTargets);
  const view = useBoardStore((state) => state.view);
  const editing = useBoardStore((state) => state.editing);
  const tool = useBoardStore((state) => state.tool);
  const message = useBoardStore((state) => state.message);
  const init = useBoardStore((state) => state.init);
  const handleSquareClick = useBoardStore((state) => state.handleSquareClick);
  const toggleEditing = useBoardStore((state) => state.toggleEditing);
  const setTool = useBoardStore((state) => state.setTool);
  const clearAll = useBoardStore((state) => state.clearAll);
  const toggleSide = useBoardStore((state) => state.toggleSide);
  const rotateView = useBoardStore((state) => state.rotateView);
  const mirrorView = useBoardStore((state) => state.mirrorView);
  const loadFen = useBoardStore((state) => state.loadFen);
  const [fenInput, setFenInput] = useState("");

  useEffect(() => {
    void init(getDefaultBoardApi());
  }, [init]);

  if (!position) {
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
          <CardDescription>中国象棋复盘与分析 — 棋盘核心（Phase 1）</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-6">
          <Board
            position={position}
            selected={selected}
            legalTargets={legalTargets}
            view={view}
            onSquareClick={(sq) => void handleSquareClick(sq)}
          />

          <div className="flex min-w-64 flex-1 flex-col gap-3">
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
            </div>

            <div className="flex flex-wrap gap-2">
              <Button variant="outline" size="sm" onClick={rotateView}>
                翻转棋盘
              </Button>
              <Button variant="outline" size="sm" onClick={mirrorView}>
                左右镜像
              </Button>
              <Button variant={editing ? "default" : "outline"} size="sm" onClick={toggleEditing}>
                编辑局面
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
