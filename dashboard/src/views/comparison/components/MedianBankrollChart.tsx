/**
 * Courbes P50 de la bankroll des six systèmes, avec légende cliquable pour
 * masquer un système.
 */

// Importation des modules framework
import { useState } from "react";

// Importation des modules externes
import { CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

// Importation des modules internes
import { CHART_MARGIN, GRID_COLOR, TICK_STYLE } from "../../../components/chart_theme";
import TooltipBox from "../../../components/TooltipBox";
import { SYSTEM_COLORS } from "../../../content/system_colors";
import { formatInteger } from "../../../format/number_format";
import type { EquityRow, SystemDefinition, SystemId } from "../../../types/data";

interface Props {
  systems: SystemDefinition[];
  /** Courbe d'équité de chaque système (même échantillonnage). */
  curves: Record<SystemId, EquityRow[]>;
}

/** Ligne large : une colonne `p50` par système, indexée par main. */
type WideRow = { hand_index: number } & Partial<Record<SystemId, number>>;

function toWide(systems: SystemDefinition[], curves: Record<SystemId, EquityRow[]>): WideRow[] {
  const byHand = new Map<number, WideRow>();
  for (const s of systems) {
    for (const row of curves[s.id] ?? []) {
      const entry = byHand.get(row.hand_index) ?? { hand_index: row.hand_index };
      entry[s.id] = row.p50;
      byHand.set(row.hand_index, entry);
    }
  }
  return [...byHand.values()].sort((a, b) => a.hand_index - b.hand_index);
}

export default function MedianBankrollChart({ systems, curves }: Props) {
  const [hidden, setHidden] = useState<Set<SystemId>>(new Set());
  const rows = toWide(systems, curves);
  const toggle = (id: SystemId) =>
    setHidden((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12, height: "100%" }}>
      <ul className="legend" aria-label="Afficher ou masquer un système">
        {systems.map((s) => (
          <li key={s.id}>
            <button
              type="button"
              className="legend__button"
              aria-pressed={!hidden.has(s.id)}
              onClick={() => toggle(s.id)}
            >
              <span className="swatch" style={{ background: SYSTEM_COLORS[s.id] }} aria-hidden="true" />
              {s.name}
            </button>
          </li>
        ))}
      </ul>
      <div style={{ flex: "1 1 auto", minHeight: 0 }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={rows} margin={CHART_MARGIN}>
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
              tick={TICK_STYLE}
              tickLine={false}
              axisLine={false}
              width={60}
              tickFormatter={(v: number) => formatInteger(v)}
            />
            <Tooltip
              content={({ active, payload, label }) => {
                if (!active || !payload?.length) return null;
                return (
                  <TooltipBox
                    title={`Main ${formatInteger(Number(label))}`}
                    rows={payload.map((p) => ({
                      label: systems.find((s) => s.id === p.dataKey)?.name ?? String(p.dataKey),
                      value: formatInteger(Number(p.value)),
                      color: SYSTEM_COLORS[p.dataKey as SystemId],
                    }))}
                  />
                );
              }}
            />
            {systems.map((s) => (
              <Line
                key={s.id}
                dataKey={s.id}
                name={s.name}
                stroke={SYSTEM_COLORS[s.id]}
                strokeWidth={2}
                dot={false}
                hide={hidden.has(s.id)}
                isAnimationActive={false}
              />
            ))}
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
