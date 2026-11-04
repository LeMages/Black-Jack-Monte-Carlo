/**
 * Contexte React portant les données chargées une seule fois au démarrage.
 */

// Importation des modules framework
import { createContext, useContext } from "react";

// Importation des modules internes
import type { DataBundle } from "../types/data";

export const DataContext = createContext<DataBundle | null>(null);

/**
 * Accède aux données du dashboard.
 *
 * @returns Le bundle de données.
 * @throws Error si appelé hors de `DataProvider`.
 */
export function useData(): DataBundle {
  const data = useContext(DataContext);
  if (data === null) {
    throw new Error("useData doit être appelé sous <DataProvider>");
  }
  return data;
}
