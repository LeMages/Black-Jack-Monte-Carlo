/**
 * Navigation latérale : accueil, six systèmes, comparaison.
 */

// Importation des modules framework
import { NavLink } from "react-router-dom";

// Importation des modules internes
import { SYSTEM_COLORS } from "../content/system_colors";
import { useData } from "../data/data_context";
import { configQuery, useSelectedConfig } from "../hooks/use_selected_config";

function navClass({ isActive }: { isActive: boolean }): string {
  return isActive ? "sidebar__link sidebar__link--active" : "sidebar__link";
}

export default function Sidebar() {
  const { systems } = useData();
  const [config] = useSelectedConfig();
  const query = configQuery(config.id);
  return (
    <nav className="sidebar" aria-label="Navigation principale">
      <NavLink to={`/${query}`} end className={navClass}>
        Accueil
      </NavLink>
      <p className="sidebar__section">Systèmes</p>
      <ul className="sidebar__list">
        {systems.map((s) => (
          <li key={s.id}>
            <NavLink to={`/systeme/${s.id}${query}`} className={navClass}>
              <span className="swatch" style={{ background: SYSTEM_COLORS[s.id] }} aria-hidden="true" />
              {s.name}
            </NavLink>
          </li>
        ))}
      </ul>
      <hr className="sidebar__separator" />
      <NavLink to={`/comparaison${query}`} className={navClass}>
        Comparaison
      </NavLink>
    </nav>
  );
}
