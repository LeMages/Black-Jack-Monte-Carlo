//! CLI batch : lit une configuration TOML, lance les simulations et écrit les
//! résultats en Parquet (`data/`) et en JSON (`dashboard/public/data/`).
//!
//! Usage : `cargo run --release -p cli -- configs/full.toml`.

mod config;
mod export;
mod run_plan;
mod table;
mod write_json;
mod write_parquet;

// Importation des modules standard
use std::path::PathBuf;

// Importation des crates externes
use anyhow::{bail, Result};

// Importation des modules internes
use config::RunConfig;
use write_json::write_json;
use write_parquet::write_parquet;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [path] = args.as_slice() else {
        bail!(
            "usage : cli <config.toml> (reçu {} argument(s))",
            args.len()
        );
    };
    let config = RunConfig::load(&PathBuf::from(path))?;
    eprintln!(
        "Configuration {path} : {} systèmes × {} configs, {} runs × {} mains, {} threads",
        config.systems.len(),
        config.configs.len(),
        config.n_runs,
        config.n_hands_per_run,
        config.n_threads
    );

    // ------------------------------------------------------------------
    // 1. Simulations
    // ------------------------------------------------------------------
    let results = run_plan::execute(&config)?;

    // ------------------------------------------------------------------
    // 2. Tables du contrat : Parquet (canonique) et JSON (dashboard)
    // ------------------------------------------------------------------
    let tables = [
        export::metrics_table(&results.cells)?,
        export::equity_table(&results.cells)?,
        export::tc_table(&results.cells)?,
        export::drawdowns_table(&results.cells)?,
        export::drawdown_histogram_table(&results.cells, config.drawdown_bin_width)?,
    ];
    for table in &tables {
        write_parquet(&config.parquet_dir, table)?;
        write_json(&config.json_dir, &table.name, &table.to_json_rows())?;
    }

    // ------------------------------------------------------------------
    // 3. Documents annexes
    // ------------------------------------------------------------------
    write_json(&config.json_dir, "configs", &export::configs_doc(&config))?;
    write_json(
        &config.json_dir,
        "baseline",
        &export::baseline_doc(&results),
    )?;
    write_json(&config.json_dir, "systems", &export::systems_doc(&config)?)?;
    write_json(
        &config.json_dir,
        "meta",
        &export::meta_doc(&config, &results),
    )?;

    eprintln!(
        "Terminé : {} mains simulées en {:.1} s ({:.1} M mains/s). Sorties : {} et {}",
        results.total_hands,
        results.elapsed_s,
        results.total_hands as f64 / results.elapsed_s.max(1e-9) / 1e6,
        config.parquet_dir.display(),
        config.json_dir.display()
    );
    Ok(())
}
