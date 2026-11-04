/**
 * Génération des exemples de comptage des pages-rapport.
 *
 * Fonction pure : les tableaux Carte / Tag / Running et l'encadré du true count
 * sont calculés à partir des tags exportés par le moteur (`systems.json`), jamais
 * écrits à la main.
 */

// Importation des modules internes
import type { SystemDefinition } from "../types/data";

/** Séquence pédagogique commune (annexe Hi-Lo). */
export const EXAMPLE_SEQUENCE = ["3", "5", "K", "7", "Q", "A", "8", "5", "4", "2"] as const;

/** Hypothèses de l'exemple : position dans le sabot. */
export interface ExampleAssumptions {
  /** Nombre de jeux du sabot. */
  nDecks: number;
  /** Jeux restant à distribuer au moment du calcul. */
  decksRemaining: number;
  /** As sortis avant la séquence (side-count). */
  acesSeenBefore: number;
}

/** Hypothèses par défaut : sabot de 6 jeux, 4 jeux restants, 4 As déjà sortis. */
export const DEFAULT_ASSUMPTIONS: ExampleAssumptions = {
  nDecks: 6,
  decksRemaining: 4,
  acesSeenBefore: 4,
};

/** Ligne du tableau Carte / Tag / Running. */
export interface ExampleRow {
  card: string;
  tag: number;
  running: number;
}

/** Correction du side-count des As. */
export interface AceCorrection {
  acesSeen: number;
  acesRemaining: number;
  acesExpected: number;
  acesRich: number;
  aceFactor: number;
  correction: number;
  bettingCount: number;
}

/** Exemple complet calculé pour un système. */
export interface CountingExample {
  initialRunning: number;
  rows: ExampleRow[];
  finalRunning: number;
  decksRemaining: number;
  /** True count (systèmes équilibrés) ; `null` pour KO. */
  trueCount: number | null;
  /** Pivot et comparaison (KO) ; `null` pour les systèmes équilibrés. */
  pivot: { value: number; reached: boolean } | null;
  /** Correction du side-count (Hi-Opt II, Omega II). */
  aceCorrection: AceCorrection | null;
}

/**
 * Convertit une carte affichée en rang encodé (1 = As, 10 = dix et figures).
 *
 * @param label - Carte affichée (« A », « 2 »…« 10 », « J », « Q », « K »).
 * @returns Le rang dans [1, 10].
 * @throws Error pour une carte inconnue.
 */
export function cardRank(label: string): number {
  if (label === "A") return 1;
  if (["10", "J", "Q", "K"].includes(label)) return 10;
  const rank = Number(label);
  if (!Number.isInteger(rank) || rank < 2 || rank > 9) {
    throw new Error(`Carte inconnue : « ${label} »`);
  }
  return rank;
}

/**
 * Calcule l'exemple de comptage d'un système sur une séquence de cartes.
 *
 * @param system - Définition du système (tags, IRC, pivot, side-count).
 * @param sequence - Cartes affichées, dans l'ordre.
 * @param assumptions - Position dans le sabot.
 * @returns Les lignes du tableau et le calcul du true count.
 */
export function buildCountingExample(
  system: SystemDefinition,
  sequence: readonly string[] = EXAMPLE_SEQUENCE,
  assumptions: ExampleAssumptions = DEFAULT_ASSUMPTIONS,
): CountingExample {
  if (assumptions.decksRemaining <= 0) {
    throw new Error(`Jeux restants invalides : ${assumptions.decksRemaining}`);
  }
  // IRC exporté par le moteur pour le sabot simulé (−20 pour KO en 6 jeux)
  const initialRunning = system.initial_running_count;
  let running = initialRunning;
  const rows = sequence.map((card) => {
    const tag = system.tags[cardRank(card) - 1];
    running += tag;
    return { card, tag, running };
  });
  const { decksRemaining } = assumptions;
  const trueCount = system.balanced ? running / decksRemaining : null;
  const pivot = system.pivot === null ? null : { value: system.pivot, reached: running >= system.pivot };
  return {
    initialRunning,
    rows,
    finalRunning: running,
    decksRemaining,
    trueCount,
    pivot,
    aceCorrection: aceCorrection(system, sequence, assumptions, trueCount),
  };
}

function aceCorrection(
  system: SystemDefinition,
  sequence: readonly string[],
  assumptions: ExampleAssumptions,
  trueCount: number | null,
): AceCorrection | null {
  if (!system.ace_side_count || system.ace_factor === null || trueCount === null) {
    return null;
  }
  const acesSeen = assumptions.acesSeenBefore + sequence.filter((c) => c === "A").length;
  const acesRemaining = 4 * assumptions.nDecks - acesSeen;
  const acesExpected = 4 * assumptions.decksRemaining;
  const acesRich = acesRemaining - acesExpected;
  const correction = (system.ace_factor * acesRich) / assumptions.decksRemaining;
  return {
    acesSeen,
    acesRemaining,
    acesExpected,
    acesRich,
    aceFactor: system.ace_factor,
    correction,
    bettingCount: trueCount + correction,
  };
}
