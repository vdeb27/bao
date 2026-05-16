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

export type MoveEvent = unknown; // events flow through but the UI ignores them in this slice

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
): { state: Uint8Array; events: MoveEvent[] } {
  const result = wasmApply(state, JSON.stringify(move)) as {
    state: Uint8Array;
    events: string;
  };
  return { state: result.state, events: JSON.parse(result.events) as MoveEvent[] };
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
