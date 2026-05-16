import { useEffect, useRef } from "react";
import {
  moveCategory,
  phaseTag,
  substate,
  substateTag,
  type BoardState,
  type Direction,
  type Move,
} from "../engine";

// Canvas layout constants. The four screen rows top-to-bottom are:
//   row 0: north's nyuma (back row)         vichwa idx = 8 + screenCol
//   row 1: north's mbele (capture row)      vichwa idx = 7 - screenCol
//   row 2: south's mbele (capture row)      vichwa idx = screenCol
//   row 3: south's nyuma (back row)         vichwa idx = 15 - screenCol
const COLS = 8;
const ROWS = 4;
const PIT_SIZE = 72;
const PIT_GAP = 8;
const EQUATOR_GAP = 20;
const GHALA_WIDTH = 56;
const PADDING = 24;

const BOARD_INNER_WIDTH = COLS * PIT_SIZE + (COLS - 1) * PIT_GAP;
const BOARD_INNER_HEIGHT =
  ROWS * PIT_SIZE + (ROWS - 1) * PIT_GAP + EQUATOR_GAP - PIT_GAP;
const CANVAS_WIDTH = BOARD_INNER_WIDTH + 2 * GHALA_WIDTH + 4 * PADDING;
const CANVAS_HEIGHT = BOARD_INNER_HEIGHT + 2 * PADDING;

const BOARD_LEFT = GHALA_WIDTH + 2 * PADDING;
const BOARD_TOP = PADDING;

type PitCoord = { player: 0 | 1; vichwa: number; screenRow: number; screenCol: number };

function pitsForRow(screenRow: number): PitCoord[] {
  const out: PitCoord[] = [];
  for (let c = 0; c < COLS; c++) {
    let player: 0 | 1;
    let vichwa: number;
    switch (screenRow) {
      case 0:
        player = 1;
        vichwa = 8 + c;
        break;
      case 1:
        player = 1;
        vichwa = 7 - c;
        break;
      case 2:
        player = 0;
        vichwa = c;
        break;
      case 3:
        player = 0;
        vichwa = 15 - c;
        break;
      default:
        throw new Error(`bad row ${screenRow}`);
    }
    out.push({ player, vichwa, screenRow, screenCol: c });
  }
  return out;
}

function rowY(screenRow: number): number {
  let y = BOARD_TOP;
  for (let r = 0; r < screenRow; r++) {
    y += PIT_SIZE + (r === 1 ? EQUATOR_GAP : PIT_GAP);
  }
  return y;
}

function colX(screenCol: number): number {
  return BOARD_LEFT + screenCol * (PIT_SIZE + PIT_GAP);
}

function findPitAt(x: number, y: number): PitCoord | null {
  for (let r = 0; r < ROWS; r++) {
    const py = rowY(r);
    if (y < py || y > py + PIT_SIZE) continue;
    for (let c = 0; c < COLS; c++) {
      const px = colX(c);
      if (x >= px && x <= px + PIT_SIZE) {
        return pitsForRow(r)[c];
      }
    }
  }
  return null;
}

function isNyumba(view: BoardState, coord: PitCoord): boolean {
  if (view.variant !== "Kiswahili") return false;
  const side = view.sides[coord.player];
  return (
    side.nyumba_owned &&
    coord.vichwa === side.nyumba_col &&
    // Only mbele pits can be nyumba.
    coord.vichwa < 8
  );
}

function isKutakatiaBlocked(view: BoardState, coord: PitCoord): boolean {
  if (!view.kutakatia) return false;
  return (
    view.kutakatia.blocked_player === coord.player &&
    view.kutakatia.blocked_field === coord.vichwa
  );
}

function movesForCoord(
  view: BoardState,
  moves: Move[],
  coord: PitCoord,
): Move[] {
  if (coord.player !== view.active) return [];
  const phaseT = phaseTag(view.phase);
  return moves.filter((m) => {
    const cat = moveCategory(m);
    if (phaseT === "Namu" && cat === "Namu") {
      // Namu uses mbele col (0..7) = vichwa index for mbele pits.
      return coord.vichwa < 8 && (m as { Namu: { col: number } }).Namu.col === coord.vichwa;
    }
    if (phaseT === "Mtaji" && cat === "Mtaji") {
      return (m as { Mtaji: { pit: number } }).Mtaji.pit === coord.vichwa;
    }
    return false;
  });
}

export type DirectionPick = {
  coord: PitCoord;
  candidates: Move[]; // length 2, one Cw + one Ccw
};

type BoardProps = {
  view: BoardState;
  moves: Move[];
  /** Called when the click resolves to exactly one legal move. */
  onPlay: (move: Move) => void;
  /** Called when a pit click yields multiple legal moves (CW / CCW choice). */
  onAmbiguous: (pick: DirectionPick) => void;
};

export function Board({ view, moves, onPlay, onAmbiguous }: BoardProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    draw(ctx, view, moves);
  }, [view, moves]);

  const handleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    // Substates short-circuit pit clicks; SubstatePrompt handles those.
    if (substateTag(substate(view.phase)) !== "AwaitMove") return;
    if (view.winner !== null) return;
    const rect = canvas.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * canvas.width;
    const y = ((e.clientY - rect.top) / rect.height) * canvas.height;
    const coord = findPitAt(x, y);
    if (!coord) return;
    const candidates = movesForCoord(view, moves, coord);
    if (candidates.length === 1) {
      onPlay(candidates[0]);
    } else if (candidates.length > 1) {
      onAmbiguous({ coord, candidates });
    }
  };

  return (
    <canvas
      ref={canvasRef}
      width={CANVAS_WIDTH}
      height={CANVAS_HEIGHT}
      className="bao-board"
      onClick={handleClick}
      role="grid"
      aria-label="Bao bord"
    />
  );
}

function draw(ctx: CanvasRenderingContext2D, view: BoardState, moves: Move[]) {
  ctx.clearRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);

  // Background panel.
  ctx.fillStyle = "#3b2a1d";
  ctx.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);

  // Ghala rectangles, left = south, right = north.
  drawGhala(ctx, PADDING, "South", view.sides[0].ghala, view.active === 0);
  drawGhala(
    ctx,
    CANVAS_WIDTH - PADDING - GHALA_WIDTH,
    "North",
    view.sides[1].ghala,
    view.active === 1,
  );

  // Legal-move targets, indexed by "player:vichwa".
  const legalKeys = new Set<string>();
  for (const m of moves) {
    if ("Namu" in m) {
      legalKeys.add(`${view.active}:${m.Namu.col}`);
    } else if ("Mtaji" in m) {
      legalKeys.add(`${view.active}:${m.Mtaji.pit}`);
    }
  }

  for (let r = 0; r < ROWS; r++) {
    const pits = pitsForRow(r);
    const y = rowY(r);
    for (let c = 0; c < COLS; c++) {
      const coord = pits[c];
      const x = colX(c);
      const side = view.sides[coord.player];
      const count = side.vichwa[coord.vichwa];
      const isOwnSide = view.active === coord.player;
      const legal = isOwnSide && legalKeys.has(`${coord.player}:${coord.vichwa}`);
      const nyumba = isNyumba(view, coord);
      const blocked = isKutakatiaBlocked(view, coord);
      drawPit(ctx, x, y, count, { legal, nyumba, blocked, owned: isOwnSide });
    }
  }
}

function drawGhala(
  ctx: CanvasRenderingContext2D,
  x: number,
  label: string,
  count: number,
  active: boolean,
) {
  const y = PADDING;
  const h = BOARD_INNER_HEIGHT;
  ctx.fillStyle = active ? "#5d4a36" : "#4a3826";
  ctx.strokeStyle = active ? "#d4b886" : "#6b5440";
  ctx.lineWidth = 2;
  roundedRect(ctx, x, y, GHALA_WIDTH, h, 10);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = "#e8d3a8";
  ctx.font = "14px system-ui, sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(label, x + GHALA_WIDTH / 2, y + 16);
  ctx.font = "bold 24px system-ui, sans-serif";
  ctx.fillText(String(count), x + GHALA_WIDTH / 2, y + h / 2);
  ctx.font = "12px system-ui, sans-serif";
  ctx.fillText("ghala", x + GHALA_WIDTH / 2, y + h - 16);
}

type PitStyle = {
  legal: boolean;
  nyumba: boolean;
  blocked: boolean;
  owned: boolean;
};

function drawPit(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  count: number,
  style: PitStyle,
) {
  let fill = "#4a3826";
  if (style.blocked) fill = "#5b2b2b";
  else if (style.nyumba) fill = "#5a4226";
  else if (style.owned) fill = "#54402c";

  ctx.fillStyle = fill;
  ctx.strokeStyle = style.legal ? "#facc15" : style.nyumba ? "#d4a574" : "#6b5440";
  ctx.lineWidth = style.legal ? 3 : 2;
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

export type { PitCoord, Direction };
