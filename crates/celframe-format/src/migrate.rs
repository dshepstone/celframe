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
pub(crate) const MIGRATIONS: &[Migration] = &[];

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
