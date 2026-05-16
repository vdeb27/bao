import { useEffect, useRef } from "react";
import { useT } from "../i18n";
import type { HistoryEntry } from "../store/gameStore";

const PLAYER_LABEL = ["S", "N"];

type Props = {
  history: HistoryEntry[];
  /** Pointer into positions[]. `historyIndex == i+1` means the state after
   * move i is currently displayed. */
  historyIndex: number;
  onJumpTo: (index: number) => void;
};

export function MoveHistory({ history, historyIndex, onJumpTo }: Props) {
  const t = useT();
  const listRef = useRef<HTMLOListElement | null>(null);

  useEffect(() => {
    const el = listRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [history.length, historyIndex]);

  return (
    <aside className="bao-history" aria-label={t("moves")}>
      <header className="bao-history-header">
        <span>{t("moves")}</span>
        <span className="bao-history-count">{history.length}</span>
      </header>
      <ol className="bao-history-list" ref={listRef}>
        <li
          className={`bao-history-entry bao-history-start${
            historyIndex === 0 ? " bao-history-current" : ""
          }${historyIndex > 0 ? "" : ""}`}
          onDoubleClick={() => onJumpTo(0)}
          title="Dubbelklik om naar de startpositie te springen"
        >
          <span className="bao-history-idx">0.</span>
          <span className="bao-history-player">·</span>
          <code className="bao-history-ban">(start)</code>
        </li>
        {history.length === 0 ? (
          <li className="bao-history-empty">{t("noMoves")}</li>
        ) : (
          history.map((entry, i) => {
            const isSubmove =
              entry.ban.startsWith("K:") || entry.ban.startsWith("S");
            const positionIdx = i + 1;
            const isCurrent = positionIdx === historyIndex;
            const isFuture = positionIdx > historyIndex;
            const cls = [
              "bao-history-entry",
              isSubmove ? "bao-history-sub" : "",
              isCurrent ? "bao-history-current" : "",
              isFuture ? "bao-history-future" : "",
            ]
              .filter(Boolean)
              .join(" ");
            return (
              <li
                key={i}
                className={cls}
                onDoubleClick={() => onJumpTo(positionIdx)}
                title="Dubbelklik om naar deze positie te springen"
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
