/**
 * Sélecteur de config global (barre du haut), synchronisé avec `?config=`.
 */

// Importation des modules internes
import { useData } from "../data/data_context";
import { useSelectedConfig } from "../hooks/use_selected_config";

export default function ConfigSelect() {
  const { configs } = useData();
  const [current, select] = useSelectedConfig();
  return (
    <label className="config-select">
      <span className="config-select__label">Config</span>
      <select value={current.id} onChange={(e) => select(e.target.value)}>
        {configs.map((c) => (
          <option key={c.id} value={c.id}>
            {c.label}
          </option>
        ))}
      </select>
    </label>
  );
}
