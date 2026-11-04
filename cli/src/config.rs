//! Lecture et validation du fichier de configuration TOML.

// Importation des modules standard
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// Importation des crates externes
use anyhow::{bail, Context, Result};
use serde::Deserialize;

// Importation des modules internes
use engine::betting::{BankrollPolicy, BankrollSpec};
use engine::counters::{system_by_id, CountSystem, DEFAULT_ACE_FACTOR};
use engine::deviations::DeviationLevel;
use engine::rules::Rules;

/// Configuration complète d'un lancement du CLI.
///
/// Attributs principaux :
/// * `rules` - règles communes (la pénétration est surchargée par config) ;
/// * `systems` - identifiants des systèmes simulés ;
/// * `configs` - configurations publiées (une entrée du sélecteur du dashboard).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfig {
    /// Seed maîtresse : toutes les seeds de run en dérivent (CRN).
    pub master_seed: u64,
    /// Threads du pool rayon.
    pub n_threads: usize,
    /// Nombre de runs (une seed = un run).
    pub n_runs: usize,
    /// Mains par run.
    pub n_hands_per_run: u64,
    /// Mains entre deux points des courbes d'équité.
    pub equity_sample_interval: u64,
    /// Poids du side-count des As.
    #[serde(default = "default_ace_factor")]
    pub ace_factor: f64,
    /// Bankroll de référence du risque de ruine (unités).
    #[serde(default = "default_ror_bankroll")]
    pub ror_bankroll_units: f64,
    /// Fréquence minimale d'un bin de compte exporté.
    #[serde(default = "default_tc_min_frequency")]
    pub tc_min_frequency: f64,
    /// Mains minimales pour qu'un bin serve à l'estimation Kelly.
    #[serde(default = "default_kelly_min_hands")]
    pub kelly_min_hands: u64,
    /// Largeur des bins de l'histogramme des drawdowns (en %).
    #[serde(default = "default_drawdown_bin_width")]
    pub drawdown_bin_width: f64,
    /// Dossier des fichiers Parquet.
    pub parquet_dir: PathBuf,
    /// Dossier des fichiers JSON du dashboard.
    pub json_dir: PathBuf,
    /// Règles communes.
    #[serde(default)]
    pub rules: Rules,
    /// Systèmes simulés.
    pub systems: Vec<String>,
    /// Configurations publiées.
    pub configs: Vec<PublishedConfig>,
}

/// Une configuration publiée (une entrée du sélecteur du dashboard).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedConfig {
    /// Identifiant stable (`s15-i18f4`, …).
    pub id: String,
    /// Libellé affiché.
    pub label: String,
    /// Mise maximale de la rampe (unités).
    pub spread: f64,
    /// Décalage de la rampe.
    pub ramp_offset: f64,
    /// Niveau de déviations.
    pub deviations: DeviationLevel,
    /// Pénétration du sabot.
    pub penetration: f64,
    /// Politique de bankroll du modèle B.
    pub bankroll_policy: BankrollPolicyConfig,
    /// Horizon du modèle B en mains (défaut : `n_hands_per_run`).
    ///
    /// Une politique qui ruine en quelques milliers de mains a besoin d'un horizon
    /// court et d'un échantillonnage fin pour que la courbe soit lisible.
    #[serde(default)]
    pub bankroll_hands: Option<u64>,
    /// Pas d'échantillonnage du modèle B (défaut : `equity_sample_interval`).
    #[serde(default)]
    pub bankroll_sample_interval: Option<u64>,
}

impl PublishedConfig {
    /// Horizon effectif du modèle B.
    pub fn bankroll_horizon(&self, global: &RunConfig) -> (u64, u64) {
        (
            self.bankroll_hands.unwrap_or(global.n_hands_per_run),
            self.bankroll_sample_interval
                .unwrap_or(global.equity_sample_interval),
        )
    }
}

/// Politique de bankroll telle qu'écrite dans le TOML.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BankrollPolicyConfig {
    /// Politique (`kind = "flat_pct" | "count_proportional" | "fractional_kelly"`).
    #[serde(flatten)]
    pub policy: BankrollPolicy,
    /// Bankroll initiale (unités).
    #[serde(default = "default_initial_bankroll")]
    pub initial_bankroll: f64,
    /// Mise minimale.
    #[serde(default = "default_min_bet")]
    pub min_bet: f64,
    /// Mise maximale.
    #[serde(default = "default_max_bet")]
    pub max_bet: f64,
}

fn default_ace_factor() -> f64 {
    DEFAULT_ACE_FACTOR
}
fn default_ror_bankroll() -> f64 {
    1000.0
}
fn default_tc_min_frequency() -> f64 {
    0.0005
}
fn default_kelly_min_hands() -> u64 {
    100_000
}
fn default_drawdown_bin_width() -> f64 {
    5.0
}
fn default_initial_bankroll() -> f64 {
    1000.0
}
fn default_min_bet() -> f64 {
    1.0
}
fn default_max_bet() -> f64 {
    100.0
}

impl BankrollPolicyConfig {
    /// Convertit en paramètres du moteur.
    pub fn to_spec(self) -> BankrollSpec {
        BankrollSpec {
            policy: self.policy,
            initial_bankroll: self.initial_bankroll,
            min_bet: self.min_bet,
            max_bet: self.max_bet,
        }
    }

    /// Description courte de la politique, exportée dans `configs.json`.
    pub fn describe(&self) -> String {
        match self.policy {
            BankrollPolicy::FlatPct { pct } => format!("{} % plat", fr_number(100.0 * pct)),
            BankrollPolicy::CountProportional { unit_pct } => {
                format!(
                    "proportionnelle au compte (unité {} %)",
                    fr_number(100.0 * unit_pct)
                )
            }
            BankrollPolicy::FractionalKelly { k } if (k - 0.5).abs() < 1e-12 => {
                "demi-Kelly".to_string()
            }
            BankrollPolicy::FractionalKelly { k } if (k - 0.25).abs() < 1e-12 => {
                "quart de Kelly".to_string()
            }
            BankrollPolicy::FractionalKelly { k } if (k - 1.0).abs() < 1e-12 => {
                "Kelly plein".to_string()
            }
            BankrollPolicy::FractionalKelly { k } => format!("Kelly × {}", fr_number(k)),
        }
    }
}

/// Formate un nombre avec virgule décimale, sans zéros inutiles.
fn fr_number(x: f64) -> String {
    let s = format!("{x:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.replace('.', ",")
}

impl RunConfig {
    /// Lit et valide un fichier de configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - Chemin du fichier TOML.
    ///
    /// # Returns
    ///
    /// La configuration validée, ou une erreur contextualisée.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("lecture de la configuration {}", path.display()))?;
        let config: RunConfig =
            toml::from_str(&text).with_context(|| format!("analyse TOML de {}", path.display()))?;
        config
            .validate()
            .with_context(|| format!("configuration invalide : {}", path.display()))?;
        Ok(config)
    }

    /// Systèmes résolus dans l'ordre du fichier.
    ///
    /// # Returns
    ///
    /// Les définitions du moteur, ou une erreur pour un identifiant inconnu.
    pub fn resolved_systems(&self) -> Result<Vec<&'static CountSystem>> {
        self.systems
            .iter()
            .map(|id| system_by_id(id).with_context(|| format!("système inconnu : {id}")))
            .collect()
    }

    /// Règles d'une configuration publiée (pénétration surchargée).
    pub fn rules_for(&self, published: &PublishedConfig) -> Rules {
        Rules {
            penetration: published.penetration,
            ..self.rules
        }
    }

    fn validate(&self) -> Result<()> {
        if self.n_runs == 0 || self.n_hands_per_run == 0 || self.n_threads == 0 {
            bail!("n_runs, n_hands_per_run et n_threads doivent être > 0");
        }
        if self.equity_sample_interval == 0
            || !self
                .n_hands_per_run
                .is_multiple_of(self.equity_sample_interval)
        {
            bail!(
                "equity_sample_interval ({}) doit diviser n_hands_per_run ({})",
                self.equity_sample_interval,
                self.n_hands_per_run
            );
        }
        self.rules.validate().map_err(anyhow::Error::msg)?;
        self.resolved_systems()?;
        if self.configs.is_empty() {
            bail!("aucune configuration [[configs]]");
        }
        let mut ids = HashSet::new();
        for published in &self.configs {
            if !ids.insert(published.id.as_str()) {
                bail!("identifiant de config dupliqué : {}", published.id);
            }
            if published.spread < 1.0 {
                bail!("config {} : spread {} < 1", published.id, published.spread);
            }
            let (hands, interval) = published.bankroll_horizon(self);
            if hands == 0 || interval == 0 || !hands.is_multiple_of(interval) {
                bail!(
                    "config {} : le pas {interval} doit diviser l'horizon du modèle B {hands}",
                    published.id
                );
            }
            self.rules_for(published)
                .validate()
                .map_err(|e| anyhow::anyhow!("config {} : {e}", published.id))?;
            published
                .bankroll_policy
                .to_spec()
                .validate()
                .map_err(|e| anyhow::anyhow!("config {} : {e}", published.id))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        master_seed = 1
        n_threads = 2
        n_runs = 4
        n_hands_per_run = 1000
        equity_sample_interval = 100
        parquet_dir = "data"
        json_dir = "out"
        systems = ["hi-lo", "ko"]

        [[configs]]
        id = "s15-i18f4"
        label = "test"
        spread = 15
        ramp_offset = 1
        deviations = "i18_fab4"
        penetration = 0.75
        bankroll_policy = { kind = "fractional_kelly", k = 0.5 }
    "#;

    #[test]
    fn parses_sample_config() {
        let config: RunConfig = toml::from_str(SAMPLE).expect("TOML valide");
        assert!(config.validate().is_ok());
        assert_eq!(config.rules, Rules::default());
        assert_eq!(config.configs[0].deviations, DeviationLevel::I18Fab4);
        assert_eq!(config.configs[0].bankroll_policy.describe(), "demi-Kelly");
        assert_eq!(config.resolved_systems().expect("systèmes").len(), 2);
    }

    #[test]
    fn rejects_unknown_system_and_bad_interval() {
        let bad = SAMPLE.replace("\"ko\"", "\"inconnu\"");
        let config: RunConfig = toml::from_str(&bad).expect("TOML valide");
        assert!(config.validate().is_err());
        let bad = SAMPLE.replace(
            "equity_sample_interval = 100",
            "equity_sample_interval = 300",
        );
        let config: RunConfig = toml::from_str(&bad).expect("TOML valide");
        assert!(config.validate().is_err());
    }

    #[test]
    fn describes_policies() {
        let flat = BankrollPolicyConfig {
            policy: BankrollPolicy::FlatPct { pct: 0.1 },
            initial_bankroll: 1000.0,
            min_bet: 1.0,
            max_bet: 100.0,
        };
        assert_eq!(flat.describe(), "10 % plat");
    }
}
