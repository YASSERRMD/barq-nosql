use crate::document::{Document, Value};
use crate::error::BarqError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Int,
    Float,
    Bool,
    Vector(usize),
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub indexed: bool,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema {
    pub name: String,
    pub fields: Vec<FieldDef>,
}

impl CollectionSchema {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
        }
    }

    pub fn with_field(
        mut self,
        name: String,
        field_type: FieldType,
        indexed: bool,
        required: bool,
    ) -> Self {
        self.fields.push(FieldDef {
            name,
            field_type,
            indexed,
            required,
        });
        self
    }

    pub fn validate(&self, doc: &Document) -> Result<(), BarqError> {
        for field_def in &self.fields {
            if field_def.required && !doc.contains_key(&field_def.name) {
                return Err(BarqError::SchemaMismatch(format!(
                    "Required field '{}' is missing",
                    field_def.name
                )));
            }

            if let Some(value) = doc.get(&field_def.name) {
                if !Self::check_type(value, &field_def.field_type) {
                    return Err(BarqError::SchemaMismatch(format!(
                        "Field '{}' has incorrect type",
                        field_def.name
                    )));
                }
            }
        }
        Ok(())
    }

    fn check_type(value: &Value, field_type: &FieldType) -> bool {
        match (value, field_type) {
            (Value::String(_), FieldType::Text) => true,
            (Value::Int(_), FieldType::Int) => true,
            (Value::Float(_), FieldType::Float) => true,
            (Value::Bool(_), FieldType::Bool) => true,
            (Value::Vector(_), FieldType::Vector(_)) => true,
            (Value::Null, _) => true,
            (_, FieldType::Any) => true,
            _ => false,
        }
    }

    pub fn get_field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|f| f.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validation() {
        let schema = CollectionSchema::new("users".to_string())
            .with_field("name".to_string(), FieldType::Text, false, true)
            .with_field("age".to_string(), FieldType::Int, true, false);

        let mut doc = Document::new(crate::DocumentId::new());
        doc.insert("name".to_string(), Value::String("Alice".to_string()));

        assert!(schema.validate(&doc).is_ok());

        let doc_no_required = Document::new(crate::DocumentId::new());
        assert!(schema.validate(&doc_no_required).is_err());
    }
}
