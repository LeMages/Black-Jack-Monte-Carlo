/**
 * Nuage risque / rendement : avantage (X) et risque de ruine (Y), un point
 * étiqueté par système. Utilisé sur l'accueil (aperçu) et la comparaison.
 */

// Importation des modules externes
import {
  CartesianGrid,
  LabelList,
  ResponsiveContainer,
  Scatter,
  ScatterChart,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

// Importation des modules internes
import { CHART_MARGIN, GRID_COLOR, TICK_STYLE } from "./chart_theme";
import TooltipBox from "./TooltipBox";
import { SYSTEM_COLORS } from "../content/system_colors";
import { formatFractionAsPercent, formatPercent } from "../format/number_format";
import type { MetricsRow, SystemDefinition } from "../types/data";

interface Props {
  metrics: MetricsRow[];
  systems: SystemDefinition[];
}

interface Point {
  name: string;
  color: string;
  x: number;
  y: number;
}

/** Arrondit un intervalle aux multiples de `step`, avec une marge d'un demi-pas. */
function niceDomain(values: number[], step: number): [number, number] {
  if (values.length === 0) return [0, step];
  const lo = Math.floor((Math.min(...values) - step / 2) / step) * step;
  const hi = Math.ceil((Math.max(...values) + step / 2) / step) * step;
  return [lo, hi];
}

export default function RiskReturnScatter({ metrics, systems }: Props) {
  const points: Point[] = metrics.map((m) => ({
    name: systems.find((s) => s.id === m.system)?.name ?? m.system,
    color: SYSTEM_COLORS[m.system],
    x: m.advantage_pct,
    y: m.risk_of_ruin,
  }));
  // Axe Y ancré à 0 (un risque se lit depuis zéro), plafonné à 100 %
  const yMax = Math.min(1, niceDomain(points.map((p) => p.y), 0.02)[1]);
  const xDomain = niceDomain(points.map((p) => p.x), 0.05);
  // Graduations explicites : un pas rond évite les libellés arrondis en doublon (2,5 % → « 3 % »)
  const yStep = yMax <= 0.2 ? 0.02 : 0.2;
  const yTicks = Array.from({ length: Math.round(yMax / yStep) + 1 }, (_, i) => i * yStep);
  return (
    <ResponsiveContainer width="100%" height="100%">
      <ScatterChart margin={{ ...CHART_MARGIN, top: 20, right: 24 }}>
        <CartesianGrid stroke={GRID_COLOR} />
        <XAxis
          type="number"
          dataKey="x"
          name="Avantage"
          domain={xDomain}
          tick={TICK_STYLE}
          tickLine={false}
          tickFormatter={(v: number) => formatPercent(v, 2)}
          label={{ value: "Avantage", position: "insideBottomRight", offset: -2, fontSize: 12, fill: "#52525b" }}
        />
        <YAxis
          type="number"
          dataKey="y"
          name="Risque de ruine"
          domain={[0, yMax]}
          ticks={yTicks}
          tick={TICK_STYLE}
          tickLine={false}
          axisLine={false}
          width={52}
          tickFormatter={(v: number) => formatFractionAsPercent(v, 0)}
        />
        <Tooltip
          cursor={{ strokeDasharray: "3 3" }}
          content={({ active, payload }) => {
            const p = active && payload?.[0] ? (payload[0].payload as Point) : null;
            if (!p) return null;
            return (
              <TooltipBox
                title={p.name}
                rows={[
                  { label: "Avantage", value: formatPercent(p.x, 2, true), color: p.color },
                  { label: "Risque de ruine", value: formatFractionAsPercent(p.y) },
                ]}
              />
            );
          }}
        />
        {points.map((p) => (
          <Scatter key={p.name} name={p.name} data={[p]} fill={p.color} isAnimationActive={false}>
            <LabelList dataKey="name" position="top" fontSize={12} fill="#18181b" />
          </Scatter>
        ))}
      </ScatterChart>
    </ResponsiveContainer>
  );
}
