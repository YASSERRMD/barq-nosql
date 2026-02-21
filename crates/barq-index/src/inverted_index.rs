use barq_core::DocumentId;
use std::collections::{BTreeSet, HashMap};

pub struct InvertedIndex {
    index: HashMap<String, BTreeSet<DocumentId>>,
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn tokenize(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn insert_doc(&mut self, doc_id: DocumentId, text: &str) {
        let tokens = Self::tokenize(text);
        let doc_id = doc_id;
        for token in tokens {
            self.index
                .entry(token)
                .or_insert_with(BTreeSet::new)
                .insert(doc_id.clone());
        }
    }

    pub fn search(&self, query: &str) -> Vec<DocumentId> {
        let tokens = Self::tokenize(query);
        if tokens.is_empty() {
            return Vec::new();
        }

        if let Some(first_set) = tokens.iter().filter_map(|t| self.index.get(t)).next() {
            let mut result: BTreeSet<DocumentId> = first_set.clone();

            for token in &tokens {
                if let Some(set) = self.index.get(token) {
                    result = result.intersection(set).cloned().collect();
                }
            }

            result.into_iter().collect()
        } else {
            Vec::new()
        }
    }

    pub fn search_or(&self, query: &str) -> Vec<DocumentId> {
        let tokens = Self::tokenize(query);
        let mut result = BTreeSet::new();

        for token in tokens {
            if let Some(docs) = self.index.get(&token) {
                for doc in docs {
                    result.insert(doc.clone());
                }
            }
        }

        result.into_iter().collect()
    }

    pub fn remove_doc(&mut self, doc_id: &DocumentId, text: &str) {
        let tokens = Self::tokenize(text);
        for token in tokens {
            if let Some(docs) = self.index.get_mut(&token) {
                docs.remove(doc_id);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn get_postings(&self, term: &str) -> Option<&BTreeSet<DocumentId>> {
        self.index.get(term)
    }
}

impl Default for InvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = InvertedIndex::tokenize("Hello World Hello");
        assert_eq!(tokens, vec!["hello", "world", "hello"]);
    }

    #[test]
    fn test_insert_and_search() {
        let mut index = InvertedIndex::new();

        let doc1 = DocumentId::new();
        let doc2 = DocumentId::new();

        index.insert_doc(doc1.clone(), "hello world");
        index.insert_doc(doc2.clone(), "hello foo");

        let results = index.search("hello");
        assert_eq!(results.len(), 2);

        let results = index.search("world");
        assert_eq!(results.len(), 1);
    }
}
