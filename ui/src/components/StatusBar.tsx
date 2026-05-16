import { phaseTag, substate, substateTag, type BoardState, type Variant } from "../engine";

type Props = {
  view: BoardState;
  error: string | null;
  onNewGame: (variant: Variant) => void;
};

const PLAYER_NAMES = ["South", "North"];

export function StatusBar({ view, error, onNewGame }: Props) {
  const phaseName = phaseTag(view.phase);
  const subTag = substateTag(substate(view.phase));
  const winner = view.winner !== null ? PLAYER_NAMES[view.winner] : null;

  return (
    <header className="bao-statusbar">
      <div className="bao-statusbar-left">
        <h1>Bao</h1>
        <span className="bao-variant-badge">{view.variant}</span>
      </div>
      <div className="bao-statusbar-mid">
        {winner ? (
          <strong className="bao-winner">{winner} wint!</strong>
        ) : (
          <>
            <span>
              Aan zet: <strong>{PLAYER_NAMES[view.active]}</strong>
            </span>
            <span>
              Fase: <strong>{phaseName}</strong>
              {subTag !== "AwaitMove" && <em> · {subTag}</em>}
            </span>
            <span>Ply: {view.ply}</span>
          </>
        )}
      </div>
      <div className="bao-statusbar-right">
        <button onClick={() => onNewGame("Kiswahili")}>Nieuw Kiswahili</button>
        <button onClick={() => onNewGame("Kujifunza")}>Nieuw Kujifunza</button>
      </div>
      {error && <div className="bao-error">{error}</div>}
    </header>
  );
}
