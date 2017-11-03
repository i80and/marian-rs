use std::collections::HashMap;
use qp_trie;
use fts::DocID;

pub struct Trie {
    trie: qp_trie::Trie<qp_trie::wrapper::BString, DocID>,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            trie: qp_trie::Trie::new(),
        }
    }

    pub fn insert(&mut self, token: &str, id: DocID) {
        self.trie.insert_str(token, id);
    }

    pub fn search(&self, term: &str) -> HashMap<DocID, Vec<&str>> {
        let mut result = HashMap::new();

        for (k, &doc_id) in self.trie.iter_prefix_str(term) {
            result
                .entry(doc_id)
                .or_insert_with(Vec::new)
                .push(k.as_str());
        }

        result
    }
}
