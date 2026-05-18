import { create } from "zustand";

export type OpponentKind = "human" | "jifunzo";

type OpponentStore = {
  south: OpponentKind;
  north: OpponentKind;
  setOpponent: (player: 0 | 1, kind: OpponentKind) => void;
};

const STORAGE_KEY = "bao.opponents";

function load(): { south: OpponentKind; north: OpponentKind } {
  if (typeof window === "undefined") return { south: "human", north: "human" };
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as { south?: OpponentKind; north?: OpponentKind };
      return {
        south: parsed.south ?? "human",
        north: parsed.north ?? "human",
      };
    }
  } catch {
    /* ignore */
  }
  return { south: "human", north: "human" };
}

export const useOpponentStore = create<OpponentStore>((set, get) => ({
  ...load(),
  setOpponent: (player, kind) => {
    const next = { south: get().south, north: get().north };
    if (player === 0) next.south = kind;
    else next.north = kind;
    set(next);
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      /* ignore */
    }
  },
}));
