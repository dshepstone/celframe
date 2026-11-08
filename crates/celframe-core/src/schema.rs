//! Version of the saved document schema.

/// The schema version this build writes. Raise it whenever saved data changes shape.
pub const SCHEMA_VERSION: u32 = 3;

/// How a saved schema version relates to this build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaCompatibility {
    Current,
    /// Saved by an older build, so it needs migrating before use.
    NeedsMigration,
    /// Saved by a newer build, so this build can't read it safely.
    TooNew,
    /// Version 0 has never existed, so the data is damaged.
    Invalid,
}

/// Compares a saved version with the version a build uses, normally `SCHEMA_VERSION`.
pub const fn schema_compatibility(version: u32, current: u32) -> SchemaCompatibility {
    if version == 0 {
        SchemaCompatibility::Invalid
    } else if version == current {
        SchemaCompatibility::Current
    } else if version < current {
        SchemaCompatibility::NeedsMigration
    } else {
        SchemaCompatibility::TooNew
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_versions() {
        assert_eq!(schema_compatibility(0, 2), SchemaCompatibility::Invalid);
        assert_eq!(
            schema_compatibility(1, 2),
            SchemaCompatibility::NeedsMigration
        );
        assert_eq!(schema_compatibility(2, 2), SchemaCompatibility::Current);
        assert_eq!(schema_compatibility(3, 2), SchemaCompatibility::TooNew);
    }
}
