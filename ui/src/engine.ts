// Typed wrapper around the wasm-bindgen surface. The engine is the source of
// truth for game state and rules; here we only translate JSON / packed bytes
// into shapes the UI components consume.

import init, {
  apply as wasmApply,
  engine_version,
  legal_moves as wasmLegalMoves,
  new_state as wasmNewState,
  state_to_json as wasmStateToJson,
  zobrist as wasmZobrist,
} from "@bao/engine";

export type Direction = "Cw" | "Ccw";
export type KichwaSide = "Left" | "Right";
export type Variant = "Kiswahili" | "Kujifunza";

export type Move =
  | { Namu: { col: number; dir: Direction } }
  | { Mtaji: { pit: number; dir: Direction } }
  | { Kichwa: KichwaSide }
  | { Safari: { go: boolean } };

export type Substate =
  | "AwaitMove"
  | { AwaitKichwa: { capture_field: number; prior_dir: Direction | null } }
  | { AwaitSafari: { sow_dir: Direction } };

export type Phase = { Namu: Substate } | { Mtaji: Substate };

export type Side = {
  vichwa: number[];
  ghala: number;
  nyumba_owned: boolean;
  nyumba_col: number;
};

export type BoardState = {
  sides: [Side, Side];
  phase: Phase;
  active: number;
  ply: number;
  variant: Variant;
  kutakatia: {
    blocked_field: number;
    blocked_player: number;
    turns_remaining: number;
  } | null;
  winner: number | null;
};

export type MoveEvent =
  | { NamuPlace: { player: number; pit: number } }
  | { Sow: { player: number; pit: number } }
  | { Pickup: { player: number; pit: number; count: number } }
  | { Capture: { from_player: number; from_pit: number; count: number } }
  | { Tax: { player: number; pit: number; taken: number } }
  | { SafariTriggered: { player: number } }
  | { KichwaSelectionRequired: { player: number; capture_field: number } }
  | "PhaseShift"
  | { NyumbaDestroyed: { player: number } }
  | { KutakatiaActivated: { blocked_player: number; blocked_field: number } }
  | "KutakatiaCleared"
  | { GameOver: { winner: number } };

/** Identifies a single pit on the board: which side it belongs to and the
 * vichwa-array index inside that side. Returned by the animator so callers
 * can briefly highlight the affected pit. */
export type PitFocus = { player: number; vichwa: number };

/** Mutates `display` in place with the visual effect of one engine event and
 * returns the pit that should flash. Phase/active/winner are NOT touched here
 * — those snap atomically when the animation completes. */
export function applyEventToDisplay(display: BoardState, ev: MoveEvent): PitFocus | null {
  if (ev === "PhaseShift" || ev === "KutakatiaCleared") {
    if (ev === "KutakatiaCleared") display.kutakatia = null;
    return null;
  }
  if ("NamuPlace" in ev) {
    const { player, pit } = ev.NamuPlace;
    display.sides[player].ghala = Math.max(0, display.sides[player].ghala - 1);
    display.sides[player].vichwa[pit] = (display.sides[player].vichwa[pit] ?? 0) + 1;
    return { player, vichwa: pit };
  }
  if ("Sow" in ev) {
    const { player, pit } = ev.Sow;
    display.sides[player].vichwa[pit] = (display.sides[player].vichwa[pit] ?? 0) + 1;
    return { player, vichwa: pit };
  }
  if ("Pickup" in ev) {
    const { player, pit } = ev.Pickup;
    display.sides[player].vichwa[pit] = 0;
    return { player, vichwa: pit };
  }
  if ("Capture" in ev) {
    const { from_player, from_pit } = ev.Capture;
    display.sides[from_player].vichwa[from_pit] = 0;
    return { player: from_player, vichwa: from_pit };
  }
  if ("Tax" in ev) {
    const { player, pit, taken } = ev.Tax;
    display.sides[player].vichwa[pit] = Math.max(
      0,
      (display.sides[player].vichwa[pit] ?? 0) - taken,
    );
    return { player, vichwa: pit };
  }
  if ("NyumbaDestroyed" in ev) {
    display.sides[ev.NyumbaDestroyed.player].nyumba_owned = false;
    return { player: ev.NyumbaDestroyed.player, vichwa: display.sides[ev.NyumbaDestroyed.player].nyumba_col };
  }
  if ("KutakatiaActivated" in ev) {
    display.kutakatia = {
      blocked_player: ev.KutakatiaActivated.blocked_player,
      blocked_field: ev.KutakatiaActivated.blocked_field,
      turns_remaining: 3,
    };
    return { player: ev.KutakatiaActivated.blocked_player, vichwa: ev.KutakatiaActivated.blocked_field };
  }
  return null;
}

export function cloneState(s: BoardState): BoardState {
  return {
    ...s,
    sides: [
      { ...s.sides[0], vichwa: [...s.sides[0].vichwa] },
      { ...s.sides[1], vichwa: [...s.sides[1].vichwa] },
    ],
    kutakatia: s.kutakatia ? { ...s.kutakatia } : null,
  };
}

let ready: Promise<void> | null = null;

export function initEngine(): Promise<void> {
  ready ??= init().then(() => undefined);
  return ready;
}

export function engineVersion(): string {
  return engine_version();
}

export function newState(variant: Variant): Uint8Array {
  return wasmNewState(variant.toLowerCase());
}

export function legalMoves(state: Uint8Array): Move[] {
  return JSON.parse(wasmLegalMoves(state)) as Move[];
}

export function applyMove(
  state: Uint8Array,
  move: Move,
): { state: Uint8Array; events: MoveEvent[]; ban: string } {
  const result = wasmApply(state, JSON.stringify(move)) as {
    state: Uint8Array;
    events: string;
    ban: string;
  };
  return {
    state: result.state,
    events: JSON.parse(result.events) as MoveEvent[],
    ban: result.ban,
  };
}

export function stateToJson(state: Uint8Array): BoardState {
  return JSON.parse(wasmStateToJson(state)) as BoardState;
}

export function zobrist(state: Uint8Array): bigint {
  return wasmZobrist(state);
}

export function phaseTag(phase: Phase): "Namu" | "Mtaji" {
  return "Namu" in phase ? "Namu" : "Mtaji";
}

export function substate(phase: Phase): Substate {
  return "Namu" in phase ? phase.Namu : phase.Mtaji;
}

export function substateTag(s: Substate): "AwaitMove" | "AwaitKichwa" | "AwaitSafari" {
  if (s === "AwaitMove") return "AwaitMove";
  return "AwaitKichwa" in s ? "AwaitKichwa" : "AwaitSafari";
}

export function moveCategory(
  m: Move,
): "Namu" | "Mtaji" | "Kichwa" | "Safari" {
  if ("Namu" in m) return "Namu";
  if ("Mtaji" in m) return "Mtaji";
  if ("Kichwa" in m) return "Kichwa";
  return "Safari";
}
