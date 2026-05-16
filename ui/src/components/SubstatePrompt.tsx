import {
  moveCategory,
  substate,
  substateTag,
  type BoardState,
  type Move,
} from "../engine";
import { useT } from "../i18n";

type Props = {
  view: BoardState;
  moves: Move[];
  onPlay: (move: Move) => void;
};

export function SubstatePrompt({ view, moves, onPlay }: Props) {
  const t = useT();
  const tag = substateTag(substate(view.phase));
  if (tag === "AwaitMove") return null;

  if (tag === "AwaitKichwa") {
    const options = moves.filter((m) => moveCategory(m) === "Kichwa");
    return (
      <div className="bao-prompt" role="dialog" aria-label={t("chooseKichwa")}>
        <span className="bao-prompt-label">{t("chooseKichwa")}</span>
        {options.map((m) => (
          <button
            key={(m as { Kichwa: string }).Kichwa}
            onClick={() => onPlay(m)}
            className="bao-prompt-button"
          >
            {(m as { Kichwa: string }).Kichwa === "Left"
              ? t("kichwaLeft")
              : t("kichwaRight")}
          </button>
        ))}
      </div>
    );
  }

  const options = moves.filter((m) => moveCategory(m) === "Safari");
  return (
    <div className="bao-prompt" role="dialog" aria-label={t("safariPrompt")}>
      <span className="bao-prompt-label">{t("safariPrompt")}</span>
      {options.map((m) => {
        const go = (m as { Safari: { go: boolean } }).Safari.go;
        return (
          <button
            key={String(go)}
            onClick={() => onPlay(m)}
            className="bao-prompt-button"
          >
            {go ? t("safariYes") : t("safariNo")}
          </button>
        );
      })}
    </div>
  );
}
