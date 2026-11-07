//! Upgrades files saved with older schema versions. Migrations work on raw JSON, because the
//! current Rust types only describe the current schema.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::FormatError;

/// A file's JSON before it becomes typed data.
#[derive(Debug)]
pub struct RawDocument {
    pub manifest: Value,
    /// Scene JSON by entry name, such as `scenes/<id>.json`.
    pub scenes: BTreeMap<String, Value>,
}

/// Upgrades a document by exactly one schema version.
pub type Migration = fn(&mut RawDocument) -> Result<(), String>;

/// Index 0 upgrades version 1 to 2, index 1 upgrades 2 to 3, and so on.
pub(crate) const MIGRATIONS: &[Migration] = &[v1_to_v2];

/// Version 2 adds `art` to every stroke. Strokes saved before it existed are line art.
fn v1_to_v2(document: &mut RawDocument) -> Result<(), String> {
    for scene in document.scenes.values_mut() {
        for layer in list(scene, "layers")? {
            for drawing in list(layer, "drawings")? {
                for stroke in list(drawing, "strokes")? {
                    stroke
                        .as_object_mut()
                        .ok_or("a stroke isn't an object")?
                        .insert("art".to_owned(), Value::from("line"));
                }
            }
        }
    }
    Ok(())
}

/// The array under `key`, or an error naming it.
fn list<'a>(value: &'a mut Value, key: &str) -> Result<&'a mut Vec<Value>, String> {
    value
        .get_mut(key)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("`{key}` is missing or isn't a list"))
}

/// Runs each migration from a file's version up to `target`.
pub(crate) fn migrate(
    document: &mut RawDocument,
    from: u32,
    target: u32,
    migrations: &[Migration],
) -> Result<(), FormatError> {
    for version in from..target {
        let migration = migrations
            .get(version as usize - 1)
            .ok_or_else(|| missing_step(version))?;
        migration(document).map_err(|reason| {
            FormatError::InvalidDocument(format!(
                "couldn't upgrade this file from schema version {version}: {reason}"
            ))
        })?;
        document.manifest["schemaVersion"] = Value::from(version + 1);
    }
    Ok(())
}

fn missing_step(version: u32) -> FormatError {
    FormatError::InvalidDocument(format!("no upgrade exists from schema version {version}"))
}
