use std::collections::{HashMap, HashSet};
use regex::Regex;
use porter2::StemmerContext;

/* Derived from the following: */
/* !
 * lunr.stopWordFilter
 * Copyright (C) 2017 Oliver Nightingale
 */

lazy_static! {
    static ref PAT_TOKEN_SEPARATOR: Regex = Regex::new(r#"[^\w$%.]+"#)
        .expect("Failed to compile token separator regex");
    static ref PAT_BAD_CHARS: Regex = Regex::new(r#"(?:^\.)|(?:\.$)"#)
        .expect("Failed to compile bad char regex");

    static ref STOP_WORDS: HashSet<&'static str> = vec!["a",
    "able",
    "about",
    "across",
    "after",
    "all",
    "almost",
    "also",
    "am",
    "among",
    "an",
    "and",
    "any",
    "are",
    "as",
    "at",
    "be",
    "because",
    "been",
    "but",
    "by",
    "can",
    "cannot",
    "could",
    "dear",
    "did",
    "do",
    "does",
    "either",
    "else",
    "ever",
    "every",
    "for",
    "from",
    "got",
    "had",
    "has",
    "have",
    "he",
    "her",
    "hers",
    "him",
    "his",
    "how",
    "however",
    "i",
    "i.e.",
    "if",
    "important",
    "in",
    "into",
    "is",
    "it",
    "its",
    "just",
    "may",
    "me",
    "might",
    "most",
    "must",
    "my",
    "neither",
    "no",
    "nor",
    "of",
    "off",
    "often",
    "on",
    "only",
    "or",
    "other",
    "our",
    "own",
    "rather",
    "said",
    "say",
    "says",
    "she",
    "should",
    "since",
    "so",
    "some",
    "than",
    "that",
    "the",
    "their",
    "them",
    "then",
    "there",
    "these",
    "they",
    "this",
    "tis",
    "to",
    "too",
    "twas",
    "us",
    "wants",
    "was",
    "we",
    "were",
    "what",
    "where",
    "which",
    "while",
    "who",
    "whom",
    "why",
    "will",
    "with",
    "would",
    "yet",
    "you",
    "your",
    "e.g."].into_iter().collect();

    static ref ATOMIC_PHRASE_MAP: HashMap<String, String> = vec![
        ("ops".to_owned(), "manager".to_owned()),
        ("cloud".to_owned(), "manager".to_owned()),
    ].into_iter().collect();

    static ref ATOMIC_PHRASES: HashSet<String> = ATOMIC_PHRASE_MAP
        .iter()
        .map(|(k, v)| format!("{} {}", k, v))
        .collect();
}

pub fn is_stop_word(word: &str) -> bool {
    STOP_WORDS.contains(word)
}

pub fn stem(word: &str) -> String {
    if ATOMIC_PHRASES.contains(word) {
        return word.to_owned();
    }

    let mut word = word.to_owned();
    StemmerContext::new(&mut word).get().to_owned()
}

pub fn tokenize(text: &str, fuzzy: bool) -> Vec<String> {
    let components: Vec<_> = PAT_TOKEN_SEPARATOR
        .split(text)
        .map(|token| PAT_BAD_CHARS.replace_all(token, "").to_lowercase())
        .collect();

    let mut skip = false;
    let mut tokens = Vec::with_capacity(components.len());
    for i in 0..components.len() {
        if skip {
            skip = false;
            continue;
        }

        let token = &components[i];
        if let Some(next_token) = components.get(i + 1) {
            let atomic_phrase_option = ATOMIC_PHRASE_MAP.get(token);
            if atomic_phrase_option == Some(next_token) {
                tokens.push(format!("{} {}", token, ATOMIC_PHRASE_MAP[token]));
                skip = true;
                continue;
            }
        }

        if token.len() > 1 {
            tokens.push(token.to_owned());
        }

        if fuzzy {
            for subtoken in token.split('.') {
                if subtoken.len() > 1 {
                    tokens.push(subtoken.to_owned());
                }
            }
        }
    }

    tokens
}
