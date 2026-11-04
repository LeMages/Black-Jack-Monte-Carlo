//! Écriture d'une [`Table`] au format Parquet (source canonique, dossier `data/`).

// Importation des modules standard
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

// Importation des crates externes
use anyhow::{Context, Result};
use arrow::array::{ArrayRef, Float64Array, Int32Array, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

// Importation des modules internes
use crate::table::{Column, Table};

/// Écrit une table dans `<dir>/<name>.parquet`.
///
/// # Arguments
///
/// * `dir` - Dossier de sortie (créé si besoin).
/// * `table` - Table à écrire.
///
/// # Returns
///
/// `Ok(())` ou une erreur contextualisée.
pub fn write_parquet(dir: &Path, table: &Table) -> Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("création de {}", dir.display()))?;
    let path = dir.join(format!("{}.parquet", table.name));
    let (fields, arrays): (Vec<Field>, Vec<ArrayRef>) = table
        .columns
        .iter()
        .map(|(name, column)| arrow_column(name, column))
        .unzip();
    let schema = Arc::new(Schema::new(fields));
    let batch = RecordBatch::try_new(schema.clone(), arrays)
        .with_context(|| format!("construction du lot Arrow de {}", table.name))?;
    let file = File::create(&path).with_context(|| format!("création de {}", path.display()))?;
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))
        .with_context(|| format!("ouverture du writer Parquet {}", path.display()))?;
    writer
        .write(&batch)
        .with_context(|| format!("écriture de {}", path.display()))?;
    writer
        .close()
        .with_context(|| format!("finalisation de {}", path.display()))?;
    Ok(())
}

fn arrow_column(name: &str, column: &Column) -> (Field, ArrayRef) {
    match column {
        Column::Str(v) => (
            Field::new(name, DataType::Utf8, false),
            Arc::new(StringArray::from(v.clone())) as ArrayRef,
        ),
        Column::F64(v) => (
            Field::new(name, DataType::Float64, false),
            Arc::new(Float64Array::from(v.clone())) as ArrayRef,
        ),
        Column::OptF64(v) => (
            Field::new(name, DataType::Float64, true),
            Arc::new(Float64Array::from(v.clone())) as ArrayRef,
        ),
        Column::I32(v) => (
            Field::new(name, DataType::Int32, false),
            Arc::new(Int32Array::from(v.clone())) as ArrayRef,
        ),
        Column::U64(v) => (
            Field::new(name, DataType::UInt64, false),
            Arc::new(UInt64Array::from(v.clone())) as ArrayRef,
        ),
    }
}
