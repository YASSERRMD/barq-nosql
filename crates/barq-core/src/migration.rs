use barq_core::FieldType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    pub fn new() -> Self {
        Self(1)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaChange {
    AddField {
        name: String,
        field_type: FieldType,
        default: Option<barq_core::Value>,
    },
    RemoveField {
        name: String,
    },
    RenameField {
        old_name: String,
        new_name: String,
    },
    ChangeFieldType {
        name: String,
        new_type: FieldType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMigration {
    pub from_version: SchemaVersion,
    pub to_version: SchemaVersion,
    pub changes: Vec<SchemaChange>,
}

impl SchemaMigration {
    pub fn new(from: SchemaVersion, to: SchemaVersion, changes: Vec<SchemaChange>) -> Self {
        Self {
            from_version: from,
            to_version: to,
            changes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        let v = SchemaVersion::new();
        assert_eq!(v.0, 1);

        let next = v.next();
        assert_eq!(next.0, 2);
    }
}
