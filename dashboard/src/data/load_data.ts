/**
 * Chargement et validation des JSON produits par le CLI.
 *
 * Les fichiers sont lus tels quels : aucune statistique n'est recalculée ici.
 * La validation vérifie la forme (colonnes présentes et typées) pour échouer
 * tôt avec un message explicite si le contrat de données a dérivé.
 */

// Importation des modules internes
import type {
  Baseline,
  ConfigEntry,
  DataBundle,
  DrawdownBinRow,
  EquityRow,
  Meta,
  MetricsRow,
  SystemDefinition,
  SystemId,
  TcRow,
} from "../types/data";

/** Identifiants de systèmes acceptés, dans l'ordre d'affichage. */
export const SYSTEM_IDS: readonly SystemId[] = [
  "hi-lo",
  "ko",
  "hi-opt-ii",
  "omega-ii",
  "wong-halves",
  "zen",
];

type FieldKind = "string" | "number" | "nullable-number" | "boolean" | "system";
type Schema = Record<string, FieldKind>;

const METRICS_SCHEMA: Schema = {
  system: "system",
  config_id: "string",
  bet_model: "string",
  deviations: "string",
  advantage_pct: "number",
  ev_per_100: "number",
  std_per_hand: "number",
  n0: "nullable-number",
  risk_of_ruin: "number",
  kelly_fraction: "number",
  avg_bet: "number",
  win_rate: "number",
  n_hands_simulated: "number",
};

const EQUITY_SCHEMA: Schema = {
  system: "system",
  config_id: "string",
  hand_index: "number",
  p05: "number",
  p25: "number",
  p50: "number",
  p75: "number",
  p95: "number",
  mean: "number",
  frac_ruined: "number",
};

const TC_SCHEMA: Schema = {
  system: "system",
  config_id: "string",
  true_count: "number",
  frequency: "number",
  advantage_at_tc: "number",
  avg_bet_at_tc: "number",
};

const DRAWDOWN_BIN_SCHEMA: Schema = {
  system: "system",
  config_id: "string",
  bin_start: "number",
  bin_end: "number",
  count: "number",
};

const CONFIG_SCHEMA: Schema = {
  id: "string",
  label: "string",
  bankroll_policy: "string",
  spread: "number",
  ramp_offset: "number",
  deviations: "string",
  penetration: "number",
  bankroll_hands: "number",
  bankroll_sample_interval: "number",
  initial_bankroll: "number",
  min_bet: "number",
  max_bet: "number",
};

const BASELINE_SCHEMA: Schema = {
  ...METRICS_SCHEMA,
  system: "string",
};

const SYSTEM_SCHEMA: Schema = {
  id: "system",
  name: "string",
  balanced: "boolean",
  level: "number",
  ace_side_count: "boolean",
  betting_correlation: "number",
  playing_efficiency: "number",
  insurance_correlation: "number",
  initial_running_count: "number",
  pivot: "nullable-number",
  index_scale: "number",
  ace_factor: "nullable-number",
};

const META_SCHEMA: Schema = {
  master_seed: "number",
  n_runs: "number",
  n_hands_per_run: "number",
  n_threads: "number",
  equity_sample_interval: "number",
  ror_bankroll_units: "number",
  total_hands_simulated: "number",
  elapsed_s: "number",
};

/** Erreur levée quand un fichier ne respecte pas le contrat. */
export class DataContractError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "DataContractError";
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function checkField(kind: FieldKind, value: unknown): boolean {
  switch (kind) {
    case "string":
      return typeof value === "string";
    case "number":
      return typeof value === "number" && Number.isFinite(value);
    case "nullable-number":
      return value === null || (typeof value === "number" && Number.isFinite(value));
    case "boolean":
      return typeof value === "boolean";
    case "system":
      return typeof value === "string" && (SYSTEM_IDS as readonly string[]).includes(value);
  }
}

/**
 * Vérifie qu'un objet respecte un schéma de colonnes.
 *
 * @param file - Nom du fichier (pour le message d'erreur).
 * @param schema - Colonnes attendues et leur type.
 * @param value - Objet à vérifier.
 * @param where - Position dans le fichier (pour le message d'erreur).
 * @returns L'objet, typé.
 * @throws DataContractError si une colonne manque ou a le mauvais type.
 */
export function validateObject<T>(file: string, schema: Schema, value: unknown, where = "racine"): T {
  if (!isRecord(value)) {
    throw new DataContractError(`${file} (${where}) : objet attendu, reçu ${JSON.stringify(value)}`);
  }
  for (const [field, kind] of Object.entries(schema)) {
    if (!checkField(kind, value[field])) {
      throw new DataContractError(
        `${file} (${where}) : colonne « ${field} » invalide (attendu ${kind}, reçu ${JSON.stringify(value[field])})`,
      );
    }
  }
  return value as T;
}

/**
 * Vérifie un tableau de lignes.
 *
 * @param file - Nom du fichier.
 * @param schema - Colonnes attendues.
 * @param value - Contenu JSON du fichier.
 * @returns Les lignes typées.
 * @throws DataContractError si le contenu n'est pas un tableau non vide de lignes valides.
 */
export function validateRows<T>(file: string, schema: Schema, value: unknown): T[] {
  if (!Array.isArray(value) || value.length === 0) {
    throw new DataContractError(`${file} : tableau non vide attendu`);
  }
  return value.map((row, i) => validateObject<T>(file, schema, row, `ligne ${i}`));
}

function validateSystems(value: unknown): SystemDefinition[] {
  const rows = validateRows<SystemDefinition>("systems.json", SYSTEM_SCHEMA, value);
  for (const row of rows) {
    const tagsOk =
      Array.isArray(row.tags) && row.tags.length === 10 && row.tags.every((t) => typeof t === "number");
    if (!tagsOk) {
      throw new DataContractError(`systems.json : tags invalides pour ${row.id}`);
    }
  }
  return rows;
}

/**
 * Valide le contenu brut de tous les fichiers et construit le bundle typé.
 *
 * @param raw - Contenu JSON de chaque fichier, indexé par nom.
 * @returns Le bundle de données typé.
 * @throws DataContractError au premier écart au contrat.
 */
export function buildBundle(raw: Record<string, unknown>): DataBundle {
  const meta = validateObject<Meta>("meta.json", META_SCHEMA, raw.meta);
  validateObject("meta.json", { n_decks: "number", penetration: "number" }, meta.rules, "rules");
  return {
    metrics: validateRows<MetricsRow>("metrics.json", METRICS_SCHEMA, raw.metrics),
    equityCurves: validateRows<EquityRow>("equity_curves.json", EQUITY_SCHEMA, raw.equity_curves),
    tcDistribution: validateRows<TcRow>("tc_distribution.json", TC_SCHEMA, raw.tc_distribution),
    drawdownHistogram: validateRows<DrawdownBinRow>(
      "drawdown_histogram.json",
      DRAWDOWN_BIN_SCHEMA,
      raw.drawdown_histogram,
    ),
    configs: validateRows<ConfigEntry>("configs.json", CONFIG_SCHEMA, raw.configs),
    baseline: validateObject<Baseline>("baseline.json", BASELINE_SCHEMA, raw.baseline),
    systems: validateSystems(raw.systems),
    meta,
  };
}

/** Fichiers lus par le dashboard (sans extension). */
export const DATA_FILES = [
  "metrics",
  "equity_curves",
  "tc_distribution",
  "drawdown_histogram",
  "configs",
  "baseline",
  "systems",
  "meta",
] as const;

/**
 * Télécharge et valide toutes les données du dashboard.
 *
 * @param baseUrl - Préfixe des fichiers (défaut : `/data`).
 * @param fetcher - Fonction `fetch` injectable (tests).
 * @returns Le bundle de données typé.
 * @throws Error si un fichier est introuvable ou invalide.
 */
export async function loadData(baseUrl = "/data", fetcher: typeof fetch = fetch): Promise<DataBundle> {
  const entries = await Promise.all(
    DATA_FILES.map(async (name) => {
      const url = `${baseUrl}/${name}.json`;
      const response = await fetcher(url);
      if (!response.ok) {
        throw new Error(`Chargement de ${url} impossible (HTTP ${response.status})`);
      }
      return [name, (await response.json()) as unknown] as const;
    }),
  );
  return buildBundle(Object.fromEntries(entries));
}
