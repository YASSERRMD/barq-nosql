use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryPlan {
    FullScan {
        collection: String,
    },
    VectorSearch {
        collection: String,
        field: String,
        query: Vec<f32>,
        k: usize,
    },
    HybridSearch {
        graph_expand: super::parser::GraphExpandSpec,
        vector_search: super::parser::VectorSearchSpec,
    },
    GraphTraversal {
        from: String,
        hops: usize,
        label: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexRange {
    Equal(serde_json::Value),
    GreaterThan(serde_json::Value),
    LessThan(serde_json::Value),
    Between(serde_json::Value, serde_json::Value),
}

pub struct QueryPlanner;

impl QueryPlanner {
    pub fn plan(query: &super::parser::BarqQuery) -> QueryPlan {
        if query.vector_search.is_some() && query.graph_expand.is_some() {
            return QueryPlan::HybridSearch {
                graph_expand: query.graph_expand.clone().unwrap(),
                vector_search: query.vector_search.clone().unwrap(),
            };
        }

        if let Some(ref vs) = query.vector_search {
            return QueryPlan::VectorSearch {
                collection: query.collection.clone(),
                field: vs.field.clone(),
                query: vs.query.clone(),
                k: vs.k,
            };
        }

        if let Some(ref ge) = query.graph_expand {
            return QueryPlan::GraphTraversal {
                from: ge.from.clone(),
                hops: ge.hops,
                label: ge.label.clone(),
            };
        }

        QueryPlan::FullScan {
            collection: query.collection.clone(),
        }
    }
}
