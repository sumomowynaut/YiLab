import { useState } from "react";
import type { GifApi } from "../../lib/gif/api";
import type { GifSource, VariationOption } from "../../lib/gif/types";
import { Button } from "../ui/button";

export interface GifExportPanelProps {
  gifApi: GifApi;
  /** 可选变例列表（用于「指定变例」）。 */
  variations: VariationOption[];
}

function downloadGif(bytes: Uint8Array, name: string): void {
  const blob = new Blob([bytes], { type: "image/gif" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  URL.revokeObjectURL(url);
}

/** GIF 导出面板：来源（当前局面/主线/指定变例）+ 帧间隔/尺寸/坐标/棋步 + 下载。 */
export function GifExportPanel({ gifApi, variations }: GifExportPanelProps) {
  const [source, setSource] = useState<GifSource>("mainline");
  const [variationNode, setVariationNode] = useState<number>(variations[0]?.nodeId ?? 0);
  const [frameDelayMs, setFrameDelayMs] = useState(500);
  const [cellSize, setCellSize] = useState(48);
  const [showCoordinates, setShowCoordinates] = useState(true);
  const [showMoves, setShowMoves] = useState(true);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const options = { frameDelayMs, cellSize, showCoordinates, showMoves };

  async function handleExport() {
    setBusy(true);
    setMessage(null);
    try {
      const bytes =
        source === "current"
          ? await gifApi.exportCurrent(options)
          : source === "mainline"
            ? await gifApi.exportMainline(options)
            : await gifApi.exportVariation(variationNode, options);
      downloadGif(bytes, `pikaxiangqi-${source}.gif`);
      setMessage("已导出 GIF");
    } catch (error) {
      setMessage(`导出失败：${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div data-testid="gif-panel" className="flex flex-col gap-3 rounded border p-3">
      <div className="text-sm font-semibold">GIF 导出</div>

      <div className="flex flex-col gap-1">
        <label htmlFor="gif-source" className="text-xs text-muted-foreground">
          来源
        </label>
        <select
          id="gif-source"
          data-testid="gif-source"
          value={source}
          onChange={(event) => setSource(event.currentTarget.value as GifSource)}
          className="h-8 rounded-md border border-input bg-background px-2 text-xs"
        >
          <option value="current">当前局面</option>
          <option value="mainline">主线</option>
          <option value="variation">指定变例</option>
        </select>
      </div>

      {source === "variation" && (
        <div className="flex flex-col gap-1">
          <label htmlFor="gif-variation" className="text-xs text-muted-foreground">
            变例
          </label>
          <select
            id="gif-variation"
            data-testid="gif-variation"
            value={variationNode}
            onChange={(event) => setVariationNode(Number(event.currentTarget.value))}
            className="h-8 rounded-md border border-input bg-background px-2 text-xs"
          >
            {variations.length === 0 && <option value={0}>（无变例）</option>}
            {variations.map((v) => (
              <option key={v.nodeId} value={v.nodeId}>
                {v.label}
              </option>
            ))}
          </select>
        </div>
      )}

      <div className="flex flex-wrap gap-3">
        <label className="flex flex-col gap-1 text-xs text-muted-foreground">
          帧间隔（ms）
          <input
            data-testid="gif-delay"
            type="number"
            min={100}
            step={100}
            value={frameDelayMs}
            onChange={(event) => setFrameDelayMs(Number(event.currentTarget.value) || 100)}
            className="h-8 w-24 rounded-md border border-input bg-background px-2 text-sm"
          />
        </label>
        <label className="flex flex-col gap-1 text-xs text-muted-foreground">
          棋盘尺寸
          <select
            data-testid="gif-cell"
            value={cellSize}
            onChange={(event) => setCellSize(Number(event.currentTarget.value))}
            className="h-8 rounded-md border border-input bg-background px-2 text-xs"
          >
            <option value={32}>小（32px）</option>
            <option value={48}>中（48px）</option>
            <option value={64}>大（64px）</option>
          </select>
        </label>
      </div>

      <div className="flex gap-4 text-xs">
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            data-testid="gif-coords"
            checked={showCoordinates}
            onChange={(event) => setShowCoordinates(event.currentTarget.checked)}
          />
          显示坐标
        </label>
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            data-testid="gif-moves"
            checked={showMoves}
            onChange={(event) => setShowMoves(event.currentTarget.checked)}
          />
          显示棋步
        </label>
      </div>

      <Button
        type="button"
        size="sm"
        data-testid="gif-export"
        disabled={busy || (source === "variation" && variations.length === 0)}
        onClick={() => void handleExport()}
      >
        {busy ? "导出中…" : "导出 GIF"}
      </Button>

      {message && (
        <p data-testid="gif-message" className="text-xs text-muted-foreground">
          {message}
        </p>
      )}
    </div>
  );
}
