import type { BoardState } from "../engine";
import { useT } from "../i18n";

type Props = {
  view: BoardState;
};

function tallies(view: BoardState) {
  return view.sides.map((s) => {
    let total = s.ghala;
    let mbele = 0;
    for (let i = 0; i < 16; i++) {
      total += s.vichwa[i];
      if (i < 8) mbele += s.vichwa[i];
    }
    return { total, mbele };
  });
}

export function AdvantageBar({ view }: Props) {
  const t = useT();
  const [south, north] = tallies(view);
  const grandTotal = south.total + north.total;
  const southPct = grandTotal > 0 ? (south.total / grandTotal) * 100 : 50;
  const mbeleTotal = south.mbele + north.mbele;
  const southMbelePct = mbeleTotal > 0 ? (south.mbele / mbeleTotal) * 100 : 50;

  return (
    <div className="bao-advantage" aria-label={t("materialBalance")}>
      <div className="bao-advantage-row">
        <span className="bao-advantage-count bao-advantage-count-south">{south.total}</span>
        <div
          className="bao-advantage-track"
          role="meter"
          aria-valuemin={0}
          aria-valuemax={64}
          aria-valuenow={south.total}
        >
          <div className="bao-advantage-fill-south" style={{ width: `${southPct}%` }} />
          <div className="bao-advantage-fill-north" style={{ width: `${100 - southPct}%` }} />
        </div>
        <span className="bao-advantage-count bao-advantage-count-north">{north.total}</span>
      </div>
      <div className="bao-advantage-sub">
        <span className="bao-advantage-label">
          {t("mbeleLabel", { south: south.mbele, north: north.mbele })}
        </span>
        <div className="bao-advantage-track bao-advantage-track-mbele">
          <div className="bao-advantage-fill-south-mbele" style={{ width: `${southMbelePct}%` }} />
          <div className="bao-advantage-fill-north-mbele" style={{ width: `${100 - southMbelePct}%` }} />
        </div>
      </div>
    </div>
  );
}
