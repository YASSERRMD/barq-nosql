use barq_core::DocumentId;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

pub struct BTreeIndex<K: Ord, V> {
    map: BTreeMap<K, V>,
}

#[derive(Clone)]
pub struct OrderedFloat(f64);

impl OrderedFloat {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for OrderedFloat {}

impl Hash for OrderedFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl<K: Ord + Clone, V: Clone> BTreeIndex<K, V> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn range_query(&self, lo: &K, hi: &K) -> Vec<(K, V)> {
        self.map
            .range(lo..hi)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn get_all(&self) -> Vec<(K, V)> {
        self.map
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }
}

impl<K: Ord + Clone, V: Clone> Default for BTreeIndex<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DocumentBTreeIndex {
    int_index: BTreeMap<i64, Vec<DocumentId>>,
    float_index: BTreeMap<OrderedFloat, Vec<DocumentId>>,
    string_index: BTreeMap<String, Vec<DocumentId>>,
}

impl DocumentBTreeIndex {
    pub fn new() -> Self {
        Self {
            int_index: BTreeMap::new(),
            float_index: BTreeMap::new(),
            string_index: BTreeMap::new(),
        }
    }

    pub fn insert_int(&mut self, key: i64, doc_id: DocumentId) {
        let entry = self.int_index.entry(key).or_insert_with(Vec::new);
        entry.push(doc_id);
    }

    pub fn insert_float(&mut self, key: f64, doc_id: DocumentId) {
        let entry = self
            .float_index
            .entry(OrderedFloat::new(key))
            .or_insert_with(Vec::new);
        entry.push(doc_id);
    }

    pub fn insert_string(&mut self, key: String, doc_id: DocumentId) {
        let entry = self.string_index.entry(key).or_insert_with(Vec::new);
        entry.push(doc_id);
    }

    pub fn range_int(&self, lo: i64, hi: i64) -> Vec<DocumentId> {
        self.int_index
            .range(lo..hi)
            .flat_map(|(_, ids)| ids.clone())
            .collect()
    }

    pub fn range_string(&self, lo: &str, hi: &str) -> Vec<DocumentId> {
        self.string_index
            .range(lo.to_string()..hi.to_string())
            .flat_map(|(_, ids)| ids.clone())
            .collect()
    }

    pub fn range_float(&self, lo: f64, hi: f64) -> Vec<DocumentId> {
        self.float_index
            .range(OrderedFloat::new(lo)..OrderedFloat::new(hi))
            .flat_map(|(_, ids)| ids.clone())
            .collect()
    }
}

impl Default for DocumentBTreeIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_insert_get() {
        let mut index: BTreeIndex<String, i64> = BTreeIndex::new();

        index.insert("key1".to_string(), 100);
        index.insert("key2".to_string(), 200);

        assert_eq!(index.get(&"key1".to_string()), Some(&100));
    }
}
