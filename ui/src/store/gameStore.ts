import { create } from "zustand";
import {
  applyEventToDisplay,
  applyMove,
  cloneState,
  legalMoves,
  newState,
  stateToJson,
  type BoardState,
  type Move,
  type MoveEvent,
  type PitFocus,
  type Variant,
} from "../engine";
import { persistState } from "../persistence";
import { useSound } from "../sound";

export type HistoryEntry = {
  ply: number;
  player: number;
  ban: string;
};

type GameStore = {
  state: Uint8Array | null;
  view: BoardState | null;
  display: BoardState | null;
  moves: Move[];
  pending: { events: MoveEvent[]; cursor: number } | null;
  focus: PitFocus | null;
  history: HistoryEntry[];
  /** Live aria-live announcement string. Updated on every state-changing
   * event so screen readers hear "Capture op pit f1" etc. */
  announcement: string;
  error: string | null;
  startNew: (variant: Variant) => void;
  hydrate: (bytes: Uint8Array) => void;
  play: (move: Move) => void;
  advance: () => void;
};

function snapshot(state: Uint8Array): { state: Uint8Array; view: BoardState; moves: Move[] } {
  const view = stateToJson(state);
  return { state, view, moves: legalMoves(state) };
}

function announceForEvent(ev: MoveEvent): string {
  if (typeof ev === "string") {
    if (ev === "PhaseShift") return "Phase shift";
    if (ev === "KutakatiaCleared") return "Kutakatia cleared";
    return "";
  }
  if ("Capture" in ev)
    return `Capture pit ${ev.Capture.from_pit} (${ev.Capture.count} kete)`;
  if ("GameOver" in ev)
    return ev.GameOver.winner === 0 ? "South wins" : "North wins";
  if ("KutakatiaActivated" in ev)
    return `Kutakatia at pit ${ev.KutakatiaActivated.blocked_field}`;
  if ("NyumbaDestroyed" in ev) return "Nyumba destroyed";
  return "";
}

export const useGameStore = create<GameStore>((set, get) => ({
  state: null,
  view: null,
  display: null,
  moves: [],
  pending: null,
  focus: null,
  history: [],
  announcement: "",
  error: null,
  startNew: (variant) => {
    const s = newState(variant);
    const snap = snapshot(s);
    set({
      ...snap,
      display: cloneState(snap.view),
      pending: null,
      focus: null,
      history: [],
      announcement: "",
      error: null,
    });
    persistState(s);
  },
  hydrate: (bytes) => {
    try {
      const snap = snapshot(bytes);
      set({
        ...snap,
        display: cloneState(snap.view),
        pending: null,
        focus: null,
        history: [],
        announcement: "",
        error: null,
      });
    } catch (e) {
      // Bad/old hash — fall back to a fresh Kiswahili game.
      set({ error: e instanceof Error ? e.message : String(e) });
      get().startNew("Kiswahili");
    }
  },
  play: (move) => {
    const current = get().state;
    const currentView = get().view;
    const currentDisplay = get().display;
    if (!current || !currentView || !currentDisplay) return;
    if (get().pending) return;
    try {
      const { state: next, events, ban } = applyMove(current, move);
      const snap = snapshot(next);
      const entry: HistoryEntry = {
        ply: currentView.ply,
        player: currentView.active,
        ban,
      };
      set({
        ...snap,
        display: currentDisplay,
        pending: events.length > 0 ? { events, cursor: 0 } : null,
        focus: null,
        history: [...get().history, entry],
        error: null,
      });
      persistState(next);
      if (events.length === 0) {
        set({ display: cloneState(snap.view) });
      }
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
      useSound.getState().play("error");
    }
  },
  advance: () => {
    const { pending, display, view } = get();
    if (!pending || !display || !view) return;
    if (pending.cursor >= pending.events.length) {
      set({ pending: null, focus: null, display: cloneState(view) });
      return;
    }
    const ev = pending.events[pending.cursor];
    const next = cloneState(display);
    const focus = applyEventToDisplay(next, ev);
    const newCursor = pending.cursor + 1;
    // Sound + announcement triggers.
    const sound = useSound.getState();
    if (typeof ev !== "string") {
      if ("Sow" in ev || "NamuPlace" in ev) sound.play("sow");
      else if ("Capture" in ev) sound.play("capture");
      else if ("GameOver" in ev) sound.play("win");
    }
    const ann = announceForEvent(ev);
    if (newCursor >= pending.events.length) {
      set({
        display: cloneState(view),
        pending: null,
        focus,
        announcement: ann || get().announcement,
      });
    } else {
      set({
        display: next,
        pending: { events: pending.events, cursor: newCursor },
        focus,
        announcement: ann || get().announcement,
      });
    }
  },
}));
