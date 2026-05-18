import { phaseTag, substate, substateTag, type BoardState, type Variant } from "../engine";
import { LOCALES, useI18n, useT, type Locale } from "../i18n";
import { useSound } from "../sound";
import { useOpponentStore, type OpponentKind } from "../store/opponentStore";

type Props = {
  view: BoardState;
  error: string | null;
  thinking: boolean;
  shareUrl: () => string;
  onNewGame: (variant: Variant) => void;
};

const PLAYER_KEYS = ["south", "north"];

export function StatusBar({ view, error, thinking, shareUrl, onNewGame }: Props) {
  const t = useT();
  const locale = useI18n((s) => s.locale);
  const setLocale = useI18n((s) => s.setLocale);
  const soundEnabled = useSound((s) => s.enabled);
  const toggleSound = useSound((s) => s.toggle);
  const opponentSouth = useOpponentStore((s) => s.south);
  const opponentNorth = useOpponentStore((s) => s.north);
  const setOpponent = useOpponentStore((s) => s.setOpponent);

  const phaseName = phaseTag(view.phase);
  const subTag = substateTag(substate(view.phase));
  const winner = view.winner !== null ? t(PLAYER_KEYS[view.winner]) : null;

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(shareUrl());
    } catch {
      /* ignore — best-effort */
    }
  };

  return (
    <header className="bao-statusbar">
      <div className="bao-statusbar-left">
        <h1>Bao</h1>
        <span className="bao-variant-badge">{view.variant}</span>
      </div>
      <div className="bao-statusbar-mid">
        {winner ? (
          <strong className="bao-winner">{t("winsBanner", { player: winner })}</strong>
        ) : (
          <>
            <span>
              {t("toMove")}: <strong>{t(PLAYER_KEYS[view.active])}</strong>
              {thinking && <em className="bao-thinking"> · {t("thinking")}</em>}
            </span>
            <span>
              {t("phase")}: <strong>{phaseName}</strong>
              {subTag !== "AwaitMove" && <em> · {subTag}</em>}
            </span>
            <span>
              {t("ply")}: {view.ply}
            </span>
          </>
        )}
      </div>
      <div className="bao-opponents">
        <label>
          <span>{t("opponentSouth")}</span>
          <select
            value={opponentSouth}
            onChange={(e) => setOpponent(0, e.target.value as OpponentKind)}
          >
            <option value="human">{t("opponentHuman")}</option>
            <option value="jifunzo">{t("opponentJifunzo")}</option>
          </select>
        </label>
        <label>
          <span>{t("opponentNorth")}</span>
          <select
            value={opponentNorth}
            onChange={(e) => setOpponent(1, e.target.value as OpponentKind)}
          >
            <option value="human">{t("opponentHuman")}</option>
            <option value="jifunzo">{t("opponentJifunzo")}</option>
          </select>
        </label>
      </div>
      <div className="bao-statusbar-right">
        <button onClick={() => onNewGame("Kiswahili")}>{t("newKiswahili")}</button>
        <button onClick={() => onNewGame("Kujifunza")}>{t("newKujifunza")}</button>
        <button onClick={copy} aria-label={t("share")} title={t("share")}>
          🔗
        </button>
        <button
          onClick={toggleSound}
          aria-pressed={soundEnabled}
          title={soundEnabled ? t("soundOn") : t("soundOff")}
        >
          {soundEnabled ? "🔊" : "🔇"}
        </button>
        <label className="bao-locale-toggle" aria-label={t("locale")}>
          <select
            value={locale}
            onChange={(e) => setLocale(e.target.value as Locale)}
          >
            {LOCALES.map((l) => (
              <option key={l} value={l}>
                {l.toUpperCase()}
              </option>
            ))}
          </select>
        </label>
      </div>
      {error && <div className="bao-error">{error}</div>}
    </header>
  );
}
