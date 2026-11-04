//! Table en colonnes, représentation commune aux sorties Parquet et JSON.
//!
//! Chaque fichier du contrat de données est construit une seule fois sous forme
//! de [`Table`], puis écrit dans les deux formats : les schémas ne peuvent donc
//! pas diverger entre Parquet et JSON.

// Importation des crates externes
use anyhow::{bail, Result};
use serde_json::{Map, Number, Value};

/// Colonne typée.
#[derive(Debug, Clone, PartialEq)]
pub enum Column {
    /// Texte.
    Str(Vec<String>),
    /// Flottant 64 bits.
    F64(Vec<f64>),
    /// Flottant 64 bits optionnel (`null` en JSON).
    OptF64(Vec<Option<f64>>),
    /// Entier signé 32 bits.
    I32(Vec<i32>),
    /// Entier non signé 64 bits.
    U64(Vec<u64>),
}

impl Column {
    /// Nombre de lignes de la colonne.
    pub fn len(&self) -> usize {
        match self {
            Column::Str(v) => v.len(),
            Column::F64(v) => v.len(),
            Column::OptF64(v) => v.len(),
            Column::I32(v) => v.len(),
            Column::U64(v) => v.len(),
        }
    }

    /// Valeur JSON de la ligne `row`.
    fn json_value(&self, row: usize) -> Value {
        match self {
            Column::Str(v) => Value::String(v[row].clone()),
            Column::F64(v) => float_value(v[row]),
            Column::OptF64(v) => v[row].map_or(Value::Null, float_value),
            Column::I32(v) => Value::from(v[row]),
            Column::U64(v) => Value::from(v[row]),
        }
    }
}

/// Un flottant non fini n'a pas de représentation JSON : il devient `null`.
fn float_value(x: f64) -> Value {
    Number::from_f64(x).map_or(Value::Null, Value::Number)
}

/// Table nommée, colonnes de même longueur.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// Nom du fichier sans extension (`metrics`, …).
    pub name: String,
    /// Colonnes dans l'ordre du contrat.
    pub columns: Vec<(String, Column)>,
}

impl Table {
    /// Construit une table en vérifiant la cohérence des longueurs.
    ///
    /// # Arguments
    ///
    /// * `name` - Nom du fichier sans extension.
    /// * `columns` - Colonnes `(nom, valeurs)`.
    ///
    /// # Returns
    ///
    /// La table, ou une erreur si les colonnes n'ont pas la même longueur.
    pub fn new(name: &str, columns: Vec<(&str, Column)>) -> Result<Self> {
        let Some((_, first)) = columns.first() else {
            bail!("table {name} sans colonne");
        };
        let n = first.len();
        if let Some((col, _)) = columns.iter().find(|(_, c)| c.len() != n) {
            bail!("table {name} : la colonne {col} n'a pas {n} lignes");
        }
        Ok(Self {
            name: name.to_string(),
            columns: columns
                .into_iter()
                .map(|(n, c)| (n.to_string(), c))
                .collect(),
        })
    }

    /// Nombre de lignes.
    pub fn n_rows(&self) -> usize {
        self.columns.first().map_or(0, |(_, c)| c.len())
    }

    /// Lignes sous forme d'objets JSON, clés dans l'ordre des colonnes.
    pub fn to_json_rows(&self) -> Value {
        let rows = (0..self.n_rows())
            .map(|row| {
                let mut obj = Map::new();
                for (name, column) in &self.columns {
                    obj.insert(name.clone(), column.json_value(row));
                }
                Value::Object(obj)
            })
            .collect();
        Value::Array(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_json_rows() {
        let table = Table::new(
            "t",
            vec![
                ("system", Column::Str(vec!["hi-lo".into(), "ko".into()])),
                ("n0", Column::OptF64(vec![Some(1.5), None])),
                ("bin", Column::I32(vec![-1, 2])),
            ],
        )
        .expect("table valide");
        let json = table.to_json_rows();
        assert_eq!(json[0]["system"], "hi-lo");
        assert_eq!(json[1]["n0"], Value::Null);
        assert_eq!(json[0]["bin"], -1);
    }

    #[test]
    fn rejects_ragged_columns() {
        let res = Table::new(
            "t",
            vec![
                ("a", Column::F64(vec![1.0])),
                ("b", Column::F64(vec![1.0, 2.0])),
            ],
        );
        assert!(res.is_err());
    }
}
