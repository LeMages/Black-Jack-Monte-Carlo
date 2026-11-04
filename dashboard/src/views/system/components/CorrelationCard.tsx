/**
 * Carte « Corrélations » : BC, PE et IC publiés, en barres proportionnelles.
 */

// Importation des modules internes
import Card from "../../../components/Card";
import { formatNumber } from "../../../format/number_format";
import type { SystemDefinition } from "../../../types/data";

export default function CorrelationCard({ system }: { system: SystemDefinition }) {
  const bars = [
    { key: "BC", label: "Betting correlation", value: system.betting_correlation },
    { key: "PE", label: "Playing efficiency", value: system.playing_efficiency },
    { key: "IC", label: "Insurance correlation", value: system.insurance_correlation },
  ];
  return (
    <Card title="Corrélations">
      {bars.map((bar) => (
        <div key={bar.key} className="bar-row" title={bar.label}>
          <span className="bar-row__label">{bar.key}</span>
          <span
            className="bar-row__track"
            role="meter"
            aria-label={bar.label}
            aria-valuemin={0}
            aria-valuemax={1}
            aria-valuenow={bar.value}
          >
            <span className="bar-row__fill" style={{ display: "block", width: `${bar.value * 100}%` }} />
          </span>
          <span className="bar-row__value">{formatNumber(bar.value, 2)}</span>
        </div>
      ))}
      <p className="bar-note">
        BC : qualité du signal de mise. PE : qualité des décisions de jeu. IC : qualité de la décision
        d&apos;assurance. Valeurs publiées pour chaque système.
      </p>
    </Card>
  );
}
