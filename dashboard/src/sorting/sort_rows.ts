/**
 * Tri des lignes du tableau récapitulatif.
 *
 * Le tri réordonne des valeurs déjà calculées par le moteur : ce n'est pas un
 * calcul statistique.
 */

/** Sens de tri. */
export type SortDirection = "asc" | "desc";

/** État de tri d'un tableau. */
export interface SortState<K extends string> {
  key: K;
  direction: SortDirection;
}

/** Valeur triable : nombre, texte, ou `null` (toujours placé en dernier). */
export type SortValue = number | string | null;

/**
 * Compare deux valeurs triables ; `null` est toujours placé après les autres.
 *
 * @param a - Première valeur.
 * @param b - Seconde valeur.
 * @param direction - Sens de tri.
 * @returns Négatif si `a` passe avant `b`, positif sinon, 0 si égalité.
 */
export function compareValues(a: SortValue, b: SortValue, direction: SortDirection): number {
  if (a === null || b === null) {
    if (a === b) return 0;
    return a === null ? 1 : -1;
  }
  const sign = direction === "asc" ? 1 : -1;
  if (typeof a === "number" && typeof b === "number") {
    return sign * (a - b);
  }
  return sign * String(a).localeCompare(String(b), "fr-FR");
}

/**
 * Trie une copie des lignes selon une colonne (tri stable).
 *
 * @param rows - Lignes à trier (non modifiées).
 * @param accessor - Lecture de la valeur triable d'une ligne pour une clé.
 * @param state - Colonne et sens de tri.
 * @returns Une nouvelle liste triée.
 */
export function sortRows<T, K extends string>(
  rows: readonly T[],
  accessor: (row: T, key: K) => SortValue,
  state: SortState<K>,
): T[] {
  return rows
    .map((row, index) => ({ row, index }))
    .sort((x, y) => {
      const cmp = compareValues(accessor(x.row, state.key), accessor(y.row, state.key), state.direction);
      return cmp !== 0 ? cmp : x.index - y.index;
    })
    .map(({ row }) => row);
}

/**
 * Calcule l'état de tri après un clic sur un en-tête.
 *
 * Premier clic : décroissant pour les nombres (le plus grand d'abord), croissant
 * pour le texte ; clic suivant sur la même colonne : inversion.
 *
 * @param current - État courant, ou `null` si aucun tri.
 * @param key - Colonne cliquée.
 * @param numeric - Vrai si la colonne est numérique.
 * @returns Le nouvel état de tri.
 */
export function nextSortState<K extends string>(
  current: SortState<K> | null,
  key: K,
  numeric: boolean,
): SortState<K> {
  if (current && current.key === key) {
    return { key, direction: current.direction === "asc" ? "desc" : "asc" };
  }
  return { key, direction: numeric ? "desc" : "asc" };
}

/**
 * Valeur de l'attribut `aria-sort` d'un en-tête.
 *
 * @param state - État de tri courant.
 * @param key - Colonne de l'en-tête.
 * @returns `ascending`, `descending` ou `none`.
 */
export function ariaSort<K extends string>(
  state: SortState<K> | null,
  key: K,
): "ascending" | "descending" | "none" {
  if (!state || state.key !== key) return "none";
  return state.direction === "asc" ? "ascending" : "descending";
}

/**
 * Indicateur visuel de tri d'un en-tête.
 *
 * @param state - État de tri courant.
 * @param key - Colonne de l'en-tête.
 * @returns « ↑ », « ↓ » ou « ↕ ».
 */
export function sortIndicator<K extends string>(state: SortState<K> | null, key: K): string {
  if (!state || state.key !== key) return "↕";
  return state.direction === "asc" ? "↑" : "↓";
}
