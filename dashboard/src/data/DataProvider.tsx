/**
 * Charge les données du dashboard et affiche un état de chargement ou d'erreur.
 */

// Importation des modules framework
import { useEffect, useState, type ReactNode } from "react";

// Importation des modules internes
import { DataContext } from "./data_context";
import { loadData } from "./load_data";
import type { DataBundle } from "../types/data";

type LoadState =
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "ready"; data: DataBundle };

export default function DataProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
    loadData()
      .then((data) => {
        if (!cancelled) setState({ status: "ready", data });
      })
      .catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        if (!cancelled) setState({ status: "error", message });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (state.status === "loading") {
    return <p className="status-message">Chargement des résultats de simulation…</p>;
  }
  if (state.status === "error") {
    return (
      <p className="status-message status-message--error" role="alert">
        Impossible de charger les données : {state.message}
      </p>
    );
  }
  return <DataContext.Provider value={state.data}>{children}</DataContext.Provider>;
}
