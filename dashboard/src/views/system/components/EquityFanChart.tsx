/**
 * Fan chart de la bankroll : bandes P5–P95 et P25–P75, médiane, et fraction de
 * runs ruinés sur un axe secondaire (en %).
 */

// Importation des modules externes
import {
  Area,
  CartesianGrid,
  ComposedChart,
  Legend,
  Line,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

// Importation des modules internes
import { CHART_MARGIN, GRID_COLOR, RUIN_COLOR, TICK_STYLE } from "../../../components/chart_theme";
import TooltipBox from "../../../components/TooltipBox";
import { formatFractionAsPercent, formatInteger } from "../../../format/number_format";
import type { EquityRow } from "../../../types/data";

interface Props {
  rows: EquityRow[];
  color: string;
}

export default function EquityFanChart({ rows, color }: Props) {
  return (
    <ResponsiveContainer width="100%" height="100%">
      <ComposedChart data={rows} margin={{ ...CHART_MARGIN, right: 4 }}>
        <CartesianGrid stroke={GRID_COLOR} vertical={false} />
        <XAxis
          dataKey="hand_index"
          type="number"
          domain={["dataMin", "dataMax"]}
          tick={TICK_STYLE}
          tickLine={false}
          tickFormatter={(v: number) => formatInteger(v)}
        />
        <YAxis
          yAxisId="bankroll"
          tick={TICK_STYLE}
          tickLine={false}
          axisLine={false}
          width={60}
          tickFormatter={(v: number) => formatInteger(v)}
        />
        <YAxis
          yAxisId="ruin"
          orientation="right"
          domain={[0, 1]}
          tick={{ ...TICK_STYLE, fill: RUIN_COLOR }}
          tickLine={false}
          axisLine={false}
          width={44}
          tickFormatter={(v: number) => formatFractionAsPercent(v, 0)}
        />
        <Tooltip
          content={({ active, payload }) => {
            const row = active && payload?.[0] ? (payload[0].payload as EquityRow) : null;
            if (!row) return null;
            return (
              <TooltipBox
                title={`Main ${formatInteger(row.hand_index)}`}
                rows={[
                  { label: "P95", value: formatInteger(row.p95) },
                  { label: "P75", value: formatInteger(row.p75) },
                  { label: "Médiane", value: formatInteger(row.p50), color },
                  { label: "P25", value: formatInteger(row.p25) },
                  { label: "P5", value: formatInteger(row.p05) },
                  { label: "Runs ruinés", value: formatFractionAsPercent(row.frac_ruined), color: RUIN_COLOR },
                ]}
              />
            );
          }}
        />
        <Legend wrapperStyle={{ fontSize: 13 }} iconSize={10} />
        <Area
          yAxisId="bankroll"
          name="P5–P95"
          dataKey={(d: EquityRow) => [d.p05, d.p95]}
          stroke="none"
          fill={color}
          fillOpacity={0.14}
          isAnimationActive={false}
        />
        <Area
          yAxisId="bankroll"
          name="P25–P75"
          dataKey={(d: EquityRow) => [d.p25, d.p75]}
          stroke="none"
          fill={color}
          fillOpacity={0.3}
          isAnimationActive={false}
        />
        <Line
          yAxisId="bankroll"
          name="Médiane"
          dataKey="p50"
          stroke={color}
          strokeWidth={2}
          dot={false}
          isAnimationActive={false}
        />
        <Line
          yAxisId="ruin"
          name="Runs ruinés (axe droit)"
          dataKey="frac_ruined"
          stroke={RUIN_COLOR}
          strokeWidth={2}
          strokeDasharray="5 3"
          dot={false}
          isAnimationActive={false}
        />
      </ComposedChart>
    </ResponsiveContainer>
  );
}
