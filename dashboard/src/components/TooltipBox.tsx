/**
 * Contenu de tooltip commun à tous les graphes.
 */

/** Ligne d'un tooltip : pastille optionnelle, libellé, valeur formatée. */
export interface TooltipRow {
  label: string;
  value: string;
  color?: string;
}

interface TooltipBoxProps {
  title: string;
  rows: TooltipRow[];
}

export default function TooltipBox({ title, rows }: TooltipBoxProps) {
  return (
    <div className="chart-tooltip">
      <p className="chart-tooltip__title">{title}</p>
      {rows.map((row) => (
        <p key={row.label} className="chart-tooltip__row">
          {row.color && <span className="swatch" style={{ background: row.color }} aria-hidden="true" />}
          <span>{row.label} :</span>
          <strong>{row.value}</strong>
        </p>
      ))}
    </div>
  );
}
