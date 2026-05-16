import { useEffect, useState } from "react";

export type Orientation = "landscape" | "portrait";

/** Tracks `(orientation: portrait)` via `matchMedia`. Falls back to landscape
 * during SSR / no-window contexts. */
export function useOrientation(): Orientation {
  const [orientation, setOrientation] = useState<Orientation>(() => {
    if (typeof window === "undefined") return "landscape";
    return window.matchMedia("(orientation: portrait) and (max-width: 700px)").matches
      ? "portrait"
      : "landscape";
  });

  useEffect(() => {
    if (typeof window === "undefined") return;
    const mql = window.matchMedia("(orientation: portrait) and (max-width: 700px)");
    const update = () => setOrientation(mql.matches ? "portrait" : "landscape");
    update();
    if (mql.addEventListener) {
      mql.addEventListener("change", update);
      return () => mql.removeEventListener("change", update);
    }
    // Older Safari fallback.
    mql.addListener(update);
    return () => mql.removeListener(update);
  }, []);

  return orientation;
}
