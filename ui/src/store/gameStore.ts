import { create } from "zustand";
import {
  applyMove,
  legalMoves,
  newState,
  stateToJson,
  type BoardState,
  type Move,
  type Variant,
} from "../engine";

type GameStore = {
  state: Uint8Array | null;
  view: BoardState | null;
  moves: Move[];
  error: string | null;
  startNew: (variant: Variant) => void;
  play: (move: Move) => void;
};

function refresh(state: Uint8Array): {
  state: Uint8Array;
  view: BoardState;
  moves: Move[];
} {
  return {
    state,
    view: stateToJson(state),
    moves: legalMoves(state),
  };
}

export const useGameStore = create<GameStore>((set, get) => ({
  state: null,
  view: null,
  moves: [],
  error: null,
  startNew: (variant) => {
    const s = newState(variant);
    set({ ...refresh(s), error: null });
  },
  play: (move) => {
    const current = get().state;
    if (!current) return;
    try {
      const { state: next } = applyMove(current, move);
      set({ ...refresh(next), error: null });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },
}));
