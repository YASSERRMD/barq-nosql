use barq_core::{BarqError, Document, DocumentId};
use std::collections::HashMap;

pub struct QueryExecutor;

impl QueryExecutor {
    pub fn execute(
        _plan: super::planner::QueryPlan,
        _documents: &HashMap<DocumentId, Document>,
    ) -> Result<Vec<Document>, BarqError> {
        Ok(Vec::new())
    }

    pub fn apply_sort(
        documents: Vec<Document>,
        _sort_field: &str,
        _order: super::parser::SortOrder,
    ) -> Vec<Document> {
        documents
    }

    pub fn paginate(documents: Vec<Document>, limit: usize, offset: usize) -> Vec<Document> {
        documents.into_iter().skip(offset).take(limit).collect()
    }
}
