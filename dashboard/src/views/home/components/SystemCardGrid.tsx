/**
 * Grille des six cartes de système, cliquables vers leur page-rapport.
 */

// Importation des modules framework
import { Link } from "react-router-dom";

// Importation des modules internes
import { SYSTEM_COLORS } from "../../../content/system_colors";
import { SYSTEMS_CONTENT } from "../../../content/systems_content";
import { formatPercent } from "../../../format/number_format";
import type { MetricsRow, SystemDefinition } from "../../../types/data";

interface Props {
  systems: SystemDefinition[];
  metrics: MetricsRow[];
  query: string;
}

export default function SystemCardGrid({ systems, metrics, query }: Props) {
  return (
    <div className="grid-3">
      {systems.map((s) => {
        const m = metrics.find((row) => row.system === s.id);
        const traits = [
          `Niveau ${s.level}`,
          s.balanced ? "équilibré" : "non équilibré",
          ...(s.ace_side_count ? ["side-count des As"] : []),
        ];
        return (
          <Link key={s.id} to={`/systeme/${s.id}${query}`} className="card system-card">
            <span className="system-card__head">
              <span className="swatch" style={{ background: SYSTEM_COLORS[s.id], width: 14, height: 14 }} aria-hidden="true" />
              <span className="system-card__name">{s.name}</span>
            </span>
            <span className="system-card__meta">{traits.join(" · ")}</span>
            <span className="muted" style={{ fontSize: 14 }}>
              {SYSTEMS_CONTENT[s.id].tagline}
            </span>
            <span className="system-card__value">
              {m ? formatPercent(m.advantage_pct, 2, true) : "—"}{" "}
              <span className="system-card__value-label">d&apos;avantage</span>
            </span>
          </Link>
        );
      })}
    </div>
  );
}
