import { useEffect, useRef, useState } from "react";
import { searchHeuristic } from "../engine";
import { useGameStore } from "../store/gameStore";
import { useOpponentStore } from "../store/opponentStore";

const AI_THINK_DELAY_MS = 80; // small breathing room so the UI can paint

/** Watches the active side and the opponent configuration; when it's the
 * AI's turn, runs a heuristic search and plays the returned move. Search
 * is synchronous (Bao positions are small enough that a few hundred ms at
 * depth 6 is plenty), so we offset it via setTimeout to avoid blocking
 * the same paint that flipped the turn. */
export function useAIPlayer(maxDepth = 6, timeBudgetMs = 400): boolean {
  const view = useGameStore((s) => s.view);
  const state = useGameStore((s) => s.state);
  const pending = useGameStore((s) => s.pending);
  const play = useGameStore((s) => s.play);
  const south = useOpponentStore((s) => s.south);
  const north = useOpponentStore((s) => s.north);
  const [thinking, setThinking] = useState(false);
  const cancelRef = useRef<number | null>(null);

  useEffect(() => {
    if (!view || !state) {
      setThinking(false);
      return;
    }
    if (pending || view.winner !== null) {
      setThinking(false);
      return;
    }
    const opp = view.active === 0 ? south : north;
    if (opp !== "jifunzo") {
      setThinking(false);
      return;
    }
    setThinking(true);
    cancelRef.current = window.setTimeout(() => {
      const r = searchHeuristic(state, maxDepth, timeBudgetMs);
      if (r.best_move) {
        play(r.best_move);
      }
      setThinking(false);
    }, AI_THINK_DELAY_MS);
    return () => {
      if (cancelRef.current !== null) window.clearTimeout(cancelRef.current);
      cancelRef.current = null;
    };
  }, [view, state, pending, south, north, play, maxDepth, timeBudgetMs]);

  return thinking;
}
