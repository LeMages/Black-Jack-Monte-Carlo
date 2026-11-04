/**
 * Histogramme du pire repli (max drawdown) par run, binné à l'export.
 */

// Importation des modules externes
import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

// Importation des modules internes
import { CHART_MARGIN, GRID_COLOR, TICK_STYLE } from "../../../components/chart_theme";
import TooltipBox from "../../../components/TooltipBox";
import { formatInteger, formatNumber } from "../../../format/number_format";
import type { DrawdownBinRow } from "../../../types/data";

interface Props {
  rows: DrawdownBinRow[];
  color: string;
}

export default function DrawdownChart({ rows, color }: Props) {
  return (
    <ResponsiveContainer width="100%" height="100%">
      <BarChart data={rows} margin={CHART_MARGIN} barCategoryGap={2}>
        <CartesianGrid stroke={GRID_COLOR} vertical={false} />
        <XAxis
          dataKey="bin_start"
          tick={TICK_STYLE}
          tickLine={false}
          interval={3}
          tickFormatter={(v: number) => `${formatNumber(v, 0)} %`}
        />
        <YAxis
          tick={TICK_STYLE}
          tickLine={false}
          axisLine={false}
          width={48}
          allowDecimals={false}
          tickFormatter={(v: number) => formatInteger(v)}
        />
        <Tooltip
          cursor={{ fill: "rgba(24,24,27,0.06)" }}
          content={({ active, payload }) => {
            const row = active && payload?.[0] ? (payload[0].payload as DrawdownBinRow) : null;
            if (!row) return null;
            return (
              <TooltipBox
                title={`Repli de ${formatNumber(row.bin_start, 0)} à ${formatNumber(row.bin_end, 0)} %`}
                rows={[{ label: "Runs", value: formatInteger(row.count) }]}
              />
            );
          }}
        />
        <Bar dataKey="count" fill={color} radius={[4, 4, 0, 0]} isAnimationActive={false} />
      </BarChart>
    </ResponsiveContainer>
  );
}
