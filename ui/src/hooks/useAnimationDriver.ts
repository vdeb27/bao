import { useEffect } from "react";
import { useGameStore } from "../store/gameStore";

const STEP_MS = 95;

/** Pulls the next pending MoveEvent into the display state on a fixed-interval
 * timer. Mounting once at the app root is enough; the hook subscribes to the
 * store and stops/restarts the timer as `pending` flips. */
export function useAnimationDriver() {
  const hasPending = useGameStore((s) => s.pending !== null);
  useEffect(() => {
    if (!hasPending) return;
    const id = window.setInterval(() => {
      useGameStore.getState().advance();
    }, STEP_MS);
    return () => window.clearInterval(id);
  }, [hasPending]);
}
