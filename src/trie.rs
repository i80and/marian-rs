use fts::DocID;

pub struct Trie;

impl Trie {
    pub fn new() -> Self {
        Trie
    }

    pub fn insert(&mut self, token: &str, id: DocID) {}

    pub fn search(&self, term: &str, prefix: bool) -> Vec<(DocID, Vec<String>)> {
        vec![]
    }
}
