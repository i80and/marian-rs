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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_split_on_whitespace() {
        assert_eq!(
            tokenize("The qUick \tbrown\n\n\t fox.", false),
            vec!["the", "quick", "brown", "fox"]
        );
    }

    #[test]
    fn test_tokenize_code() {
        assert_eq!(
            tokenize(
                "db.scores.find(\n   { results: { $elemMatch: { $gte: 80, $lt: 85 } } }\n)",
                false
            ),
            vec![
                "db.scores.find",
                "results",
                "$elemmatch",
                "$gte",
                "80",
                "$lt",
                "85",
            ]
        );
    }

    #[test]
    fn test_atomic_phrases() {
        assert_eq!(
            tokenize("ops manager configuration", false),
            vec!["ops manager", "configuration"]
        );
        assert_eq!(stem("ops manager"), "ops manager");
    }

    #[test]
    fn test_porter2() {
        let f = File::open("test/stemmed-corpus.txt").expect("Failed to open porter2 test corpus");
        let buffered_reader = BufReader::new(&f);
        for raw_line in buffered_reader.lines() {
            let raw_line = raw_line.unwrap();
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<_> = trimmed.split_whitespace().take(2).collect();
            let word = &parts[0];
            let correct_stemmed = parts[1];
            let stemmed = stem(word);
            assert_eq!(stemmed, correct_stemmed);
        }
    }
}
