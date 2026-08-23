import type { ReactNode } from "react";
import type { TreeNodeDto } from "../../lib/game/types";

export interface MoveTreeProps {
  tree: TreeNodeDto;
  currentId: number;
  expanded: number[];
  onNavigate: (nodeId: number) => void;
  onToggleVariation: (nodeId: number) => void;
  onDeleteVariation: (nodeId: number) => void;
  onPromoteVariation: (nodeId: number) => void;
  onReorderVariation: (parentId: number, from: number, to: number) => void;
}

function MoveChip({
  node,
  isCurrent,
  onNavigate,
}: {
  node: TreeNodeDto;
  isCurrent: boolean;
  onNavigate: (id: number) => void;
}) {
  return (
    <button
      type="button"
      data-testid={`move-${node.id}`}
      onClick={() => onNavigate(node.id)}
      className={`inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-xs font-medium transition-colors ${
        isCurrent
          ? "border-primary bg-primary text-primary-foreground"
          : "border-input bg-background hover:bg-accent"
      }`}
    >
      {node.moveNumber > 0 && (
        <span className={isCurrent ? "" : "text-muted-foreground"}>
          {node.moveNumber}
          {node.isRed ? "." : "…"}
        </span>
      )}
      <span className="font-mono">{node.chineseMv || node.mv}</span>
      {node.nags.length > 0 && (
        <span className="font-bold text-amber-500">{node.nags.join("")}</span>
      )}
      {node.comment !== "" && (
        <span className="text-blue-500" title={node.comment} aria-label={`注释：${node.comment}`}>
          *
        </span>
      )}
    </button>
  );
}

interface RenderContext {
  currentId: number;
  expanded: number[];
  onNavigate: (id: number) => void;
  onToggleVariation: (id: number) => void;
  onDeleteVariation: (id: number) => void;
  onPromoteVariation: (id: number) => void;
  onReorderVariation: (parentId: number, from: number, to: number) => void;
}

function renderLine(start: TreeNodeDto, ctx: RenderContext): ReactNode[] {
  const out: ReactNode[] = [];
  let node: TreeNodeDto | null = start;
  while (node) {
    const current: TreeNodeDto = node;
    const variations = current.children.slice(1);
    const expandedHere = ctx.expanded.includes(current.id);
    out.push(
      <span key={current.id} className="inline-flex flex-wrap items-center gap-1">
        <MoveChip
          node={current}
          isCurrent={current.id === ctx.currentId}
          onNavigate={ctx.onNavigate}
        />
        {variations.length > 0 && (
          <button
            type="button"
            data-testid={`variation-toggle-${current.id}`}
            onClick={() => ctx.onToggleVariation(current.id)}
            className="inline-flex items-center gap-0.5 rounded border border-dashed px-1.5 py-0.5 text-xs text-muted-foreground hover:bg-accent"
          >
            <span aria-hidden>{expandedHere ? "▾" : "▸"}</span>变例 {variations.length}
          </button>
        )}
      </span>,
    );

    if (variations.length > 0 && expandedHere) {
      out.push(
        <div
          key={`variations-${current.id}`}
          className="flex flex-col gap-1 border-l border-dashed pl-2"
        >
          {variations.map((variation, index) => {
            const canUp = index > 0;
            const canDown = index < variations.length - 1;
            return (
              <div
                key={variation.id}
                data-testid={`variation-${variation.id}`}
                className="flex flex-col gap-0.5"
              >
                <div className="flex items-center gap-1 text-xs text-muted-foreground">
                  <span>变例 {index + 1}</span>
                  <button
                    type="button"
                    data-testid={`promote-variation-${variation.id}`}
                    aria-label={`提升变例 ${variation.id} 为主线`}
                    onClick={() => ctx.onPromoteVariation(variation.id)}
                    className="rounded px-1 hover:bg-accent"
                    title="提升为主线"
                  >
                    ⭐
                  </button>
                  <button
                    type="button"
                    data-testid={`reorder-variation-${variation.id}-up`}
                    aria-label={`变例 ${variation.id} 上移`}
                    onClick={() => ctx.onReorderVariation(current.id, index + 1, index)}
                    disabled={!canUp}
                    className="rounded px-1 hover:bg-accent disabled:opacity-30"
                    title="上移"
                  >
                    ↑
                  </button>
                  <button
                    type="button"
                    data-testid={`reorder-variation-${variation.id}-down`}
                    aria-label={`变例 ${variation.id} 下移`}
                    onClick={() => ctx.onReorderVariation(current.id, index + 1, index + 2)}
                    disabled={!canDown}
                    className="rounded px-1 hover:bg-accent disabled:opacity-30"
                    title="下移"
                  >
                    ↓
                  </button>
                  <button
                    type="button"
                    data-testid={`delete-variation-${variation.id}`}
                    aria-label={`删除变例 ${variation.id}`}
                    onClick={() => ctx.onDeleteVariation(variation.id)}
                    className="rounded px-1 text-red-500 hover:bg-red-50"
                  >
                    🗑
                  </button>
                </div>
                <div className="flex flex-wrap items-center gap-1">
                  {renderLine(variation, ctx)}
                </div>
              </div>
            );
          })}
        </div>,
      );
    }

    node = current.children[0] ?? null;
  }
  return out;
}

/** 棋谱树视图：主线 + 可变例展开/删除/提升/排序、当前棋步高亮、注释标记。 */
export function MoveTree({
  tree,
  currentId,
  expanded,
  onNavigate,
  onToggleVariation,
  onDeleteVariation,
  onPromoteVariation,
  onReorderVariation,
}: MoveTreeProps) {
  const ctx: RenderContext = {
    currentId,
    expanded,
    onNavigate,
    onToggleVariation,
    onDeleteVariation,
    onPromoteVariation,
    onReorderVariation,
  };
  const hasMoves = tree.children.length > 0;
  return (
    <div
      data-testid="move-tree"
      className="flex max-h-64 flex-col gap-1 overflow-y-auto rounded border p-2"
    >
      {hasMoves ? (
        <div className="flex flex-wrap items-center gap-1">{renderLine(tree, ctx)}</div>
      ) : (
        <p className="text-sm text-muted-foreground" data-testid="move-tree-empty">
          尚无着法——点击棋盘走子，或切换「编辑局面」摆棋。
        </p>
      )}
    </div>
  );
}
