import { useRef, useState } from "react";
import type { IoApi } from "../../lib/io/api";
import type { IoFormat } from "../../lib/io/types";
import type { GameSnapshot } from "../../lib/game/types";
import { Button } from "../ui/button";

export interface GameCodecProps {
  ioApi: IoApi;
  onImported: (snapshot: GameSnapshot) => void;
}

/** 棋谱导入导出入口：粘贴/文件导入（PGN/FEN，自动嗅探），复制/下载导出。 */
export function GameCodec({ ioApi, onImported }: GameCodecProps) {
  const [importText, setImportText] = useState("");
  const [importFormat, setImportFormat] = useState<IoFormat | "auto">("auto");
  const [exportFormat, setExportFormat] = useState<IoFormat>("pgn");
  const [message, setMessage] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  async function doImport(text: string, format: IoFormat | "") {
    try {
      const snapshot = await ioApi.importText(format, text);
      onImported(snapshot);
      setImportText("");
      setMessage("导入成功");
    } catch (error) {
      setMessage(`导入失败：${String(error)}`);
    }
  }

  function readFileText(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result ?? ""));
      reader.onerror = () => reject(reader.error ?? new Error("读取文件失败"));
      reader.readAsText(file);
    });
  }

  async function handleFile(file: File | null) {
    if (!file) return;
    try {
      const text = await readFileText(file);
      await doImport(text, "");
    } catch (error) {
      setMessage(`导入失败：${String(error)}`);
    }
  }

  async function handleExportCopy() {
    try {
      const text = await ioApi.exportText(exportFormat);
      await navigator.clipboard?.writeText(text);
      setMessage(`已复制 ${exportFormat.toUpperCase()} 到剪贴板`);
    } catch (error) {
      setMessage(`导出失败：${String(error)}`);
    }
  }

  async function handleExportDownload() {
    try {
      const text = await ioApi.exportText(exportFormat);
      const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `pikaxiangqi.${exportFormat}`;
      a.click();
      URL.revokeObjectURL(url);
      setMessage(`已导出 ${exportFormat.toUpperCase()} 文件`);
    } catch (error) {
      setMessage(`导出失败：${String(error)}`);
    }
  }

  return (
    <div data-testid="game-codec" className="flex flex-col gap-3 rounded border p-3">
      <div className="text-sm font-semibold">棋谱导入导出</div>

      <div className="flex flex-col gap-1">
        <label htmlFor="codec-import" className="text-xs text-muted-foreground">
          导入（粘贴 PGN / FEN）
        </label>
        <textarea
          id="codec-import"
          data-testid="codec-import"
          value={importText}
          onChange={(event) => setImportText(event.currentTarget.value)}
          rows={4}
          placeholder="在此粘贴棋谱文本…"
          className="rounded-md border border-input bg-background px-3 py-2 font-mono text-xs"
        />
        <div className="flex flex-wrap items-center gap-2">
          <select
            data-testid="codec-import-format"
            value={importFormat}
            onChange={(event) => setImportFormat(event.currentTarget.value as IoFormat | "auto")}
            className="h-8 rounded-md border border-input bg-background px-2 text-xs"
          >
            <option value="auto">自动识别</option>
            <option value="pgn">PGN</option>
            <option value="fen">FEN</option>
          </select>
          <Button
            type="button"
            size="sm"
            data-testid="codec-import-button"
            disabled={importText.trim() === ""}
            onClick={() => void doImport(importText, importFormat === "auto" ? "" : importFormat)}
          >
            导入
          </Button>
          <input
            ref={fileRef}
            type="file"
            accept=".pgn,.fen,.txt"
            data-testid="codec-file"
            className="hidden"
            onChange={(event) => void handleFile(event.currentTarget.files?.[0] ?? null)}
          />
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => fileRef.current?.click()}
          >
            打开文件…
          </Button>
        </div>
      </div>

      <div className="flex flex-col gap-1">
        <label htmlFor="codec-export" className="text-xs text-muted-foreground">
          导出当前棋谱
        </label>
        <div className="flex flex-wrap items-center gap-2">
          <select
            data-testid="codec-export-format"
            value={exportFormat}
            onChange={(event) => setExportFormat(event.currentTarget.value as IoFormat)}
            className="h-8 rounded-md border border-input bg-background px-2 text-xs"
          >
            <option value="pgn">PGN</option>
            <option value="fen">FEN</option>
          </select>
          <Button
            type="button"
            variant="outline"
            size="sm"
            data-testid="codec-copy"
            onClick={() => void handleExportCopy()}
          >
            复制
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            data-testid="codec-download"
            onClick={() => void handleExportDownload()}
          >
            下载文件
          </Button>
        </div>
      </div>

      {message && (
        <p data-testid="codec-message" className="text-xs text-muted-foreground">
          {message}
        </p>
      )}
    </div>
  );
}
