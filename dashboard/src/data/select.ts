/**
 * Sélection de lignes dans les données chargées (filtrage, pas de calcul).
 */

// Importation des modules internes
import type {
  DataBundle,
  DrawdownBinRow,
  EquityRow,
  MetricsRow,
  SystemDefinition,
  SystemId,
  TcRow,
} from "../types/data";

/**
 * Métriques d'une config, dans l'ordre des systèmes de `systems.json`.
 *
 * @param data - Données chargées.
 * @param configId - Identifiant de config.
 * @returns Une ligne par système présent.
 */
export function metricsForConfig(data: DataBundle, configId: string): MetricsRow[] {
  return data.systems
    .map((s) => data.metrics.find((m) => m.system === s.id && m.config_id === configId))
    .filter((m): m is MetricsRow => m !== undefined);
}

/**
 * Métriques d'un système pour une config.
 *
 * @param data - Données chargées.
 * @param system - Identifiant du système.
 * @param configId - Identifiant de config.
 * @returns La ligne, ou `undefined` si absente.
 */
export function metricsFor(data: DataBundle, system: SystemId, configId: string): MetricsRow | undefined {
  return data.metrics.find((m) => m.system === system && m.config_id === configId);
}

/**
 * Courbe d'équité d'un système pour une config, triée par main.
 */
export function equityFor(data: DataBundle, system: SystemId, configId: string): EquityRow[] {
  return data.equityCurves
    .filter((r) => r.system === system && r.config_id === configId)
    .sort((a, b) => a.hand_index - b.hand_index);
}

/**
 * Distribution par true count d'un système pour une config, triée par bin.
 */
export function tcFor(data: DataBundle, system: SystemId, configId: string): TcRow[] {
  return data.tcDistribution
    .filter((r) => r.system === system && r.config_id === configId)
    .sort((a, b) => a.true_count - b.true_count);
}

/**
 * Histogramme des drawdowns d'un système pour une config.
 */
export function drawdownBinsFor(data: DataBundle, system: SystemId, configId: string): DrawdownBinRow[] {
  return data.drawdownHistogram
    .filter((r) => r.system === system && r.config_id === configId)
    .sort((a, b) => a.bin_start - b.bin_start);
}

/**
 * Définition d'un système par identifiant.
 */
export function systemById(data: DataBundle, id: string): SystemDefinition | undefined {
  return data.systems.find((s) => s.id === id);
}
