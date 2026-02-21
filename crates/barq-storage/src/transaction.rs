use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionId(u64);

impl TransactionId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for TransactionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxnState {
    Active,
    Committed,
    Aborted,
}

#[derive(Debug, Clone)]
pub enum TxnOp {
    Insert(barq_core::Document),
    Update(barq_core::Document),
    Delete,
}

#[derive(Debug, Clone)]
pub struct PendingWrite {
    pub collection: String,
    pub doc_id: barq_core::DocumentId,
    pub op: TxnOp,
}

#[derive(Debug)]
pub struct Transaction {
    pub id: TransactionId,
    pub snapshot_version: u64,
    pub writes: Vec<PendingWrite>,
    pub state: TxnState,
}

impl Transaction {
    pub fn new(id: TransactionId, snapshot_version: u64) -> Self {
        Self {
            id,
            snapshot_version,
            writes: Vec::new(),
            state: TxnState::Active,
        }
    }

    pub fn add_write(&mut self, write: PendingWrite) {
        self.writes.push(write);
    }

    pub fn is_active(&self) -> bool {
        self.state == TxnState::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_id() {
        let id1 = TransactionId::new();
        let id2 = TransactionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_transaction_state() {
        let mut txn = Transaction::new(TransactionId::new(), 1);
        assert!(txn.is_active());

        txn.state = TxnState::Committed;
        assert!(!txn.is_active());
    }
}
