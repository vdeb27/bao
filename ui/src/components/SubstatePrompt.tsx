import {
  moveCategory,
  substate,
  substateTag,
  type BoardState,
  type Move,
} from "../engine";

type Props = {
  view: BoardState;
  moves: Move[];
  onPlay: (move: Move) => void;
};

export function SubstatePrompt({ view, moves, onPlay }: Props) {
  const tag = substateTag(substate(view.phase));
  if (tag === "AwaitMove") return null;

  if (tag === "AwaitKichwa") {
    const options = moves.filter((m) => moveCategory(m) === "Kichwa");
    return (
      <div className="bao-prompt" role="dialog" aria-label="Kies kichwa">
        <span className="bao-prompt-label">Kies kichwa voor capture-sow:</span>
        {options.map((m) => (
          <button
            key={(m as { Kichwa: string }).Kichwa}
            onClick={() => onPlay(m)}
            className="bao-prompt-button"
          >
            {(m as { Kichwa: string }).Kichwa === "Left" ? "← Links" : "Rechts →"}
          </button>
        ))}
      </div>
    );
  }

  // AwaitSafari
  const options = moves.filter((m) => moveCategory(m) === "Safari");
  return (
    <div className="bao-prompt" role="dialog" aria-label="Safari beslissing">
      <span className="bao-prompt-label">Safari — eigen nyumba leegmaken?</span>
      {options.map((m) => {
        const go = (m as { Safari: { go: boolean } }).Safari.go;
        return (
          <button
            key={String(go)}
            onClick={() => onPlay(m)}
            className="bao-prompt-button"
          >
            {go ? "Plunder & ga door" : "Stop"}
          </button>
        );
      })}
    </div>
  );
}
