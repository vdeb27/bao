import { useEffect, useRef } from "react";
import { useT } from "../i18n";
import type { HistoryEntry } from "../store/gameStore";

const PLAYER_LABEL = ["S", "N"];

type Props = {
  history: HistoryEntry[];
};

export function MoveHistory({ history }: Props) {
  const t = useT();
  const listRef = useRef<HTMLOListElement | null>(null);

  useEffect(() => {
    const el = listRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [history.length]);

  return (
    <aside className="bao-history" aria-label={t("moves")}>
      <header className="bao-history-header">
        <span>{t("moves")}</span>
        <span className="bao-history-count">{history.length}</span>
      </header>
      <ol className="bao-history-list" ref={listRef}>
        {history.length === 0 ? (
          <li className="bao-history-empty">{t("noMoves")}</li>
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
