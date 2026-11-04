/**
 * Page d'accueil : méthode, indicateurs clés, aperçu risque/rendement, systèmes.
 */

// Importation des modules framework
import { Link } from "react-router-dom";

// Importation des modules internes
import SystemCardGrid from "./components/SystemCardGrid";
import ChartCard from "../../components/ChartCard";
import MetricCard from "../../components/MetricCard";
import RiskReturnScatter from "../../components/RiskReturnScatter";
import { useData } from "../../data/data_context";
import { metricsForConfig } from "../../data/select";
import { formatCompact, formatFractionAsPercent, formatInteger, formatPercent } from "../../format/number_format";
import { configQuery, useSelectedConfig } from "../../hooks/use_selected_config";
import type { MetricsRow } from "../../types/data";

/** Ligne qui maximise un critère (sélection, pas de calcul statistique). */
function pickBest(rows: MetricsRow[], better: (a: MetricsRow, b: MetricsRow) => boolean): MetricsRow | undefined {
  return rows.reduce<MetricsRow | undefined>((best, row) => (best && !better(row, best) ? best : row), undefined);
}

export default function HomeView() {
  const data = useData();
  const [config] = useSelectedConfig();
  const metrics = metricsForConfig(data, config.id);
  const query = configQuery(config.id);
  const nameOf = (id: string) => data.systems.find((s) => s.id === id)?.name ?? id;
  const best = pickBest(metrics, (a, b) => a.advantage_pct > b.advantage_pct);
  const safest = pickBest(metrics, (a, b) => a.risk_of_ruin < b.risk_of_ruin);
  const handsPerCell = metrics[0]?.n_hands_simulated ?? 0;

  return (
    <>
      <header className="page-header">
        <h1 className="page-title">Six systèmes de comptage, un même protocole</h1>
        <p className="page-lead">
          Chaque système joue {formatCompact(handsPerCell)} de mains par config sur exactement les mêmes sabots
          (Common Random Numbers), avec les mêmes règles et la même rampe de mise. Au total,{" "}
          {formatCompact(data.meta.total_hands_simulated)} de mains ont été simulées par le moteur Rust.
        </p>
      </header>

      <div className="grid-4">
        <MetricCard
          label="Mains simulées"
          value={formatCompact(data.meta.total_hands_simulated)}
          hint={`${formatInteger(data.meta.n_runs)} runs, seed ${data.meta.master_seed}`}
        />
        <MetricCard
          label="Meilleur avantage"
          value={best ? formatPercent(best.advantage_pct, 2, true) : "—"}
          hint={best ? nameOf(best.system) : undefined}
        />
        <MetricCard
          label="Plus faible risque de ruine"
          value={safest ? formatFractionAsPercent(safest.risk_of_ruin) : "—"}
          hint={safest ? `${nameOf(safest.system)}, bankroll de ${formatInteger(data.meta.ror_bankroll_units)} u` : undefined}
        />
        <MetricCard
          label="Basic strategy seule"
          value={formatPercent(data.baseline.advantage_pct, 2, true)}
          hint="mise plate, sans comptage"
        />
      </div>

      <ChartCard
        title="Risque / rendement"
        subtitle={`Config « ${config.label} »`}
        height={300}
        source="metrics.json"
      >
        <RiskReturnScatter metrics={metrics} systems={data.systems} />
      </ChartCard>
      <Link to={`/comparaison${query}`} className="link-arrow">
        Voir la comparaison complète →
      </Link>

      <section className="page-header">
        <h2 className="section-title">Les six systèmes</h2>
      </section>
      <SystemCardGrid systems={data.systems} metrics={metrics} query={query} />
    </>
  );
}
