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

type GameStore = {
  /** Authoritative engine packed bytes. */
  state: Uint8Array | null;
  /** Authoritative engine view — the final state of the most recent move. */
  view: BoardState | null;
  /** Animation-paced visualisation. Equals `view` when idle, lags behind during
   * event playback. */
  display: BoardState | null;
  /** Legal moves for the *authoritative* view (always reflects what the next
   * input should produce). */
  moves: Move[];
  /** Remaining unplayed events, plus pre-move snapshot for the substate-prompt
   * gate. Null when idle. */
  pending: { events: MoveEvent[]; cursor: number } | null;
  /** Pit to flash on the current animation step. */
  focus: PitFocus | null;
  error: string | null;
  startNew: (variant: Variant) => void;
  play: (move: Move) => void;
  /** Advance the animation by exactly one event. The animation hook calls this
   * on a fixed timer; when the queue empties, display snaps to `view`. */
  advance: () => void;
};

function snapshot(state: Uint8Array): { state: Uint8Array; view: BoardState; moves: Move[] } {
  const view = stateToJson(state);
  return { state, view, moves: legalMoves(state) };
}

export const useGameStore = create<GameStore>((set, get) => ({
  state: null,
  view: null,
  display: null,
  moves: [],
  pending: null,
  focus: null,
  error: null,
  startNew: (variant) => {
    const s = newState(variant);
    const snap = snapshot(s);
    set({
      ...snap,
      display: cloneState(snap.view),
      pending: null,
      focus: null,
      error: null,
    });
  },
  play: (move) => {
    const current = get().state;
    const currentDisplay = get().display;
    if (!current || !currentDisplay) return;
    if (get().pending) return; // ignore clicks while animating
    try {
      const { state: next, events } = applyMove(current, move);
      const snap = snapshot(next);
      set({
        ...snap,
        // display stays where it was — events will move it forward.
        display: currentDisplay,
        pending: events.length > 0 ? { events, cursor: 0 } : null,
        focus: null,
        error: null,
      });
      // If no events, snap display immediately.
      if (events.length === 0) {
        set({ display: cloneState(snap.view) });
      }
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },
  advance: () => {
    const { pending, display, view } = get();
    if (!pending || !display || !view) return;
    if (pending.cursor >= pending.events.length) {
      set({ pending: null, focus: null, display: cloneState(view) });
      return;
    }
    const next = cloneState(display);
    const focus = applyEventToDisplay(next, pending.events[pending.cursor]);
    const newCursor = pending.cursor + 1;
    if (newCursor >= pending.events.length) {
      // Last event applied — snap to authoritative view to pick up phase /
      // active / winner changes that aren't carried by event mutations.
      set({
        display: cloneState(view),
        pending: null,
        focus,
      });
    } else {
      set({
        display: next,
        pending: { events: pending.events, cursor: newCursor },
        focus,
      });
    }
  },
}));
