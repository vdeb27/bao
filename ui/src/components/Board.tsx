import { useEffect, useMemo, useRef } from "react";
import {
  moveCategory,
  phaseTag,
  substate,
  substateTag,
  type BoardState,
  type Direction,
  type Move,
  type PitFocus,
} from "../engine";
import { useT } from "../i18n";

/** Layout-agnostic descriptor of a single pit: which side it belongs to,
 * the vichwa index inside that side, and its screen-space top-left corner. */
type PitDescriptor = {
  player: 0 | 1;
  vichwa: number;
  /** 0..3: north-nyuma=0, north-mbele=1, south-mbele=2, south-nyuma=3. */
  logicalRow: number;
  /** 0..7: screen column from South's perspective (a..h). */
  logicalCol: number;
  /** Whether this pit is mbele (capture row). */
  mbele: boolean;
  x: number;
  y: number;
};

export type Orientation = "landscape" | "portrait";

const PIT_SIZE = 72;
const PIT_GAP = 8;
const EQUATOR_GAP = 20;
const GHALA_THICKNESS = 56;
const PADDING = 24;

const LOGICAL_COLS = 8;
const LOGICAL_ROWS = 4;

type Geometry = {
  width: number;
  height: number;
  ghala: {
    south: { x: number; y: number; w: number; h: number };
    north: { x: number; y: number; w: number; h: number };
  };
  pits: PitDescriptor[];
};

function pitFor(logicalRow: number, logicalCol: number): {
  player: 0 | 1;
  vichwa: number;
  mbele: boolean;
} {
  // logicalRow ordering: 0=north-nyuma, 1=north-mbele, 2=south-mbele, 3=south-nyuma.
  // The vichwa indexing matches the landscape board.rs comment.
  switch (logicalRow) {
    case 0:
      return { player: 1, vichwa: 8 + logicalCol, mbele: false };
    case 1:
      return { player: 1, vichwa: 7 - logicalCol, mbele: true };
    case 2:
      return { player: 0, vichwa: logicalCol, mbele: true };
    case 3:
      return { player: 0, vichwa: 15 - logicalCol, mbele: false };
    default:
      throw new Error(`bad row ${logicalRow}`);
  }
}

function buildLandscape(): Geometry {
  // 8 cols × 4 rows. South at the bottom, North at the top. Each ghala
  // spans only its own player's two rows so it's visually clear which
  // side it belongs to (north's = upper half, south's = lower half).
  const boardInnerW = LOGICAL_COLS * PIT_SIZE + (LOGICAL_COLS - 1) * PIT_GAP;
  const boardInnerH =
    LOGICAL_ROWS * PIT_SIZE + (LOGICAL_ROWS - 1) * PIT_GAP + EQUATOR_GAP - PIT_GAP;
  const width = boardInnerW + 2 * GHALA_THICKNESS + 4 * PADDING;
  const height = boardInnerH + 2 * PADDING;
  const boardLeft = GHALA_THICKNESS + 2 * PADDING;
  const boardTop = PADDING;
  // Each side's row-pair height: two pits + the pit-gap between them.
  const sideHeight = 2 * PIT_SIZE + PIT_GAP;
  const pits: PitDescriptor[] = [];
  for (let r = 0; r < LOGICAL_ROWS; r++) {
    let y = boardTop;
    for (let rr = 0; rr < r; rr++) y += PIT_SIZE + (rr === 1 ? EQUATOR_GAP : PIT_GAP);
    for (let c = 0; c < LOGICAL_COLS; c++) {
      const x = boardLeft + c * (PIT_SIZE + PIT_GAP);
      const info = pitFor(r, c);
      pits.push({ ...info, logicalRow: r, logicalCol: c, x, y });
    }
  }
  return {
    width,
    height,
    ghala: {
      // North's ghala aligned to rows 0+1 (top half).
      north: {
        x: width - PADDING - GHALA_THICKNESS,
        y: boardTop,
        w: GHALA_THICKNESS,
        h: sideHeight,
      },
      // South's ghala aligned to rows 2+3 (bottom half).
      south: {
        x: PADDING,
        y: boardTop + sideHeight + EQUATOR_GAP,
        w: GHALA_THICKNESS,
        h: sideHeight,
      },
    },
    pits,
  };
}

function buildPortrait(): Geometry {
  // Rotated 90° counter-clockwise: 4 cols × 8 rows. Portrait col index ↔
  // landscape row index (3 - portCol). Portrait row index ↔ landscape col
  // index. South ends up on the left two cols, North on the right two.
  const boardInnerW =
    LOGICAL_ROWS * PIT_SIZE + (LOGICAL_ROWS - 1) * PIT_GAP + EQUATOR_GAP - PIT_GAP;
  const boardInnerH = LOGICAL_COLS * PIT_SIZE + (LOGICAL_COLS - 1) * PIT_GAP;
  const width = boardInnerW + 2 * GHALA_THICKNESS + 4 * PADDING;
  const height = boardInnerH + 2 * PADDING;
  const boardLeft = GHALA_THICKNESS + 2 * PADDING;
  const boardTop = PADDING;
  const sideHeight = boardInnerH;
  // South's two cols (port cols 0 and 1) occupy this much width on screen.
  const sideWidth = 2 * PIT_SIZE + PIT_GAP;
  const pits: PitDescriptor[] = [];
  for (let portRow = 0; portRow < LOGICAL_COLS; portRow++) {
    for (let portCol = 0; portCol < LOGICAL_ROWS; portCol++) {
      const landRow = 3 - portCol;
      const landCol = portRow;
      let x = boardLeft;
      for (let pc = 0; pc < portCol; pc++)
        x += PIT_SIZE + (pc === 1 ? EQUATOR_GAP : PIT_GAP);
      const y = boardTop + portRow * (PIT_SIZE + PIT_GAP);
      const info = pitFor(landRow, landCol);
      pits.push({ ...info, logicalRow: landRow, logicalCol: landCol, x, y });
    }
  }
  return {
    width,
    height,
    ghala: {
      // South ghala on the left of South's two columns (port cols 0, 1).
      south: {
        x: PADDING,
        y: boardTop,
        w: GHALA_THICKNESS,
        h: sideHeight,
      },
      // North ghala on the right of North's two columns (port cols 2, 3).
      // boardLeft + sideWidth + EQUATOR_GAP gets us past South's cols and
      // the equator gap into North's territory.
      north: {
        x: boardLeft + sideWidth + EQUATOR_GAP + sideWidth + PADDING,
        y: boardTop,
        w: GHALA_THICKNESS,
        h: sideHeight,
      },
    },
    pits,
  };
}

function buildGeometry(orientation: Orientation): Geometry {
  return orientation === "landscape" ? buildLandscape() : buildPortrait();
}

function colLetter(col: number): string {
  return String.fromCharCode("a".charCodeAt(0) + col);
}

function pitAriaLabel(t: ReturnType<typeof useT>, p: PitDescriptor, count: number, legal: boolean): string {
  const playerName = p.player === 0 ? t("south") : t("north");
  const row = p.mbele ? "mbele" : "nyuma";
  return `${playerName} ${row} ${colLetter(p.logicalCol)}, ${count} kete${legal ? " (legal)" : ""}`;
}

function isNyumba(view: BoardState, p: PitDescriptor): boolean {
  if (view.variant !== "Kiswahili") return false;
  const side = view.sides[p.player];
  return side.nyumba_owned && p.vichwa === side.nyumba_col && p.mbele;
}

function isKutakatiaBlocked(view: BoardState, p: PitDescriptor): boolean {
  if (!view.kutakatia) return false;
  return (
    view.kutakatia.blocked_player === p.player &&
    view.kutakatia.blocked_field === p.vichwa
  );
}

function movesForPit(view: BoardState, moves: Move[], p: PitDescriptor): Move[] {
  if (p.player !== view.active) return [];
  const phaseT = phaseTag(view.phase);
  return moves.filter((m) => {
    const cat = moveCategory(m);
    if (phaseT === "Namu" && cat === "Namu") {
      return p.vichwa < 8 && (m as { Namu: { col: number } }).Namu.col === p.vichwa;
    }
    if (phaseT === "Mtaji" && cat === "Mtaji") {
      return (m as { Mtaji: { pit: number } }).Mtaji.pit === p.vichwa;
    }
    return false;
  });
}

export type DirectionPick = {
  pit: PitDescriptor;
  candidates: Move[];
};

type BoardProps = {
  view: BoardState;
  moves: Move[];
  focus: PitFocus | null;
  animating: boolean;
  orientation: Orientation;
  onPlay: (move: Move) => void;
  onAmbiguous: (pick: DirectionPick) => void;
};

export function Board({
  view,
  moves,
  focus,
  animating,
  orientation,
  onPlay,
  onAmbiguous,
}: BoardProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const t = useT();
  const geometry = useMemo(() => buildGeometry(orientation), [orientation]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    draw(ctx, view, animating ? [] : moves, focus, geometry, t);
  }, [view, moves, focus, animating, geometry, t]);

  // Flying-kete sprite: a single absolutely-positioned dot follows the
  // animation focus across pits. The browser interpolates `transform`
  // between focus changes via the CSS transition, giving the visual a
  // sense of motion without us scheduling per-frame paints.
  const spritePos = focus
    ? (() => {
        const p = geometry.pits.find(
          (pp) => pp.player === focus.player && pp.vichwa === focus.vichwa,
        );
        return p ? { x: p.x + PIT_SIZE / 2, y: p.y + PIT_SIZE / 2 } : null;
      })()
    : null;

  const handlePitActivate = (p: PitDescriptor) => {
    if (animating) return;
    if (substateTag(substate(view.phase)) !== "AwaitMove") return;
    if (view.winner !== null) return;
    const candidates = movesForPit(view, moves, p);
    if (candidates.length === 1) {
      onPlay(candidates[0]);
    } else if (candidates.length > 1) {
      onAmbiguous({ pit: p, candidates });
    }
  };

  return (
    <div
      className="bao-board-canvas-wrap"
      style={{ width: geometry.width, height: geometry.height }}
    >
      <canvas
        ref={canvasRef}
        width={geometry.width}
        height={geometry.height}
        className="bao-board"
        aria-hidden="true"
      />
      {animating && spritePos && (
        <div
          className="bao-kete-sprite"
          aria-hidden="true"
          style={{
            transform: `translate(${spritePos.x - 8}px, ${spritePos.y - 8}px)`,
          }}
        />
      )}
      <div className="bao-board-overlay" role="grid" aria-label={t("materialBalance")}>
        {geometry.pits.map((p) => {
          const count = view.sides[p.player].vichwa[p.vichwa];
          const candidates = movesForPit(view, moves, p);
          const legal = !animating && candidates.length > 0;
          return (
            <button
              key={`${p.player}:${p.vichwa}`}
              type="button"
              className={`bao-pit-btn${legal ? " bao-pit-legal" : ""}`}
              style={{
                left: p.x,
                top: p.y,
                width: PIT_SIZE,
                height: PIT_SIZE,
              }}
              aria-label={pitAriaLabel(t, p, count, legal)}
              aria-disabled={!legal}
              tabIndex={legal ? 0 : -1}
              onClick={() => handlePitActivate(p)}
            />
          );
        })}
      </div>
    </div>
  );
}

type DrawT = ReturnType<typeof useT>;

function draw(
  ctx: CanvasRenderingContext2D,
  view: BoardState,
  moves: Move[],
  focus: PitFocus | null,
  geometry: Geometry,
  t: DrawT,
) {
  ctx.clearRect(0, 0, geometry.width, geometry.height);
  ctx.fillStyle = "#3b2a1d";
  ctx.fillRect(0, 0, geometry.width, geometry.height);

  drawGhala(ctx, geometry.ghala.south, t("south"), view.sides[0].ghala, view.active === 0);
  drawGhala(ctx, geometry.ghala.north, t("north"), view.sides[1].ghala, view.active === 1);

  const legalKeys = new Set<string>();
  for (const m of moves) {
    if ("Namu" in m) legalKeys.add(`${view.active}:${m.Namu.col}`);
    else if ("Mtaji" in m) legalKeys.add(`${view.active}:${m.Mtaji.pit}`);
  }

  for (const p of geometry.pits) {
    const side = view.sides[p.player];
    const count = side.vichwa[p.vichwa];
    const isOwnSide = view.active === p.player;
    const legal = isOwnSide && legalKeys.has(`${p.player}:${p.vichwa}`);
    const nyumba = isNyumba(view, p);
    const blocked = isKutakatiaBlocked(view, p);
    const flashing = focus !== null && focus.player === p.player && focus.vichwa === p.vichwa;
    drawPit(ctx, p.x, p.y, count, { legal, nyumba, blocked, owned: isOwnSide, flashing });
  }
}

function drawGhala(
  ctx: CanvasRenderingContext2D,
  rect: { x: number; y: number; w: number; h: number },
  label: string,
  count: number,
  active: boolean,
) {
  ctx.fillStyle = active ? "#5d4a36" : "#4a3826";
  ctx.strokeStyle = active ? "#d4b886" : "#6b5440";
  ctx.lineWidth = 2;
  roundedRect(ctx, rect.x, rect.y, rect.w, rect.h, 10);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = "#e8d3a8";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.font = "14px system-ui, sans-serif";
  // For wide-but-short ghalas (portrait) we just stack label/value horizontally.
  const horiz = rect.w > rect.h;
  if (horiz) {
    ctx.fillText(label, rect.x + 40, rect.y + rect.h / 2);
    ctx.font = "bold 24px system-ui, sans-serif";
    ctx.fillText(String(count), rect.x + rect.w / 2, rect.y + rect.h / 2);
    ctx.font = "12px system-ui, sans-serif";
    ctx.fillText("ghala", rect.x + rect.w - 40, rect.y + rect.h / 2);
  } else {
    ctx.fillText(label, rect.x + rect.w / 2, rect.y + 16);
    ctx.font = "bold 24px system-ui, sans-serif";
    ctx.fillText(String(count), rect.x + rect.w / 2, rect.y + rect.h / 2);
    ctx.font = "12px system-ui, sans-serif";
    ctx.fillText("ghala", rect.x + rect.w / 2, rect.y + rect.h - 16);
  }
}

type PitStyle = {
  legal: boolean;
  nyumba: boolean;
  blocked: boolean;
  owned: boolean;
  flashing: boolean;
};

function drawPit(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  count: number,
  style: PitStyle,
) {
  let fill = "#4a3826";
  if (style.flashing) fill = "#8b6a3a";
  else if (style.blocked) fill = "#5b2b2b";
  else if (style.nyumba) fill = "#5a4226";
  else if (style.owned) fill = "#54402c";

  ctx.fillStyle = fill;
  ctx.strokeStyle = style.flashing
    ? "#fde68a"
    : style.legal
      ? "#facc15"
      : style.nyumba
        ? "#d4a574"
        : "#6b5440";
  ctx.lineWidth = style.flashing ? 4 : style.legal ? 3 : 2;
  roundedRect(ctx, x, y, PIT_SIZE, PIT_SIZE, style.nyumba ? 14 : 10);
  ctx.fill();
  ctx.stroke();

  if (style.nyumba) {
    ctx.strokeStyle = "#d4a574";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(x + PIT_SIZE / 2, y + PIT_SIZE / 2, PIT_SIZE * 0.35, 0, Math.PI * 2);
    ctx.stroke();
  }

  // Colorblind supplement: a small star marker inside legal pits, so the
  // yellow rim isn't the only signal.
  if (style.legal) {
    drawStar(ctx, x + PIT_SIZE - 14, y + 14, 5);
  }

  ctx.fillStyle = count > 0 ? "#f3e1bd" : "#7d6648";
  ctx.font = "bold 22px system-ui, sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(String(count), x + PIT_SIZE / 2, y + PIT_SIZE / 2);

  if (style.blocked) {
    ctx.strokeStyle = "#f87171";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x + 8, y + 8);
    ctx.lineTo(x + PIT_SIZE - 8, y + PIT_SIZE - 8);
    ctx.stroke();
  }
}

function drawStar(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number) {
  ctx.fillStyle = "#facc15";
  ctx.beginPath();
  for (let i = 0; i < 10; i++) {
    const a = (i * Math.PI) / 5 - Math.PI / 2;
    const rr = i % 2 === 0 ? r : r * 0.45;
    const px = cx + Math.cos(a) * rr;
    const py = cy + Math.sin(a) * rr;
    if (i === 0) ctx.moveTo(px, py);
    else ctx.lineTo(px, py);
  }
  ctx.closePath();
  ctx.fill();
}

function roundedRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

export type { PitDescriptor, Direction };
