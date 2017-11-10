use std::collections::{HashMap, HashSet};
use qp_trie;
use fts::DocID;

pub struct Trie {
    trie: qp_trie::Trie<qp_trie::wrapper::BString, HashSet<DocID>>,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            trie: qp_trie::Trie::new(),
        }
    }

    pub fn insert(&mut self, token: &str, id: DocID) {
        let key = qp_trie::wrapper::BString::from(token);
        self.trie.entry(key).or_insert_with(HashSet::new).insert(id);
    }

    pub fn search(&self, term: &str) -> HashMap<DocID, Vec<&str>> {
        let mut result = HashMap::new();

        for (k, doc_ids) in self.trie.iter_prefix_str(term) {
            for &doc_id in doc_ids {
                result
                    .entry(doc_id)
                    .or_insert_with(Vec::new)
                    .push(k.as_str());
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotency() {
        // Should be idempotent
        let mut trie = Trie::new();

        trie.insert("foobar", DocID(0));
        trie.insert("foobar", DocID(0));

        assert_eq!(trie.search("foobar"), hashmap![DocID(0) => vec!["foobar"]]);
    }

    #[test]
    fn test_additive() {
        let mut trie = Trie::new();
        trie.insert("foobar", DocID(0));
        trie.insert("foobar", DocID(1));

        assert_eq!(
            trie.search("foobar"),
            hashmap![
            DocID(0) => vec!["foobar"],
            DocID(1) => vec!["foobar"],
        ]
        );
    }

    #[test]
    fn test_prefix() {
        let mut trie = Trie::new();
        trie.insert("foobar", DocID(0));
        trie.insert("foobar", DocID(1));
        trie.insert("foobaz", DocID(0));

        assert_eq!(
            trie.search("foo"),
            hashmap![
                DocID(0) => vec!["foobar", "foobaz"],
                DocID(1) => vec!["foobar"]]
        );
    }
}
