/**
 * Histogramme de la fréquence des bins de compte de mise.
 */

// Importation des modules externes
import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

// Importation des modules internes
import { CHART_MARGIN, GRID_COLOR, TICK_STYLE } from "../../../components/chart_theme";
import TooltipBox from "../../../components/TooltipBox";
import { formatFractionAsPercent, formatNumber } from "../../../format/number_format";
import type { TcRow } from "../../../types/data";

interface Props {
  rows: TcRow[];
  color: string;
  /** Libellé de l'axe des bins (true count, ou running count pour KO). */
  countLabel: string;
}

export default function TcFrequencyChart({ rows, color, countLabel }: Props) {
  return (
    <ResponsiveContainer width="100%" height="100%">
      <BarChart data={rows} margin={CHART_MARGIN} barCategoryGap={2}>
        <CartesianGrid stroke={GRID_COLOR} vertical={false} />
        <XAxis dataKey="true_count" tick={TICK_STYLE} tickLine={false} interval="preserveStartEnd" />
        <YAxis
          tick={TICK_STYLE}
          tickLine={false}
          axisLine={false}
          width={52}
          tickFormatter={(v: number) => formatFractionAsPercent(v, 0)}
        />
        <Tooltip
          cursor={{ fill: "rgba(24,24,27,0.06)" }}
          content={({ active, payload }) => {
            const row = active && payload?.[0] ? (payload[0].payload as TcRow) : null;
            if (!row) return null;
            return (
              <TooltipBox
                title={`${countLabel} ${row.true_count}`}
                rows={[
                  { label: "Fréquence", value: formatFractionAsPercent(row.frequency) },
                  { label: "Mise moyenne", value: `${formatNumber(row.avg_bet_at_tc)} u` },
                ]}
              />
            );
          }}
        />
        <Bar dataKey="frequency" fill={color} radius={[4, 4, 0, 0]} isAnimationActive={false} />
      </BarChart>
    </ResponsiveContainer>
  );
}
