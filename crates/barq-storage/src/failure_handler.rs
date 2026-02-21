use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum WritePolicy {
    FailFast,
    RetryWithBackoff { max_retries: u32, base_ms: u64 },
    WriteAhead { deadline_ms: u64 },
}

#[derive(Debug)]
pub struct FailureRecord {
    pub node_id: String,
    pub failed_at: Instant,
    pub retry_count: u32,
}

#[derive(Debug)]
pub struct ReplayEntry {
    pub target_node: String,
    pub operation: Vec<u8>,
    pub queued_at: Instant,
    pub deadline: Instant,
}

pub struct FailureHandler {
    policy: WritePolicy,
    failed_nodes: Arc<Mutex<HashMap<String, FailureRecord>>>,
    pending_replays: Arc<Mutex<VecDeque<ReplayEntry>>>,
}

impl FailureHandler {
    pub fn new(policy: WritePolicy) -> Self {
        Self {
            policy,
            failed_nodes: Arc::new(Mutex::new(HashMap::new())),
            pending_replays: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn handle_write_failure(
        &self,
        node_id: String,
        operation: Vec<u8>,
    ) -> Result<(), String> {
        match self.policy {
            WritePolicy::FailFast => {
                Err(format!("Write failed for node {}", node_id))
            }
            WritePolicy::RetryWithBackoff { max_retries, base_ms } => {
                let mut failed = self.failed_nodes.lock().await;
                let record = failed.entry(node_id.clone()).or_insert(FailureRecord {
                    node_id: node_id.clone(),
                    failed_at: Instant::now(),
                    retry_count: 0,
                });
                
                if record.retry_count < max_retries {
                    record.retry_count += 1;
                    Ok(())
                } else {
                    failed.remove(&node_id);
                    Err(format!("Max retries exceeded for node {}", node_id))
                }
            }
            WritePolicy::WriteAhead { deadline_ms } => {
                let deadline = Instant::now() + Duration::from_millis(deadline_ms);
                let entry = ReplayEntry {
                    target_node: node_id,
                    operation,
                    queued_at: Instant::now(),
                    deadline,
                };
                let mut queue = self.pending_replays.lock().await;
                queue.push_back(entry);
                Ok(())
            }
        }
    }

    pub async fn background_replay_worker(&self) {
        let mut queue = self.pending_replays.lock().await;
        let now = Instant::now();
        
        queue.retain(|entry| entry.deadline > now);
    }

    pub async fn get_stats(&self) -> serde_json::Value {
        let queue = self.pending_replays.lock().await;
        let failed = self.failed_nodes.lock().await;
        
        serde_json::json!({
            "pending_replays": queue.len(),
            "failed_nodes": failed.len(),
        })
    }
}

impl Default for FailureHandler {
    fn default() -> Self {
        Self::new(WritePolicy::RetryWithBackoff { max_retries: 3, base_ms: 100 })
    }
}
