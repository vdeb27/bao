import { useEffect, useRef } from "react";
import type { HistoryEntry } from "../store/gameStore";

const PLAYER_LABEL = ["S", "N"]; // South / North short labels

type Props = {
  history: HistoryEntry[];
};

/** Renders the move log as a vertical scroll-list. Each row shows the
 * one-based action index, who moved, and the BAN string. Sub-moves (kichwa
 * and safari) are grouped under the same parent ply via a faint indent. */
export function MoveHistory({ history }: Props) {
  const listRef = useRef<HTMLOListElement | null>(null);

  // Auto-scroll to the latest entry whenever the log grows.
  useEffect(() => {
    const el = listRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [history.length]);

  return (
    <aside className="bao-history" aria-label="Zettenlijst (BAN)">
      <header className="bao-history-header">
        <span>Zetten</span>
        <span className="bao-history-count">{history.length}</span>
      </header>
      <ol className="bao-history-list" ref={listRef}>
        {history.length === 0 ? (
          <li className="bao-history-empty">— nog geen zetten —</li>
        ) : (
          history.map((entry, i) => {
            const isSubmove = entry.ban.startsWith("K:") || entry.ban.startsWith("S");
            return (
              <li
                key={i}
                className={`bao-history-entry${isSubmove ? " bao-history-sub" : ""}`}
              >
                <span className="bao-history-idx">{i + 1}.</span>
                <span
                  className={`bao-history-player bao-history-player-${entry.player}`}
                >
                  {PLAYER_LABEL[entry.player]}
                </span>
                <code className="bao-history-ban">{entry.ban}</code>
              </li>
            );
          })
        )}
      </ol>
    </aside>
  );
}
