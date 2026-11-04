/**
 * Barre du haut : logo, titre et sélecteur de config.
 */

// Importation des modules framework
import { Link } from "react-router-dom";

// Importation des modules internes
import ConfigSelect from "./ConfigSelect";
import { configQuery, useSelectedConfig } from "../hooks/use_selected_config";

export default function TopBar() {
  const [config] = useSelectedConfig();
  return (
    <header className="topbar">
      <Link to={`/${configQuery(config.id)}`} className="topbar__brand">
        <span className="topbar__logo" aria-hidden="true">
          BJ
        </span>
        <span className="topbar__title">Blackjack Monte Carlo</span>
      </Link>
      <ConfigSelect />
    </header>
  );
}
