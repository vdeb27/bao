import { useEffect, useState } from "react";
import { Board, type DirectionPick } from "./components/Board";
import { StatusBar } from "./components/StatusBar";
import { SubstatePrompt } from "./components/SubstatePrompt";
import { engineVersion, initEngine, type Move } from "./engine";
import { useAnimationDriver } from "./hooks/useAnimationDriver";
import { useGameStore } from "./store/gameStore";
import "./styles/app.css";

export function App() {
  const [engineReady, setEngineReady] = useState(false);
  const [ambiguous, setAmbiguous] = useState<DirectionPick | null>(null);
  const { state, view, display, moves, focus, pending, error, startNew, play } =
    useGameStore();

  useAnimationDriver();

  useEffect(() => {
    initEngine()
      .then(() => {
        setEngineReady(true);
        useGameStore.getState().startNew("Kiswahili");
      })
      .catch((e) => console.error("engine init failed", e));
  }, []);

  if (!engineReady || !state || !view || !display) {
    return (
      <main className="bao-loading">
        <p>Engine laden…</p>
      </main>
    );
  }

  const animating = pending !== null;

  const handlePlay = (m: Move) => {
    setAmbiguous(null);
    play(m);
  };

  return (
    <main className="bao-app">
      <StatusBar
        view={display}
        error={error}
        onNewGame={(v) => {
          setAmbiguous(null);
          startNew(v);
        }}
      />
      <div className="bao-board-wrap">
        <Board
          view={display}
          moves={moves}
          focus={focus}
          animating={animating}
          onPlay={handlePlay}
          onAmbiguous={setAmbiguous}
        />
        {!animating && (
          <SubstatePrompt view={view} moves={moves} onPlay={handlePlay} />
        )}
        {!animating && ambiguous && (
          <div className="bao-prompt" role="dialog" aria-label="Kies richting">
            <span className="bao-prompt-label">
              Pit {ambiguous.coord.vichwa} ({ambiguous.coord.player === 0 ? "South" : "North"}) — richting?
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
                  {dir === "Cw" ? "↻ Met de klok mee" : "↺ Tegen de klok in"}
                </button>
              );
            })}
            <button
              className="bao-prompt-button bao-prompt-cancel"
              onClick={() => setAmbiguous(null)}
            >
              Annuleer
            </button>
          </div>
        )}
      </div>
      <footer className="bao-footer">engine v{engineVersion()}</footer>
    </main>
  );
}
