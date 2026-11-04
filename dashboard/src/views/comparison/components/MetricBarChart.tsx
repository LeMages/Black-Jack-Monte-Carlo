/**
 * Barres horizontales d'une métrique, une barre par système (couleur fixe).
 */

// Importation des modules externes
import { Bar, BarChart, CartesianGrid, Cell, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

// Importation des modules internes
import { CHART_MARGIN, GRID_COLOR, TICK_STYLE } from "../../../components/chart_theme";
import TooltipBox from "../../../components/TooltipBox";
import { SYSTEM_COLORS } from "../../../content/system_colors";
import type { SystemId } from "../../../types/data";

/** Barre à afficher : valeur déjà calculée par le moteur. */
export interface BarDatum {
  system: SystemId;
  name: string;
  value: number | null;
}

interface Props {
  data: BarDatum[];
  format: (value: number) => string;
  /** Format compact des graduations. */
  tickFormat: (value: number) => string;
  label: string;
}

export default function MetricBarChart({ data, format, tickFormat, label }: Props) {
  // Une valeur absente (N₀ indéfini) est tracée à 0 et signalée dans le tooltip
  const rows = data.map((d) => ({ ...d, plotted: d.value ?? 0 }));
  return (
    <ResponsiveContainer width="100%" height="100%">
      <BarChart data={rows} layout="vertical" margin={{ ...CHART_MARGIN, right: 16 }} barCategoryGap={6}>
        <CartesianGrid stroke={GRID_COLOR} horizontal={false} />
        <XAxis type="number" tick={TICK_STYLE} tickLine={false} tickFormatter={tickFormat} />
        <YAxis type="category" dataKey="name" tick={TICK_STYLE} tickLine={false} axisLine={false} width={88} />
        <Tooltip
          cursor={{ fill: "rgba(24,24,27,0.06)" }}
          content={({ active, payload }) => {
            const row = active && payload?.[0] ? (payload[0].payload as BarDatum) : null;
            if (!row) return null;
            return (
              <TooltipBox
                title={row.name}
                rows={[
                  {
                    label,
                    value: row.value === null ? "non défini" : format(row.value),
                    color: SYSTEM_COLORS[row.system],
                  },
                ]}
              />
            );
          }}
        />
        <Bar dataKey="plotted" radius={[0, 4, 4, 0]} isAnimationActive={false}>
          {rows.map((r) => (
            <Cell key={r.system} fill={SYSTEM_COLORS[r.system]} />
          ))}
        </Bar>
      </BarChart>
    </ResponsiveContainer>
  );
}
