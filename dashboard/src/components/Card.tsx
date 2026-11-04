/**
 * Carte blanche à bordure, avec titre optionnel.
 */

// Importation des modules framework
import type { ReactNode } from "react";

interface CardProps {
  title?: string;
  className?: string;
  children: ReactNode;
}

export default function Card({ title, className, children }: CardProps) {
  return (
    <section className={className ? `card ${className}` : "card"}>
      {title && <h2 className="card__title">{title}</h2>}
      {children}
    </section>
  );
}
