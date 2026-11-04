/**
 * Page affichée pour une route ou un système inconnu.
 */

// Importation des modules framework
import { Link } from "react-router-dom";

export default function NotFound({ message = "Cette page n'existe pas." }: { message?: string }) {
  return (
    <div className="card">
      <h1 className="page-title">Page introuvable</h1>
      <p>{message}</p>
      <p>
        <Link to="/">Retour à l&apos;accueil</Link>
      </p>
    </div>
  );
}
