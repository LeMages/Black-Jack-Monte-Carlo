/**
 * Types du contrat de données produit par le CLI (`dashboard/public/data/*.json`).
 *
 * Ils reflètent exactement les colonnes des tables Parquet/JSON. Toute
 * modification du CLI doit être répercutée ici (et dans `load_data.ts`).
 */

/** Identifiant d'un système de comptage. */
export type SystemId =
  | "hi-lo"
  | "ko"
  | "hi-opt-ii"
  | "omega-ii"
  | "wong-halves"
  | "zen";

/** Niveau de déviations exporté. */
export type DeviationLabel = "none" | "insurance" | "i18" | "i18_fab4";

/** Ligne de `metrics.json` : une par (système × config). */
export interface MetricsRow {
  system: SystemId;
  config_id: string;
  bet_model: string;
  deviations: DeviationLabel;
  advantage_pct: number;
  ev_per_100: number;
  std_per_hand: number;
  /** `null` quand l'espérance est négative ou nulle. */
  n0: number | null;
  risk_of_ruin: number;
  kelly_fraction: number;
  avg_bet: number;
  win_rate: number;
  n_hands_simulated: number;
}

/** Ligne de `equity_curves.json` (format long). */
export interface EquityRow {
  system: SystemId;
  config_id: string;
  hand_index: number;
  p05: number;
  p25: number;
  p50: number;
  p75: number;
  p95: number;
  mean: number;
  frac_ruined: number;
}

/** Ligne de `tc_distribution.json`. */
export interface TcRow {
  system: SystemId;
  config_id: string;
  true_count: number;
  frequency: number;
  advantage_at_tc: number;
  avg_bet_at_tc: number;
}

/** Ligne de `drawdowns.json` : une valeur par run. */
export interface DrawdownRow {
  system: SystemId;
  config_id: string;
  max_drawdown_pct: number;
}

/** Ligne de `drawdown_histogram.json` (binning fait à l'export). */
export interface DrawdownBinRow {
  system: SystemId;
  config_id: string;
  bin_start: number;
  bin_end: number;
  count: number;
}

/** Entrée de `configs.json`. */
export interface ConfigEntry {
  id: string;
  label: string;
  bankroll_policy: string;
  spread: number;
  ramp_offset: number;
  deviations: DeviationLabel;
  penetration: number;
  bankroll_hands: number;
  bankroll_sample_interval: number;
  initial_bankroll: number;
  min_bet: number;
  max_bet: number;
}

/** Contenu de `baseline.json` : basic strategy seule, mise plate. */
export interface Baseline extends Omit<MetricsRow, "system" | "config_id"> {
  system: "basic";
  config_id: "baseline";
}

/** Entrée de `systems.json` : définition exacte utilisée par le moteur. */
export interface SystemDefinition {
  id: SystemId;
  name: string;
  /** Tags par rang : index 0 = As, 1 = 2, …, 9 = dix. */
  tags: number[];
  balanced: boolean;
  level: number;
  ace_side_count: boolean;
  betting_correlation: number;
  playing_efficiency: number;
  insurance_correlation: number;
  initial_running_count: number;
  pivot: number | null;
  index_scale: number;
  ace_factor: number | null;
}

/** Règles simulées, reprises de `meta.json`. */
export interface Rules {
  n_decks: number;
  penetration: number;
  dealer_hits_soft_17: boolean;
  blackjack_payout: number;
  double_after_split: boolean;
  double_any_two: boolean;
  max_splits: number;
  resplit_aces: boolean;
  hit_split_aces: boolean;
  late_surrender: boolean;
  dealer_peeks: boolean;
}

/** Contenu de `meta.json` : paramètres du lancement. */
export interface Meta {
  master_seed: number;
  n_runs: number;
  n_hands_per_run: number;
  n_threads: number;
  equity_sample_interval: number;
  ror_bankroll_units: number;
  total_hands_simulated: number;
  elapsed_s: number;
  rules: Rules;
}

/** Ensemble des données chargées par le dashboard. */
export interface DataBundle {
  metrics: MetricsRow[];
  equityCurves: EquityRow[];
  tcDistribution: TcRow[];
  drawdownHistogram: DrawdownBinRow[];
  configs: ConfigEntry[];
  baseline: Baseline;
  systems: SystemDefinition[];
  meta: Meta;
}
