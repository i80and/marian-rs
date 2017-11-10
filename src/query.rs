use std::collections::{HashMap, HashSet};
use regex::Regex;
use stemmer::{is_stop_word, stem, tokenize};

lazy_static! {
    static ref PAT_QUERY_PARTS: Regex = Regex::new(r#""|[^"\s]+"#)
        .expect("Failed to compile query regex");
}

/// Return true if there is a configuration of numbers in the `tree` that
/// appear in sequential order.
fn have_contiguous_path(tree: &[&[u32]], last_candidate: Option<u32>) -> bool {
    if tree.is_empty() {
        return true;
    }

    for &element in tree[0] {
        if match last_candidate {
            None => true,
            Some(e) if element == e + 1 => true,
            _ => continue,
        } && have_contiguous_path(&tree[1..], Some(element))
        {
            return true;
        }
    }

    false
}

/// Check if the given `phrase_components` appear in contiguous positions
/// within the keywords map.
fn have_contiguous_keywords(
    phrase_components: &[String],
    keywords: &HashMap<&String, &[u32]>,
) -> bool {
    let mut path = vec![];

    for component in phrase_components {
        if let Some(&positions) = keywords.get(component) {
            path.push(positions);
        } else {
            return false;
        }
    }

    have_contiguous_path(&path, None)
}

pub struct Query<'a> {
    pub terms: HashSet<String>,
    pub phrases: Vec<String>,
    pub stemmed_phrases: Vec<Vec<String>>,
    pub search_properties: &'a [&'a str],
}

impl<'a> Query<'a> {
    pub fn new(query_string: &str, search_properties: &'a [&str]) -> Self {
        let mut query = Self {
            terms: HashSet::new(),
            phrases: vec![],
            stemmed_phrases: vec![],
            search_properties: search_properties,
        };

        let mut phrase: Option<String> = None;
        let mut end_phrase = false;
        for m in PAT_QUERY_PARTS.find_iter(query_string) {
            let match_str = m.as_str();

            match phrase {
                Some(ref mut s) => if match_str == "\"" {
                    end_phrase = true;
                } else {
                    query.add_term(match_str.to_owned());
                    s.push_str(match_str);
                    s.push(' ');
                },
                None => {
                    if match_str == "\"" {
                        phrase = Some(String::new());
                        continue;
                    }

                    query.add_term(match_str.to_owned());
                }
            }

            if end_phrase {
                if let Some(phrase) = phrase {
                    query.add_phrase(phrase);
                }

                phrase = None;
                end_phrase = false;
            }
        }

        if let Some(phrase) = phrase {
            query.add_phrase(phrase);
        }

        query
    }

    /// Return true if the exact phrases in the query appear in ANY of the fields
    /// appearing in the match.
    pub fn check_phrases(&self, tokens: &HashMap<&String, &[u32]>) -> bool {
        for phrase_tokens in &self.stemmed_phrases {
            if !have_contiguous_keywords(phrase_tokens.as_slice(), tokens) {
                return false;
            }
        }

        true
    }

    fn add_phrase(&mut self, mut phrase: String) {
        if phrase.as_bytes().ends_with(b" ") {
            phrase.pop();
        }

        let parts: Vec<_> = tokenize(&phrase, false)
            .iter()
            .filter(|term| !is_stop_word(term))
            .map(|term| stem(term).to_owned())
            .collect();
        self.stemmed_phrases.push(parts);
        self.phrases.push(phrase);
    }

    fn add_term(&mut self, term: String) {
        if is_stop_word(&term) {
            return;
        }

        self.terms.insert(term);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_term() {
        let query = Query::new("foo", &[]);
        assert_eq!(query.terms, hashset!["foo".to_owned()]);
        assert_eq!(query.search_properties, &[] as &[&str]);
        assert_eq!(query.phrases, Vec::<String>::new());
    }

    #[test]
    fn test_whitespace() {
        // it should delimit terms with any standard whitespace characters
        let query = Query::new("foo   \t  bar", &[]);
        assert_eq!(query.terms, hashset!["foo".to_owned(), "bar".to_owned()]);
        assert_eq!(query.phrases, Vec::<String>::new());
    }

    #[test]
    fn test_multi_word_phrases() {
        let query = Query::new("foo \"one phrase\" bar \"second phrase\"", &[]);
        assert_eq!(
            query.terms,
            hashset![
                "foo".to_owned(),
                "bar".to_owned(),
                "one".to_owned(),
                "phrase".to_owned(),
                "second".to_owned(),
            ]
        );
        assert_eq!(
            query.phrases,
            vec!["one phrase".to_owned(), "second phrase".to_owned()]
        );
    }

    #[test]
    fn test_adjacent_phrases() {
        let query = Query::new("\"introduce the\" \"officially supported\"", &[]);
        assert_eq!(
            query.terms,
            hashset![
                "introduce".to_owned(),
                "officially".to_owned(),
                "supported".to_owned(),
            ]
        );
        assert_eq!(
            query.phrases,
            vec![
                "introduce the".to_owned(),
                "officially supported".to_owned(),
            ]
        );
        assert_eq!(
            query.stemmed_phrases,
            vec![
                vec!["introduc".to_owned()],
                vec!["offici".to_owned(), "support".to_owned()],
            ]
        );
    }

    #[test]
    fn test_phrase_fragment() {
        // it should handle a phrase fragment as a single phrase
        let query = Query::new("\"officially supported", &[]);
        assert_eq!(
            query.terms,
            hashset!["officially".to_owned(), "supported".to_owned()]
        );
        assert_eq!(query.phrases, vec!["officially supported".to_owned()]);
    }

    #[test]
    fn test_check_phrases() {
        // it should match phrases with adjacent words
        let query = Query::new("\"Quoth the raven\"", &[]);
        let s1 = "quoth".to_owned();
        let s2 = "raven".to_owned();
        let v1 = vec![0, 5];
        let v2 = vec![8, 1];
        let token_positions = hashmap![&s1 => v1.as_slice(), &s2 => v2.as_slice()];
        assert_eq!(query.check_phrases(&token_positions), true);
    }

    #[test]
    fn test_check_phrases_negative() {
        // it should refuse phrases without adjacent words
        let query = Query::new("\"foo bar\" \"Quoth the raven\"", &[]);
        let s1 = "quoth".to_owned();
        let s2 = "raven".to_owned();
        let s3 = "foo".to_owned();
        let s4 = "bar".to_owned();
        let v1 = vec![0, 3];
        let v2 = vec![2, 5];
        let v3 = vec![6];
        let v4 = vec![7];

        let token_positions = hashmap![
            &s1 => v1.as_slice(),
            &s2 => v2.as_slice(),
            &s3 => v3.as_slice(),
            &s4 => v4.as_slice(),];
        assert_eq!(query.check_phrases(&token_positions), false);
    }
}
