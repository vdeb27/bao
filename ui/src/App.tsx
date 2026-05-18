import { useEffect, useState } from "react";
import { AdvantageBar } from "./components/AdvantageBar";
import { Board, type DirectionPick } from "./components/Board";
import { MoveHistory } from "./components/MoveHistory";
import { StatusBar } from "./components/StatusBar";
import { SubstatePrompt } from "./components/SubstatePrompt";
import {
  engineVersion,
  initEngine,
  moveCategory,
  substate,
  substateTag,
  type Move,
} from "./engine";
import { useAIPlayer } from "./hooks/useAIPlayer";
import { useAnimationDriver } from "./hooks/useAnimationDriver";
import { useOrientation } from "./hooks/useOrientation";
import { useT } from "./i18n";
import { readPersistedState, shareUrl } from "./persistence";
import { useGameStore } from "./store/gameStore";
import "./styles/app.css";

export function App() {
  const [engineReady, setEngineReady] = useState(false);
  const [ambiguous, setAmbiguous] = useState<DirectionPick | null>(null);
  const t = useT();
  const orientation = useOrientation();
  const {
    state,
    view,
    display,
    moves,
    focus,
    pending,
    history,
    historyIndex,
    announcement,
    error,
    startNew,
    play,
    jumpTo,
  } = useGameStore();

  useAnimationDriver();
  const thinking = useAIPlayer(6, 400);

  // Auto-resolve kichwa selections when there's only one legal option.
  // The substate-prompt UI is suppressed in that case; the player doesn't
  // need to be asked when there's no actual choice (e.g. when the capture
  // happened at a kimbi pit, RULES.md §6.3 dictates the side).
  useEffect(() => {
    if (!view || pending) return;
    if (view.winner !== null) return;
    if (substateTag(substate(view.phase)) !== "AwaitKichwa") return;
    const kichwas = moves.filter((m) => moveCategory(m) === "Kichwa");
    if (kichwas.length === 1) {
      play(kichwas[0]);
    }
  }, [view, moves, pending, play]);

  useEffect(() => {
    initEngine()
      .then(() => {
        setEngineReady(true);
        const persisted = readPersistedState();
        if (persisted) {
          useGameStore.getState().hydrate(persisted);
        } else {
          useGameStore.getState().startNew("Kiswahili");
        }
      })
      .catch((e) => console.error("engine init failed", e));
  }, []);

  if (!engineReady || !state || !view || !display) {
    return (
      <main className="bao-loading">
        <p>{t("loading")}</p>
      </main>
    );
  }

  const animating = pending !== null;

  const handlePlay = (m: Move) => {
    setAmbiguous(null);
    play(m);
  };

  return (
    <main className={`bao-app bao-orientation-${orientation}`}>
      <StatusBar
        view={display}
        error={error}
        thinking={thinking}
        shareUrl={() => shareUrl(state)}
        onNewGame={(v) => {
          setAmbiguous(null);
          startNew(v);
        }}
      />
      <AdvantageBar view={display} />
      <div className="bao-board-wrap">
        <div className="bao-board-row">
          <Board
            view={display}
            moves={moves}
            focus={focus}
            animating={animating}
            orientation={orientation}
            onPlay={handlePlay}
            onAmbiguous={setAmbiguous}
          />
          <MoveHistory
            history={history}
            historyIndex={historyIndex}
            onJumpTo={jumpTo}
          />
        </div>
        {!animating && (
          <SubstatePrompt view={view} moves={moves} onPlay={handlePlay} />
        )}
        {!animating && ambiguous && (
          <div className="bao-prompt" role="dialog" aria-label={t("cancel")}>
            <span className="bao-prompt-label">
              {t("directionPrompt", {
                pit: ambiguous.pit.vichwa,
                player: ambiguous.pit.player === 0 ? t("south") : t("north"),
              })}
            </span>
            {ambiguous.candidates.map((m, i) => {
              const dir =
                "Namu" in m
                  ? m.Namu.dir
                  : "Mtaji" in m
                    ? m.Mtaji.dir
                    : "Cw";
              return (
                <button
                  key={i}
                  className="bao-prompt-button"
                  onClick={() => handlePlay(m)}
                >
                  {dir === "Cw" ? t("directionCw") : t("directionCcw")}
                </button>
              );
            })}
            <button
              className="bao-prompt-button bao-prompt-cancel"
              onClick={() => setAmbiguous(null)}
            >
              {t("cancel")}
            </button>
          </div>
        )}
      </div>
      <div role="status" aria-live="polite" className="bao-sr-only">
        {announcement}
      </div>
      <footer className="bao-footer">
        {t("engineVersion", { ver: engineVersion() })}
      </footer>
    </main>
  );
}
