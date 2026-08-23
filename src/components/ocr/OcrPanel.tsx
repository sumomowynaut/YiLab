import { useRef, useState } from "react";
import type { OcrApi } from "../../lib/ocr/api";
import type { OcrResultDto } from "../../lib/ocr/types";
import { Button } from "../ui/button";

export interface OcrPanelProps {
  ocrApi: OcrApi;
  /** 把识别 FEN 载入棋谱（供用户手动修正）。 */
  onLoaded: (fen: string) => void;
}

function readFileBytes(file: File): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const buf = reader.result as ArrayBuffer;
      resolve(new Uint8Array(buf));
    };
    reader.onerror = () => reject(reader.error ?? new Error("读取文件失败"));
    reader.readAsArrayBuffer(file);
  });
}

/** 截图识别面板：本地识别 → 展示置信度/问题 → 载入人工修正。 */
export function OcrPanel({ ocrApi, onLoaded }: OcrPanelProps) {
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<OcrResultDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  async function handleFile(file: File | null) {
    if (!file) return;
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      const bytes = await readFileBytes(file);
      const out = await ocrApi.recognize(bytes);
      setResult(out);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  const uncertainCount = result?.cells.filter((c) => c.uncertain).length ?? 0;

  return (
    <div data-testid="ocr-panel" className="flex flex-col gap-3 rounded border p-3">
      <div className="text-sm font-semibold">截图识别（本地）</div>

      <div className="flex flex-wrap gap-2">
        <input
          ref={fileRef}
          type="file"
          accept="image/png,image/jpeg"
          data-testid="ocr-file"
          className="hidden"
          onChange={(event) => void handleFile(event.currentTarget.files?.[0] ?? null)}
        />
        <Button
          type="button"
          variant="outline"
          size="sm"
          data-testid="ocr-pick"
          disabled={busy}
          onClick={() => fileRef.current?.click()}
        >
          {busy ? "识别中…" : "选择截图并识别…"}
        </Button>
      </div>

      {error && (
        <p data-testid="ocr-error" className="text-xs text-red-500">
          识别失败：{error}
        </p>
      )}

      {result && (
        <div data-testid="ocr-result" className="flex flex-col gap-1 text-xs">
          <p>
            置信度 <span data-testid="ocr-confidence">{Math.round(result.confidence * 100)}%</span>
            {" · "}
            方向 {result.orientation === "flipped180" ? "已旋转 180°" : "正常"}
            {result.valid ? " · ✅ 通过规则校验" : " · ⚠️ 存在问题"}
          </p>
          {uncertainCount > 0 && (
            <p className="text-amber-600">有 {uncertainCount} 格识别不确定，请在棋盘上核对修正。</p>
          )}
          {result.issues.length > 0 && (
            <ul data-testid="ocr-issues" className="list-inside list-disc text-amber-600">
              {result.issues.map((issue, index) => (
                <li key={index}>{issue}</li>
              ))}
            </ul>
          )}
          <p className="break-all font-mono text-muted-foreground" data-testid="ocr-fen">
            {result.fen}
          </p>
          <div className="flex gap-2">
            <Button
              type="button"
              size="sm"
              data-testid="ocr-load"
              onClick={() => onLoaded(result.fen)}
            >
              载入棋谱（可再手动修正）
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
