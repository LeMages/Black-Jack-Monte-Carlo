/**
 * Tuile de métrique : libellé, valeur, précision optionnelle.
 */

interface MetricCardProps {
  label: string;
  value: string;
  hint?: string;
}

export default function MetricCard({ label, value, hint }: MetricCardProps) {
  return (
    <div className="card metric-card">
      <p className="metric-card__label">{label}</p>
      <p className="metric-card__value">{value}</p>
      {hint && <p className="metric-card__hint">{hint}</p>}
    </div>
  );
}
