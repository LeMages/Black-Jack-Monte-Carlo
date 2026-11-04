/**
 * Tableau récapitulatif triable (clic sur l'en-tête).
 */

// Importation des modules framework
import { useState } from "react";
import { Link } from "react-router-dom";

// Importation des modules internes
import { SYSTEM_COLORS } from "../../../content/system_colors";
import {
  formatFractionAsPercent,
  formatN0,
  formatNumber,
  formatPercent,
} from "../../../format/number_format";
import {
  ariaSort,
  nextSortState,
  sortIndicator,
  sortRows,
  type SortState,
  type SortValue,
} from "../../../sorting/sort_rows";
import type { MetricsRow, SystemDefinition } from "../../../types/data";

type ColumnKey = "name" | "advantage" | "ev" | "std" | "n0" | "ror" | "kelly" | "bet";

interface Row {
  system: SystemDefinition;
  metrics: MetricsRow;
}

interface ColumnDef {
  key: ColumnKey;
  label: string;
  numeric: boolean;
  value: (row: Row) => SortValue;
  render: (row: Row) => string;
}

const COLUMNS: ColumnDef[] = [
  { key: "name", label: "Système", numeric: false, value: (r) => r.system.name, render: (r) => r.system.name },
  {
    key: "advantage",
    label: "Avantage",
    numeric: true,
    value: (r) => r.metrics.advantage_pct,
    render: (r) => formatPercent(r.metrics.advantage_pct, 2, true),
  },
  {
    key: "ev",
    label: "EV/100",
    numeric: true,
    value: (r) => r.metrics.ev_per_100,
    render: (r) => formatNumber(r.metrics.ev_per_100, 2),
  },
  {
    key: "std",
    label: "σ/main",
    numeric: true,
    value: (r) => r.metrics.std_per_hand,
    render: (r) => formatNumber(r.metrics.std_per_hand, 2),
  },
  { key: "n0", label: "N₀", numeric: true, value: (r) => r.metrics.n0, render: (r) => formatN0(r.metrics.n0) },
  {
    key: "ror",
    label: "RoR",
    numeric: true,
    value: (r) => r.metrics.risk_of_ruin,
    render: (r) => formatFractionAsPercent(r.metrics.risk_of_ruin),
  },
  {
    key: "kelly",
    label: "Kelly",
    numeric: true,
    value: (r) => r.metrics.kelly_fraction,
    render: (r) => formatFractionAsPercent(r.metrics.kelly_fraction, 3),
  },
  {
    key: "bet",
    label: "Mise moy.",
    numeric: true,
    value: (r) => r.metrics.avg_bet,
    render: (r) => formatNumber(r.metrics.avg_bet, 2),
  },
];

interface Props {
  rows: Row[];
  /** Suffixe `?config=` des liens vers les pages-rapport. */
  query: string;
}

export default function SummaryTable({ rows, query }: Props) {
  const [sort, setSort] = useState<SortState<ColumnKey> | null>(null);
  const accessor = (row: Row, key: ColumnKey) => {
    const column = COLUMNS.find((c) => c.key === key);
    return column ? column.value(row) : null;
  };
  const sorted = sort ? sortRows(rows, accessor, sort) : rows;
  return (
    <div className="table-scroll">
      <table className="data-table">
        <thead>
          <tr>
            {COLUMNS.map((c) => (
              <th key={c.key} scope="col" aria-sort={ariaSort(sort, c.key)}>
                <button
                  type="button"
                  className="sort-button"
                  onClick={() => setSort((prev) => nextSortState(prev, c.key, c.numeric))}
                >
                  {c.label}
                  <span className="sort-indicator" aria-hidden="true">
                    {sortIndicator(sort, c.key)}
                  </span>
                </button>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sorted.map((row) => (
            <tr key={row.system.id}>
              {COLUMNS.map((c) =>
                c.key === "name" ? (
                  <td key={c.key}>
                    <Link to={`/systeme/${row.system.id}${query}`} className="system-link">
                      <span className="swatch" style={{ background: SYSTEM_COLORS[row.system.id] }} aria-hidden="true" />
                      {row.system.name}
                    </Link>
                  </td>
                ) : (
                  <td key={c.key}>{c.render(row)}</td>
                ),
              )}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
