/**
 * Six cartes de métriques du modèle A pour la config choisie.
 */

// Importation des modules internes
import MetricCard from "../../../components/MetricCard";
import {
  formatFractionAsPercent,
  formatInteger,
  formatN0,
  formatNumber,
  formatPercent,
} from "../../../format/number_format";
import type { MetricsRow } from "../../../types/data";

interface Props {
  metrics: MetricsRow;
  /** Bankroll de référence du risque de ruine (unités), lue dans meta.json. */
  rorBankroll: number;
}

export default function SystemMetricsGrid({ metrics, rorBankroll }: Props) {
  return (
    <div className="grid-6">
      <MetricCard label="Avantage" value={formatPercent(metrics.advantage_pct, 2, true)} hint="gain / mises initiales" />
      <MetricCard
        label="EV / 100 mains"
        value={`${formatNumber(metrics.ev_per_100, 2)} u`}
        hint="en unités de mise"
      />
      <MetricCard label="σ / main" value={`${formatNumber(metrics.std_per_hand, 2)} u`} hint="écart-type" />
      <MetricCard label="N₀" value={formatN0(metrics.n0)} hint="mains pour que l'EV égale un écart-type" />
      <MetricCard
        label="Risque de ruine"
        value={formatFractionAsPercent(metrics.risk_of_ruin)}
        hint={`bankroll de référence de ${formatInteger(rorBankroll)} unités`}
      />
      <MetricCard
        label="Fraction Kelly"
        value={formatFractionAsPercent(metrics.kelly_fraction, 3)}
        hint="taille d'unité optimale / bankroll"
      />
    </div>
  );
}
