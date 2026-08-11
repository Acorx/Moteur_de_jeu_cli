use serde::Serialize;

use crate::Result;

pub const SCHEMAS: &[(&str, &str)] = &[
    (
        "agent-request",
        include_str!("../schemas/agent-request-v1.json"),
    ),
    (
        "agent-response",
        include_str!("../schemas/agent-response-v1.json"),
    ),
    ("snapshot", include_str!("../schemas/snapshot-v1.json")),
    ("events", include_str!("../schemas/events-v1.json")),
    ("replay-v2", include_str!("../schemas/replay-v2.json")),
    ("scenario", include_str!("../schemas/scenario-v1.json")),
    (
        "scenario-report",
        include_str!("../schemas/scenario-report-v1.json"),
    ),
    ("telemetry", include_str!("../schemas/telemetry-v1.json")),
    (
        "capture-manifest",
        include_str!("../schemas/capture-manifest-v1.json"),
    ),
    (
        "capability-policy",
        include_str!("../schemas/capability-policy-v1.json"),
    ),
    (
        "capture-multi",
        include_str!("../schemas/capture-multi-v1.json"),
    ),
    (
        "capture-views",
        include_str!("../schemas/capture-views-v1.json"),
    ),
    ("scene", include_str!("../schemas/scene-v1.json")),
    ("assets", include_str!("../schemas/assets-v1.json")),
    (
        "visual-diff",
        include_str!("../schemas/visual-diff-v1.json"),
    ),
];

#[derive(Serialize)]
struct SchemaItem {
    name: &'static str,
    id: String,
}

pub fn list() -> Result<String> {
    let mut items = Vec::new();
    for (name, source) in SCHEMAS {
        let value: serde_json::Value = serde_json::from_str(source)
            .map_err(|error| format!("schéma interne {name} invalide: {error}"))?;
        items.push(SchemaItem {
            name,
            id: value["$id"].as_str().unwrap_or_default().to_owned(),
        });
    }
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "aetherion.schema-list/v1",
        "schemas": items
    }))
    .map_err(|error| format!("sérialisation de la liste des schémas: {error}").into())
}

pub fn show(name: &str) -> Result<String> {
    let source = SCHEMAS
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, source)| *source)
        .ok_or_else(|| format!("schéma inconnu: {name}"))?;
    let value: serde_json::Value = serde_json::from_str(source)
        .map_err(|error| format!("schéma interne {name} invalide: {error}"))?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("sérialisation du schéma {name}: {error}").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_published_schemas_are_valid_and_identified() {
        for (name, source) in SCHEMAS {
            let value: serde_json::Value =
                serde_json::from_str(source).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(
                value["$id"].as_str().is_some_and(|id| !id.is_empty()),
                "{name}"
            );
            assert_eq!(
                value["$schema"],
                "https://json-schema.org/draft/2020-12/schema"
            );
        }
    }
}
