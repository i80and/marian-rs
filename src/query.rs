use std::collections::{HashMap, HashSet};
use regex::Regex;
use fts::DocID;

lazy_static! {
    static ref PAT_QUERY_PARTS: Regex = Regex::new(r#"((?:\s+|^)"[^"]+"(?:\s+|$))"#)
        .expect("Failed to compile query regex");
    static ref PAT_PHRASE: Regex = Regex::new(r#"\s*"([^"]*)"?\s*"#)
        .expect("Failed to compile phrase regex");
}

pub struct Query {
    pub terms: HashSet<String>,
    pub phrases: Vec<String>,
    pub stemmed_phrases: Vec<String>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            terms: HashSet::new(),
            phrases: vec![],
            stemmed_phrases: vec![],
        }
    }

    pub fn check_phrases(&self, _tokens: &HashMap<&String, &[u32]>) -> bool {
        true
    }

    pub fn filter(&self, doc_id: DocID) -> bool {
        true
    }
}
