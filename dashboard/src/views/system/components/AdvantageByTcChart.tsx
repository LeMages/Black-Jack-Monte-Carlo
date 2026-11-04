/**
 * Avantage par unité misée selon le bin de compte.
 */

// Importation des modules externes
import {
  CartesianGrid,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

// Importation des modules internes
import { AXIS_COLOR, CHART_MARGIN, GRID_COLOR, TICK_STYLE } from "../../../components/chart_theme";
import TooltipBox from "../../../components/TooltipBox";
import { formatFractionAsPercent, formatPercent } from "../../../format/number_format";
import type { TcRow } from "../../../types/data";

interface Props {
  rows: TcRow[];
  color: string;
  countLabel: string;
}

export default function AdvantageByTcChart({ rows, color, countLabel }: Props) {
  return (
    <ResponsiveContainer width="100%" height="100%">
      <LineChart data={rows} margin={CHART_MARGIN}>
        <CartesianGrid stroke={GRID_COLOR} vertical={false} />
        <XAxis dataKey="true_count" tick={TICK_STYLE} tickLine={false} interval="preserveStartEnd" />
        <YAxis
          tick={TICK_STYLE}
          tickLine={false}
          axisLine={false}
          width={52}
          tickFormatter={(v: number) => formatPercent(v, 0)}
        />
        <ReferenceLine y={0} stroke={AXIS_COLOR} strokeDasharray="4 4" />
        <Tooltip
          content={({ active, payload }) => {
            const row = active && payload?.[0] ? (payload[0].payload as TcRow) : null;
            if (!row) return null;
            return (
              <TooltipBox
                title={`${countLabel} ${row.true_count}`}
                rows={[
                  { label: "Avantage", value: formatPercent(row.advantage_at_tc, 2, true) },
                  { label: "Fréquence", value: formatFractionAsPercent(row.frequency) },
                ]}
              />
            );
          }}
        />
        <Line
          type="monotone"
          dataKey="advantage_at_tc"
          stroke={color}
          strokeWidth={2}
          dot={{ r: 3, fill: color, strokeWidth: 0 }}
          activeDot={{ r: 5 }}
          isAnimationActive={false}
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
