use crate::planner::QueryPlan;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedCost {
    pub row_count: usize,
    pub cpu_cost: f64,
    pub memory_cost: usize,
    pub is_full_scan: bool,
    pub recommendation: String,
}

pub struct QueryCostEstimator {
    stats_registry: HashMap<String, CollectionStats>,
}

#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub row_count: usize,
    pub avg_row_size: usize,
}

impl QueryCostEstimator {
    pub fn new() -> Self {
        Self {
            stats_registry: HashMap::new(),
        }
    }

    pub fn register_collection(&mut self, name: &str, row_count: usize, avg_row_size: usize) {
        self.stats_registry.insert(
            name.to_string(),
            CollectionStats {
                row_count,
                avg_row_size,
            },
        );
    }

    pub fn estimate(&self, plan: &QueryPlan) -> EstimatedCost {
        match plan {
            QueryPlan::FullScan { collection, .. } => {
                let stats = self.stats_registry.get(collection);
                let row_count = stats.map(|s| s.row_count).unwrap_or(0);

                if row_count > 10000 {
                    EstimatedCost {
                        row_count,
                        cpu_cost: 10.0,
                        memory_cost: row_count * stats.map(|s| s.avg_row_size).unwrap_or(512),
                        is_full_scan: true,
                        recommendation: "Add secondary index on filtered fields to reduce scan"
                            .to_string(),
                    }
                } else {
                    EstimatedCost {
                        row_count,
                        cpu_cost: 1.0,
                        memory_cost: row_count * stats.map(|s| s.avg_row_size).unwrap_or(512),
                        is_full_scan: true,
                        recommendation: "Small collection, scan acceptable".to_string(),
                    }
                }
            }
            QueryPlan::VectorSearch { k, .. } => EstimatedCost {
                row_count: *k,
                cpu_cost: 2.0,
                memory_cost: k * 1024,
                is_full_scan: false,
                recommendation: "Using HNSW index for vector search".to_string(),
            },
            QueryPlan::HybridSearch { .. } => EstimatedCost {
                row_count: 100,
                cpu_cost: 5.0,
                memory_cost: 50 * 1024,
                is_full_scan: false,
                recommendation: "Hybrid graph + vector search".to_string(),
            },
            _ => EstimatedCost {
                row_count: 10,
                cpu_cost: 0.5,
                memory_cost: 1024,
                is_full_scan: false,
                recommendation: "Optimized query plan".to_string(),
            },
        }
    }
}

impl Default for QueryCostEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimation() {
        let mut estimator = QueryCostEstimator::new();
        estimator.register_collection("users", 50000, 512);

        let plan = QueryPlan::FullScan {
            collection: "users".to_string(),
        };

        let cost = estimator.estimate(&plan);
        assert!(cost.is_full_scan);
        assert!(cost.cpu_cost > 1.0);
    }
}
