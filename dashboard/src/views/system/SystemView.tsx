/**
 * Page-rapport d'un système (`/systeme/:slug`).
 */

// Importation des modules framework
import { Link, useParams } from "react-router-dom";

// Importation des modules internes
import AdvantageByTcChart from "./components/AdvantageByTcChart";
import CardValueTiles from "./components/CardValueTiles";
import CorrelationCard from "./components/CorrelationCard";
import CountingExampleCard from "./components/CountingExampleCard";
import DrawdownChart from "./components/DrawdownChart";
import EquityFanChart from "./components/EquityFanChart";
import ProsConsCard from "./components/ProsConsCard";
import SystemMetricsGrid from "./components/SystemMetricsGrid";
import TcFrequencyChart from "./components/TcFrequencyChart";
import Card from "../../components/Card";
import ChartCard from "../../components/ChartCard";
import NotFound from "../../components/NotFound";
import { SYSTEM_COLORS } from "../../content/system_colors";
import { SYSTEMS_CONTENT } from "../../content/systems_content";
import { useData } from "../../data/data_context";
import { drawdownBinsFor, equityFor, metricsFor, systemById, tcFor } from "../../data/select";
import { formatInteger } from "../../format/number_format";
import { configQuery, useSelectedConfig } from "../../hooks/use_selected_config";

export default function SystemView() {
  const { slug = "" } = useParams();
  const data = useData();
  const [config] = useSelectedConfig();
  const system = systemById(data, slug);
  if (!system) {
    return <NotFound message={`Le système « ${slug} » n'existe pas.`} />;
  }
  const content = SYSTEMS_CONTENT[system.id];
  const color = SYSTEM_COLORS[system.id];
  const metrics = metricsFor(data, system.id, config.id);
  const index = data.systems.findIndex((s) => s.id === system.id);
  const previous = data.systems[(index - 1 + data.systems.length) % data.systems.length];
  const next = data.systems[(index + 1) % data.systems.length];
  const query = configQuery(config.id);
  const equity = equityFor(data, system.id, config.id);
  // KO mise au running count brut : ses bins de compte sont des running counts
  const countLabel = system.balanced ? "TC" : "RC";
  const countAxis = system.balanced
    ? system.ace_side_count
      ? "True count de mise (side-count des As inclus)"
      : "True count"
    : "Running count (KO, sans division)";

  return (
    <>
      {/* 1. En-tête */}
      <header className="page-header">
        <div className="badges">
          <span className="badge">Niveau {system.level}</span>
          <span className="badge">{system.balanced ? "Équilibré" : "Non équilibré"}</span>
          {system.ace_side_count && <span className="badge">Side-count des As</span>}
          <span className="badge badge--neutral">
            {content.author}, {content.year}
          </span>
        </div>
        <h1 className="system-name">{system.name}</h1>
        <p className="page-lead">{content.tagline}</p>
      </header>

      {/* 2. En quoi ça consiste */}
      <Card title="En quoi ça consiste">
        <div className="prose">
          {content.paragraphs.map((p) => (
            <p key={p}>{p}</p>
          ))}
        </div>
      </Card>

      {/* 3. Valeurs des cartes */}
      <Card title="Valeurs des cartes">
        <CardValueTiles tags={system.tags} />
      </Card>

      {/* 4. Exemple calculé */}
      <CountingExampleCard system={system} />

      {/* 5. Corrélations, forces et faiblesses */}
      <div className="grid-2">
        <CorrelationCard system={system} />
        <ProsConsCard strengths={content.strengths} weaknesses={content.weaknesses} />
      </div>

      {/* 6. Métriques de la config */}
      <section className="page-header">
        <h2 className="section-title">Résultats — {config.label}</h2>
        <p className="muted">
          Modèle A (unités, bankroll infinie) sur {metrics ? formatInteger(metrics.n_hands_simulated) : "—"} mains.
          Courbes de bankroll : modèle B, politique {config.bankroll_policy}, bankroll initiale de{" "}
          {formatInteger(config.initial_bankroll)} unités, mises de {formatInteger(config.min_bet)} à{" "}
          {formatInteger(config.max_bet)} unités, sur {formatInteger(config.bankroll_hands)} mains.
        </p>
      </section>
      {metrics ? (
        <SystemMetricsGrid metrics={metrics} rorBankroll={data.meta.ror_bankroll_units} />
      ) : (
        <p className="muted">Aucune métrique pour cette config.</p>
      )}

      {/* 7. Graphes */}
      <div className="grid-2">
        <ChartCard title="Fréquence du true count" subtitle={countAxis} source="tc_distribution.json">
          <TcFrequencyChart rows={tcFor(data, system.id, config.id)} color={color} countLabel={countLabel} />
        </ChartCard>
        <ChartCard
          title="Avantage selon le true count"
          subtitle={`Avantage par unité misée, par bin de ${countLabel}`}
          source="tc_distribution.json"
        >
          <AdvantageByTcChart rows={tcFor(data, system.id, config.id)} color={color} countLabel={countLabel} />
        </ChartCard>
        <ChartCard
          title="Évolution de la bankroll"
          subtitle={`${formatInteger(data.meta.n_runs)} runs ; axe droit : part des runs ruinés`}
          source="equity_curves.json"
        >
          <EquityFanChart rows={equity} color={color} />
        </ChartCard>
        <ChartCard
          title="Pire repli (max drawdown)"
          subtitle="Nombre de runs par tranche de repli depuis un plus haut"
          source="drawdown_histogram.json"
        >
          <DrawdownChart rows={drawdownBinsFor(data, system.id, config.id)} color={color} />
        </ChartCard>
      </div>

      {/* 8. Navigation entre systèmes */}
      <nav className="pager" aria-label="Systèmes voisins">
        <Link to={`/systeme/${previous.id}${query}`}>← {previous.name}</Link>
        <Link to={`/systeme/${next.id}${query}`}>{next.name} →</Link>
      </nav>
    </>
  );
}
