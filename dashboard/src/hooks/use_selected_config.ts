/**
 * Config sélectionnée, conservée dans l'URL (`?config=`).
 */

// Importation des modules framework
import { useCallback } from "react";
import { useSearchParams } from "react-router-dom";

// Importation des modules internes
import { useData } from "../data/data_context";
import type { ConfigEntry } from "../types/data";

/**
 * Lit et modifie la config sélectionnée.
 *
 * Une valeur absente ou inconnue retombe sur la première config publiée.
 *
 * @returns La config courante et une fonction pour en changer.
 */
export function useSelectedConfig(): [ConfigEntry, (id: string) => void] {
  const { configs } = useData();
  const [params, setParams] = useSearchParams();
  const requested = params.get("config");
  const current = configs.find((c) => c.id === requested) ?? configs[0];
  const select = useCallback(
    (id: string) => {
      setParams(
        (prev) => {
          const next = new URLSearchParams(prev);
          next.set("config", id);
          return next;
        },
        { replace: true },
      );
    },
    [setParams],
  );
  return [current, select];
}

/**
 * Suffixe de requête à ajouter aux liens internes pour garder la config.
 *
 * @param configId - Identifiant de config.
 * @returns La chaîne `?config=<id>`.
 */
export function configQuery(configId: string): string {
  return `?config=${encodeURIComponent(configId)}`;
}
