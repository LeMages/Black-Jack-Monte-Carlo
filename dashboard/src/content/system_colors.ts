/**
 * Couleur fixe de chaque système, identique sur tous les graphes.
 *
 * Palette catégorielle validée (séparation daltonisme ΔE ≥ 8 entre voisins,
 * luminosités distinctes). Deux teintes passent sous 3:1 de contraste sur fond
 * blanc : l'identité n'est donc jamais portée par la couleur seule (légendes
 * textuelles, étiquettes directes, tableau récapitulatif).
 */

// Importation des modules internes
import type { SystemId } from "../types/data";

export const SYSTEM_COLORS: Record<SystemId, string> = {
  "hi-lo": "#2a78d6",
  ko: "#eb6834",
  "hi-opt-ii": "#1baf7a",
  "omega-ii": "#c98500",
  "wong-halves": "#d55181",
  zen: "#008300",
};
