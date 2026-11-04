/**
 * Page de comparaison des six systèmes (`/comparaison`).
 */

// Importation des modules internes
import MedianBankrollChart from "./components/MedianBankrollChart";
import MetricBarChart, { type BarDatum } from "./components/MetricBarChart";
import SummaryTable from "./components/SummaryTable";
import Card from "../../components/Card";
import ChartCard from "../../components/ChartCard";
import RiskReturnScatter from "../../components/RiskReturnScatter";
import { useData } from "../../data/data_context";
import { equityFor, metricsForConfig } from "../../data/select";
import {
  formatCompact,
  formatFractionAsPercent,
  formatInteger,
  formatN0,
  formatPercent,
} from "../../format/number_format";
import { configQuery, useSelectedConfig } from "../../hooks/use_selected_config";
import type { EquityRow, MetricsRow, SystemId } from "../../types/data";

/** Config publiée qui illustre la sur-mise (10 % de la bankroll à plat). */
const FLAT_DEMO_CONFIG = "s15-flat10";

export default function ComparisonView() {
  const data = useData();
  const [config] = useSelectedConfig();
  const metrics = metricsForConfig(data, config.id);
  const nameOf = (id: SystemId) => data.systems.find((s) => s.id === id)?.name ?? id;
  const bars = (pick: (m: MetricsRow) => number | null): BarDatum[] =>
    metrics.map((m) => ({ system: m.system, name: nameOf(m.system), value: pick(m) }));
  const curves = Object.fromEntries(
    data.systems.map((s) => [s.id, equityFor(data, s.id, config.id)]),
  ) as Record<SystemId, EquityRow[]>;
  const rows = metrics.flatMap((m) => {
    const system = data.systems.find((s) => s.id === m.system);
    return system ? [{ system, metrics: m }] : [];
  });
  const handsPerCell = metrics[0]?.n_hands_simulated ?? 0;
  const maxKelly = Math.max(0, ...metrics.map((m) => m.kelly_fraction));
  // Part de runs ruinés en fin d'horizon pour la config de démonstration « 10 % à plat »
  const flatRuin = equityFor(data, "hi-lo", FLAT_DEMO_CONFIG).at(-1)?.frac_ruined;
  const flatHands = data.configs.find((c) => c.id === FLAT_DEMO_CONFIG)?.bankroll_hands;

  return (
    <>
      <header className="page-header">
        <h1 className="page-title">Comparaison des 6 systèmes</h1>
        <p className="page-lead">
          Les six systèmes jouent exactement les mêmes sabots (Common Random Numbers) : {formatCompact(handsPerCell)}{" "}
          de mains chacun pour la config « {config.label} ». Les écarts mesurés viennent donc du système, pas du
          hasard des cartes.
        </p>
      </header>

      <div className="grid-3">
        <ChartCard title="Avantage" subtitle="Gain / mises initiales" source="metrics.json">
          <MetricBarChart
            data={bars((m) => m.advantage_pct)}
            format={(v) => formatPercent(v, 2, true)}
            tickFormat={(v) => formatPercent(v, 2)}
            label="Avantage"
          />
        </ChartCard>
        <ChartCard
          title="N₀ (mains)"
          subtitle="Plus court = meilleur ; absent si l'avantage est négatif"
          source="metrics.json"
        >
          <MetricBarChart
            data={bars((m) => m.n0)}
            format={(v) => formatN0(v)}
            tickFormat={(v) => formatCompact(v)}
            label="N₀"
          />
        </ChartCard>
        <ChartCard
          title="Risque de ruine"
          subtitle={`Bankroll de ${formatInteger(data.meta.ror_bankroll_units)} unités`}
          source="metrics.json"
        >
          <MetricBarChart
            data={bars((m) => m.risk_of_ruin)}
            format={(v) => formatFractionAsPercent(v)}
            tickFormat={(v) => formatFractionAsPercent(v, 0)}
            label="Risque de ruine"
          />
        </ChartCard>
      </div>

      <ChartCard
        title="Bankroll médiane dans le temps"
        subtitle={`Médiane (P50) de ${formatInteger(data.meta.n_runs)} runs, politique ${config.bankroll_policy}. Cliquez sur un système pour le masquer.`}
        height={380}
        source="equity_curves.json"
      >
        <MedianBankrollChart systems={data.systems} curves={curves} />
      </ChartCard>

      <ChartCard
        title="Risque / rendement"
        subtitle="En bas à droite : meilleur compromis"
        height={360}
        source="metrics.json"
      >
        <RiskReturnScatter metrics={metrics} systems={data.systems} />
      </ChartCard>

      <Card title="Tableau récapitulatif">
        <SummaryTable rows={rows} query={configQuery(config.id)} />
        <p className="bar-note">Source : metrics.json. EV/100 et mise moyenne en unités ; σ par main en unités.</p>
      </Card>

      <aside className="callout">
        <p>
          <strong>Pourquoi la mise compte autant que le système.</strong> Le critère de Kelly fixe la taille
          d&apos;unité optimale à avantage / variance : au mieux{" "}
          {formatFractionAsPercent(maxKelly, 3)} de la bankroll pour cette config. Miser 10 % de la bankroll à plat,
          quel que soit le compte, revient à jouer très au-dessus de Kelly : la bankroll s&apos;effondre même quand le
          compte est favorable.
          {flatRuin !== undefined &&
            ` Avec la config « ${FLAT_DEMO_CONFIG} », ${formatFractionAsPercent(flatRuin, 1)} des runs Hi-Lo sont ruinés${flatHands ? ` en ${formatInteger(flatHands)} mains` : ""}.`}
        </p>
      </aside>
    </>
  );
}
