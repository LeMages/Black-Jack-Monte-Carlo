//! Écriture des fichiers JSON consommés par le dashboard.

// Importation des modules standard
use std::path::Path;

// Importation des crates externes
use anyhow::{Context, Result};
use serde::Serialize;

/// Écrit une valeur sérialisable dans `<dir>/<name>.json` (JSON compact).
///
/// # Arguments
///
/// * `dir` - Dossier de sortie (créé si besoin).
/// * `name` - Nom du fichier sans extension.
/// * `value` - Valeur à sérialiser.
///
/// # Returns
///
/// `Ok(())` ou une erreur contextualisée.
pub fn write_json<T: Serialize + ?Sized>(dir: &Path, name: &str, value: &T) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("création de {}", dir.display()))?;
    let path = dir.join(format!("{name}.json"));
    let mut text =
        serde_json::to_string(value).with_context(|| format!("sérialisation JSON de {name}"))?;
    text.push('\n');
    std::fs::write(&path, text).with_context(|| format!("écriture de {}", path.display()))
}
