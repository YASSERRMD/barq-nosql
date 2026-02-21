use crate::transaction::{PendingWrite, Transaction, TransactionId};
use crate::transaction_manager::TransactionManager;
use std::collections::HashSet;

#[derive(Debug)]
pub struct TxnConflict {
    pub txn_id: TransactionId,
    pub conflicting_keys: Vec<String>,
}

pub struct ConflictDetector {
    committed_writes: std::sync::RwLock<HashSet<String>>,
}

impl ConflictDetector {
    pub fn new() -> Self {
        Self {
            committed_writes: std::sync::RwLock::new(HashSet::new()),
        }
    }

    pub fn record_commit(&self, writes: &[PendingWrite]) {
        let mut committed = self.committed_writes.write().unwrap();
        for write in writes {
            let key = format!("{}:{}", write.collection, write.doc_id.0);
            committed.insert(key);
        }
    }

    pub fn detect_conflict(&self, txn: &Transaction, snapshot_version: u64) -> Option<TxnConflict> {
        let committed = self.committed_writes.read().unwrap();

        for write in &txn.writes {
            let key = format!("{}:{}", write.collection, write.doc_id.0);
            if committed.contains(&key) {
                return Some(TxnConflict {
                    txn_id: txn.id,
                    conflicting_keys: vec![key],
                });
            }
        }
        None
    }

    pub fn clear(&self) {
        let mut committed = self.committed_writes.write().unwrap();
        committed.clear();
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate_commit(
    txn: &Transaction,
    detector: &ConflictDetector,
    current_version: u64,
) -> Result<(), TxnConflict> {
    if let Some(conflict) = detector.detect_conflict(txn, current_version) {
        return Err(conflict);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_detection() {
        let detector = ConflictDetector::new();

        let mut txn = Transaction::new(TransactionId::new(), 1);
        txn.writes.push(PendingWrite {
            collection: "users".to_string(),
            doc_id: barq_core::DocumentId::new(),
            op: crate::transaction::TxnOp::Insert(barq_core::Document::new(
                barq_core::DocumentId::new(),
            )),
        });

        let result = validate_commit(&txn, &detector, 1);
        assert!(result.is_ok());
    }
}
