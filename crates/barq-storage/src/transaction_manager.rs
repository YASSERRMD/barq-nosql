use crate::transaction::{PendingWrite, Transaction, TransactionId, TxnOp, TxnState};
use barq_core::BarqError;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct TransactionManager {
    active_txns: Arc<DashMap<TransactionId, Transaction>>,
    version_counter: Arc<AtomicU64>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            active_txns: Arc::new(DashMap::new()),
            version_counter: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn begin(&self) -> TransactionId {
        let id = TransactionId::new();
        let snapshot = self.version_counter.load(Ordering::Relaxed);
        let txn = Transaction::new(id, snapshot);
        self.active_txns.insert(id, txn);
        id
    }

    pub fn write(
        &self,
        txn_id: TransactionId,
        collection: String,
        doc_id: barq_core::DocumentId,
        op: TxnOp,
    ) -> Result<(), BarqError> {
        let mut txn = self
            .active_txns
            .get_mut(&txn_id)
            .ok_or_else(|| BarqError::StorageError("Transaction not found".to_string()))?;

        if !txn.is_active() {
            return Err(BarqError::StorageError(
                "Transaction is not active".to_string(),
            ));
        }

        let write = PendingWrite {
            collection,
            doc_id,
            op,
        };
        txn.add_write(write);
        Ok(())
    }

    pub fn read(
        &self,
        _txn_id: TransactionId,
        _collection: &str,
        _doc_id: &barq_core::DocumentId,
    ) -> Result<Option<barq_core::Document>, BarqError> {
        Ok(None)
    }

    pub fn commit(&self, txn_id: TransactionId) -> Result<(), BarqError> {
        let mut txn = self
            .active_txns
            .get_mut(&txn_id)
            .ok_or_else(|| BarqError::StorageError("Transaction not found".to_string()))?;

        if !txn.is_active() {
            return Err(BarqError::StorageError(
                "Transaction is not active".to_string(),
            ));
        }

        txn.state = TxnState::Committed;
        self.version_counter.fetch_add(1, Ordering::Relaxed);
        self.active_txns.remove(&txn_id);
        Ok(())
    }

    pub fn abort(&self, txn_id: TransactionId) -> Result<(), BarqError> {
        let mut txn = self
            .active_txns
            .get_mut(&txn_id)
            .ok_or_else(|| BarqError::StorageError("Transaction not found".to_string()))?;

        if !txn.is_active() {
            return Err(BarqError::StorageError(
                "Transaction is not active".to_string(),
            ));
        }

        txn.state = TxnState::Aborted;
        txn.writes.clear();
        self.active_txns.remove(&txn_id);
        Ok(())
    }

    pub fn get_version(&self) -> u64 {
        self.version_counter.load(Ordering::Relaxed)
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_commit() {
        let mgr = TransactionManager::new();
        let txn_id = mgr.begin();
        assert!(txn_id.as_u64() >= 0);

        mgr.commit(txn_id).unwrap();
    }

    #[test]
    fn test_abort() {
        let mgr = TransactionManager::new();
        let txn_id = mgr.begin();
        mgr.abort(txn_id).unwrap();
    }
}
