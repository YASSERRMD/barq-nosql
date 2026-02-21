use barq_core::{Document, DocumentId};
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub const DEFAULT_FLUSH_THRESHOLD_DOCS: usize = 10_000;
pub const DEFAULT_FLUSH_THRESHOLD_BYTES: usize = 64 * 1024 * 1024;

pub struct MemTable {
    map: DashMap<DocumentId, Document>,
    doc_count: Arc<AtomicUsize>,
    byte_size: Arc<AtomicUsize>,
    pub flush_threshold_docs: usize,
    pub flush_threshold_bytes: usize,
}

impl MemTable {
    pub fn new() -> Self {
        Self::with_thresholds(DEFAULT_FLUSH_THRESHOLD_DOCS, DEFAULT_FLUSH_THRESHOLD_BYTES)
    }

    pub fn with_thresholds(docs: usize, bytes: usize) -> Self {
        Self {
            map: DashMap::new(),
            doc_count: Arc::new(AtomicUsize::new(0)),
            byte_size: Arc::new(AtomicUsize::new(0)),
            flush_threshold_docs: docs,
            flush_threshold_bytes: bytes,
        }
    }

    pub fn insert(&self, doc_id: DocumentId, doc: Document) -> Option<Document> {
        let new_size = self.estimate_doc_size(doc_id.clone(), Some(&doc));

        let result = self.map.insert(doc_id.clone(), doc);

        self.doc_count.fetch_add(1, Ordering::Relaxed);
        if let Some(ref old) = result {
            let old_size = self.estimate_doc_size(doc_id.clone(), Some(old));
            self.byte_size.fetch_sub(old_size, Ordering::Relaxed);
        }
        self.byte_size.fetch_add(new_size, Ordering::Relaxed);

        result
    }

    pub fn get(&self, doc_id: &DocumentId) -> Option<Document> {
        self.map.get(doc_id).map(|r| r.value().clone())
    }

    pub fn delete(&self, doc_id: &DocumentId) -> Option<Document> {
        if let Some((_key, result)) = self.map.remove(doc_id) {
            let size = self.estimate_doc_size(doc_id.clone(), Some(&result));
            self.doc_count.fetch_sub(1, Ordering::Relaxed);
            self.byte_size.fetch_sub(size, Ordering::Relaxed);
            Some(result)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.doc_count.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn current_size(&self) -> usize {
        self.byte_size.load(Ordering::Relaxed)
    }

    pub fn should_flush(&self) -> bool {
        self.doc_count.load(Ordering::Relaxed) >= self.flush_threshold_docs
            || self.byte_size.load(Ordering::Relaxed) >= self.flush_threshold_bytes
    }

    pub fn iter(&self) -> impl Iterator<Item = (DocumentId, Document)> + '_ {
        self.map
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
    }

    pub fn flush_batch(&self) -> MemTableFlushBatch {
        let items: Vec<(DocumentId, Document)> = self.iter().collect();
        self.doc_count.store(0, Ordering::Relaxed);
        self.byte_size.store(0, Ordering::Relaxed);

        MemTableFlushBatch { items }
    }

    fn estimate_doc_size(&self, _doc_id: DocumentId, doc: Option<&Document>) -> usize {
        if let Some(d) = doc {
            let json = serde_json::to_string(d).unwrap_or_default();
            json.len()
        } else {
            0
        }
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MemTableFlushBatch {
    pub items: Vec<(DocumentId, Document)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memtable_insert_get() {
        let memtable = MemTable::new();
        let doc = Document::new(DocumentId::new());
        let id = doc.id.clone();

        memtable.insert(id.clone(), doc);

        assert!(memtable.get(&id).is_some());
    }

    #[test]
    fn test_memtable_delete() {
        let memtable = MemTable::new();
        let doc = Document::new(DocumentId::new());
        let id = doc.id.clone();

        memtable.insert(id.clone(), doc);
        assert!(memtable.delete(&id).is_some());
        assert!(memtable.get(&id).is_none());
    }

    #[test]
    fn test_memtable_flush_threshold() {
        let memtable = MemTable::with_thresholds(10, 1024);

        for _i in 0..10 {
            let doc = Document::new(DocumentId::new());
            memtable.insert(doc.id.clone(), doc);
        }

        assert!(memtable.should_flush());
    }
}
