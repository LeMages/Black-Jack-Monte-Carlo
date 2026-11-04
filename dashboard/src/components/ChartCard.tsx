/**
 * Carte de graphe : titre, sous-titre optionnel, graphe et mention de la source.
 */

// Importation des modules framework
import type { ReactNode } from "react";

interface ChartCardProps {
  title: string;
  subtitle?: string;
  /** Fichier de données dont provient le graphe. */
  source: string;
  children: ReactNode;
}

export default function ChartCard({ title, subtitle, source, children }: ChartCardProps) {
  return (
    <figure className="card chart-card">
      <figcaption>
        <h3 className="chart-card__title">{title}</h3>
        {subtitle && <p className="chart-card__subtitle">{subtitle}</p>}
      </figcaption>
      <div className="chart-card__plot">{children}</div>
      <p className="chart-card__source">Source : {source}</p>
    </figure>
  );
}
