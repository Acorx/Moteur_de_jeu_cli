use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::Result;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct DiffReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub left: String,
    pub right: String,
    pub differences: Vec<Difference>,
}

#[derive(Debug, Serialize)]
pub struct Difference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<u64>,
    pub operation: &'static str,
    pub field: String,
    #[serde(rename = "old_value")]
    pub left: Value,
    #[serde(rename = "new_value")]
    pub right: Value,
}

pub fn compare_values(left: &Value, right: &Value) -> Value {
    let tick = left
        .get("tick")
        .and_then(Value::as_u64)
        .or_else(|| right.get("tick").and_then(Value::as_u64));
    let mut differences = Vec::new();
    compare_value(left, right, "$", tick, None, &mut differences);
    serde_json::json!({
        "schema": "aetherion.semantic-diff/v1",
        "status": if differences.is_empty() { "identical" } else { "different" },
        "operations": differences
    })
}

pub fn compare_files(left: &Path, right: &Path) -> Result<DiffReport> {
    let left_value = read_json(left)?;
    let right_value = read_json(right)?;
    let tick = left_value
        .get("tick")
        .and_then(Value::as_u64)
        .or_else(|| right_value.get("tick").and_then(Value::as_u64));
    let mut differences = Vec::new();
    compare_value(&left_value, &right_value, "$", tick, None, &mut differences);
    Ok(DiffReport {
        schema: "aetherion.diff/v1",
        status: if differences.is_empty() {
            "identical"
        } else {
            "different"
        },
        left: left.to_string_lossy().replace('\\', "/"),
        right: right.to_string_lossy().replace('\\', "/"),
        differences,
    })
}

pub fn outcome(report: DiffReport) -> Result<String> {
    let different = !report.differences.is_empty();
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("sérialisation du diff: {error}"))?;
    if different {
        Err(AppError::outcome("les documents sont différents", 1, json))
    } else {
        Ok(json)
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes =
        fs::read(path).map_err(|error| format!("lecture de {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("JSON invalide dans {}: {error}", path.display()).into())
}

fn compare_value(
    left: &Value,
    right: &Value,
    path: &str,
    tick: Option<u64>,
    entity_id: Option<u64>,
    output: &mut Vec<Difference>,
) {
    if path == "$.entities" {
        compare_entities(left, right, tick, output);
        return;
    }
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            let keys: BTreeSet<_> = a.keys().chain(b.keys()).collect();
            for key in keys {
                let child_path = format!("{path}.{key}");
                compare_value(
                    a.get(key).unwrap_or(&Value::Null),
                    b.get(key).unwrap_or(&Value::Null),
                    &child_path,
                    tick,
                    entity_id,
                    output,
                );
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            for index in 0..a.len().max(b.len()) {
                compare_value(
                    a.get(index).unwrap_or(&Value::Null),
                    b.get(index).unwrap_or(&Value::Null),
                    &format!("{path}[{index}]"),
                    tick,
                    entity_id,
                    output,
                );
            }
        }
        _ if left != right => output.push(Difference {
            tick,
            entity_id,
            operation: if left.is_null() {
                "add"
            } else if right.is_null() {
                "remove"
            } else {
                "replace"
            },
            field: path.into(),
            left: left.clone(),
            right: right.clone(),
        }),
        _ => {}
    }
}

fn compare_entities(left: &Value, right: &Value, tick: Option<u64>, output: &mut Vec<Difference>) {
    fn to_map(value: &Value) -> BTreeMap<u64, &Value> {
        value
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|entity| {
                entity
                    .get("id")
                    .and_then(Value::as_u64)
                    .map(|id| (id, entity))
            })
            .collect()
    }
    let left_entities = to_map(left);
    let right_entities = to_map(right);
    let ids: BTreeSet<_> = left_entities
        .keys()
        .chain(right_entities.keys())
        .copied()
        .collect();
    for id in ids {
        compare_value(
            left_entities.get(&id).copied().unwrap_or(&Value::Null),
            right_entities.get(&id).copied().unwrap_or(&Value::Null),
            &format!("$.entities[id={id}]"),
            tick,
            Some(id),
            output,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_entity_field() {
        let left = serde_json::json!({"tick": 2, "entities": [{"id": 7, "position": {"x": 1}}]});
        let right = serde_json::json!({"tick": 2, "entities": [{"id": 7, "position": {"x": 9}}]});
        let mut differences = Vec::new();
        compare_value(&left, &right, "$", Some(2), None, &mut differences);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].entity_id, Some(7));
        assert_eq!(differences[0].tick, Some(2));
    }
}
