use std::collections::{HashMap, HashSet};
use std::{cmp, iter};
use query::Query;
use trie::Trie;

const MAX_MATCHES: usize = 150;

pub type DocID = u32;
type TokenID = u32;

fn is_stop_word(word: &str) -> bool {
    false
}

fn stem(word: &str) -> String {
    word.to_owned()
}

fn tokenize(text: &str, prefix: bool) -> Vec<String> {
    vec![]
}

/// Normalize URLs by chopping off trailing index.html components.
/// standard deviation of relevancy. Return that minimum relevancy score.
fn normalize_url(url: &str) -> &str {
    url.trim_right_matches("index.html")
}

/// We want to penalize the final score of any matches that are in the bottom
/// standard deviation of relevancy. Return that minimum relevancy score.
fn compute_relevancy_threshold(matches: &[SearchMatch]) -> f32 {
    let mut mean_score: f32 = 0.0;
    for m in matches {
        mean_score += m.relevancy_score;
    }
    mean_score /= matches.len() as f32;

    let mut sum: f32 = 0.0;
    for m in matches {
        sum += (m.relevancy_score - mean_score).powf(2.0);
    }

    (1.0 / (matches.len() as f32 - 1.0) * sum).log2()
}

fn hits(
    mut matches: Vec<SearchMatch>,
    convergance_threshold: f32,
    max_iterations: u32,
) -> Vec<SearchMatch> {
    let mut last_authority_norm = 0.0;
    let mut last_hub_norm = 0.0;
    for _ in 0..max_iterations {
        let mut authority_norm: f32 = 0.0;
        // Update all authority scores
        for mut m in &mut matches {
            m.authority_score = 0.0;
            for incoming_match in &m.incoming_neighbors {
                // m.authority_score += incoming_match.hub_score;
            }
            authority_norm += m.authority_score.powf(2.0);
        }

        // Normalise the authority scores
        let authority_norm = authority_norm.sqrt();
        for mut m in &mut matches {
            m.authority_score /= authority_norm;
        }

        // Update all hub scores
        let mut hub_norm: f32 = 0.0;
        for mut m in &mut matches {
            m.hub_score = 0.0;
            for outgoing_match in &m.outgoing_neighbors {
                // m.hub_score += outgoing_match.authority_score;
            }
            hub_norm += m.hub_score.powf(2.0);
        }

        // Normalise the hub scores
        let hub_norm = hub_norm.sqrt();
        for mut m in &mut matches {
            m.hub_score /= hub_norm
        }

        if (authority_norm - last_authority_norm).abs() < convergance_threshold
            && (hub_norm - last_hub_norm).abs() < convergance_threshold
        {
            break;
        }

        last_authority_norm = authority_norm;
        last_hub_norm = hub_norm;
    }

    // Cut anything with zero relevancy
    let mut matches: Vec<SearchMatch> = matches
        .drain(..)
        .filter(|m| m.relevancy_score > 0.0)
        .collect();

    // Compute statistics for score normalization
    let mut max_relevancy_score: f32 = 0.0;
    let mut max_authority_score: f32 = 0.0;
    let relevancy_score_threshold = compute_relevancy_threshold(&matches);
    for mut m in &mut matches {
        if m.authority_score.is_nan() {
            m.authority_score = 1e-10;
        }

        // Ignore anything with bad relevancy for the purposes of score normalization
        if m.relevancy_score < relevancy_score_threshold {
            continue;
        }

        if m.relevancy_score > max_relevancy_score {
            max_relevancy_score = m.relevancy_score;
        }
        if m.authority_score > max_authority_score {
            max_authority_score = m.authority_score;
        }
    }

    // Compute the final ranking score
    for mut m in &mut matches {
        m.compute_score(max_relevancy_score, max_authority_score);

        // Penalize anything with especially poor relevancy
        if m.relevancy_score < relevancy_score_threshold * 2.5 {
            m.score -= relevancy_score_threshold / m.relevancy_score;
        }
    }

    matches.sort_unstable_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
    matches.truncate(MAX_MATCHES);
    matches
}

/// Yuanhua Lv and ChengXiang Zhai. 2011. Lower-bounding term frequency
/// normalization. In Proceedings of the 20th ACM international
/// conference on Information and knowledge management (CIKM '11), Bettina
/// Berendt, Arjen de Vries, Wenfei Fan, Craig Macdonald, Iadh Ounis, and
/// Ian Ruthven (Eds.). ACM, New York, NY, USA, 7-16. DOI: <https://doi.org/10.1145/2063576.2063584>
fn dirichlet_plus(
    term_frequency_in_query: f32,
    term_frequency_in_doc: u32,
    term_probability_in_language: f32,
    doc_length: u32,
    query_length: u32,
) -> f32 {
    let delta = 0.05;

    // In the range suggested by A Study of Smoothing Methods for Language Models
    // Applied to Ad Hoc Information Retrieval [Zhai, Lafferty]
    let mu = 2000.0;

    // In some fields, the query may never exist, making its probability 0.
    // This is... weird. Return 0 to avoid NaN since while dirichlet+
    // prefers rare words, a nonexistent word should probably be ignored.
    if term_probability_in_language == 0.0 {
        return 0.0;
    }

    let term2 = (1.0 + (term_frequency_in_doc as f32 / (mu * term_probability_in_language))).log2();
    let term2 = term2 + (1.0 + (delta / (mu * term_probability_in_language))).log2();

    let term3 = query_length as f32 * (mu / (doc_length as f32 + mu)).log2();

    (term_frequency_in_query * term2) + term3
}

struct TermEntry {
    docs: Vec<DocID>,
    positions: HashMap<DocID, Vec<TokenID>>,
    times_appeared: HashMap<String, u32>,
}

impl TermEntry {
    fn new() -> Self {
        Self {
            docs: vec![],
            positions: HashMap::new(),
            times_appeared: HashMap::new(),
        }
    }

    fn register(&mut self, field_name: String, docid: DocID) {
        self.docs.push(docid);
        *self.times_appeared.entry(field_name).or_insert(0) += 1;
    }

    fn add_token_position(&mut self, docid: DocID, token_id: TokenID) {
        self.positions
            .entry(docid)
            .or_insert_with(Vec::new)
            .push(token_id);
    }
}

struct DocumentEntry {
    len: u32,
    term_frequencies: HashMap<String, u32>,
    weight: f32,
}

impl DocumentEntry {
    fn new(number_of_tokens: u32, term_frequencies: HashMap<String, u32>, weight: f32) -> Self {
        DocumentEntry {
            len: number_of_tokens,
            term_frequencies,
            weight,
        }
    }
}

pub struct SearchMatch {
    _id: DocID,
    relevancy_score: f32,
    terms: HashSet<String>,

    score: f32,
    authority_score: f32,
    hub_score: f32,
    incoming_neighbors: Vec<DocID>,
    outgoing_neighbors: Vec<DocID>,
}

impl SearchMatch {
    fn new(docid: DocID) -> Self {
        Self {
            _id: docid,
            relevancy_score: 0.0,
            terms: HashSet::new(),

            score: 0.0,
            authority_score: 1.0,
            hub_score: 1.0,
            incoming_neighbors: vec![],
            outgoing_neighbors: vec![],
        }
    }

    fn compute_score(&mut self, max_relevancy_score: f32, max_authority_score: f32) {
        let normalized_relevancy_score = self.relevancy_score / max_relevancy_score + 1.0;
        let normalized_authority_score = self.authority_score / max_authority_score + 1.0;
        self.score = normalized_relevancy_score.log2() + normalized_authority_score.log2();
    }
}

pub struct Field {
    name: String,
    documents: HashMap<DocID, DocumentEntry>,
    weight: f32,
    total_tokens: u32,

    length_weight: f32,
}

impl Field {
    pub fn new(name: &str, weight: f32) -> Self {
        Self {
            name: name.to_owned(),
            documents: HashMap::new(),
            weight,
            total_tokens: 0,
            length_weight: 0.0,
        }
    }

    /// Return the inverse average number of unique terms per document.
    /// This makes no fscking sense, but is useful as a weighting factor
    /// in my testing.
    fn compute_length_weight(&mut self) {
        let mut n_terms: usize = 0;
        for doc in self.documents.values() {
            n_terms += doc.term_frequencies.len()
        }

        self.length_weight = self.documents.len() as f32 / n_terms as f32;
    }
}

pub struct Document {
    pub _id: DocID,
    pub url: String,
    pub links: Vec<String>,
    pub weight: f32,
    pub data: HashMap<String, String>,
}

pub struct FTSIndex {
    fields: Vec<Field>,
    trie: Trie,
    terms: HashMap<String, TermEntry>,
    doc_id: DocID,
    term_id: u32,
    document_weights: HashMap<DocID, f32>,

    link_graph: HashMap<String, Vec<String>>,
    inverse_link_graph: HashMap<String, Vec<String>>,
    url_to_id: HashMap<String, DocID>,
    id_to_url: HashMap<DocID, String>,

    incoming_neighbors: Vec<Vec<DocID>>,
    outgoing_neighbors: Vec<Vec<DocID>>,

    word_correlations: HashMap<String, Vec<(String, f32)>>,

    dirty: bool,
}

impl FTSIndex {
    pub fn new(fields: Vec<Field>) -> Self {
        Self {
            fields,
            trie: Trie::new(),
            terms: HashMap::new(),
            doc_id: 0,
            term_id: 0,
            document_weights: HashMap::new(),

            link_graph: HashMap::new(),
            inverse_link_graph: HashMap::new(),
            url_to_id: HashMap::new(),
            id_to_url: HashMap::new(),

            incoming_neighbors: vec![],
            outgoing_neighbors: vec![],

            word_correlations: HashMap::new(),

            dirty: true,
        }
    }

    // word can be multiple tokens. synonym must be a single token.
    fn correlate_word(&mut self, word: &str, synonym: &str, closeness: f32) {
        let parts = tokenize(word, false);
        let word = parts.iter().map(|w| stem(w)).collect::<Vec<_>>().join(" ");
        let synonym = stem(synonym);

        let correlation_entry = self.word_correlations.entry(word).or_insert_with(|| vec![]);
        correlation_entry.push((synonym, closeness));
    }

    fn collect_correlations(&self, terms: &[&String]) -> HashMap<String, f32> {
        let mut stemmed_terms: HashMap<String, f32> = HashMap::new();
        for term in terms {
            stemmed_terms.insert(stem(term), 1.0);
        }

        for i in 0..terms.len() {
            let mut pair = vec![stem(terms[i])];

            if i < terms.len() - 1 {
                let new_value = format!("{} {}", pair[0], stem(terms[i + 1]));
                pair.push(new_value);
            }

            for term in pair {
                let correlations = match self.word_correlations.get(&term) {
                    Some(c) => c,
                    None => continue,
                };

                for &(ref correlation, weight) in correlations {
                    let new_weight = stemmed_terms.get(correlation).unwrap_or(&0.0).max(weight);
                    stemmed_terms.insert(correlation.to_owned(), new_weight);
                }
            }
        }

        stemmed_terms
    }

    pub fn add<F>(&mut self, mut document: Document, on_token: F) -> DocID
    where
        F: Fn(&str),
    {
        self.dirty = true;
        document._id = self.doc_id;
        document.url = normalize_url(&document.url).to_owned();

        for href in &document.links {
            let normalized_href = normalize_url(href).to_owned();
            let mut incoming_links = self.inverse_link_graph
                .entry(normalized_href)
                .or_insert_with(Vec::new);
            incoming_links.push(document.url.to_owned());
        }

        self.link_graph
            .insert(document.url.to_owned(), document.links);
        self.url_to_id.insert(document.url.to_owned(), document._id);
        self.id_to_url.insert(document._id, document.url);

        let mut correlations: Vec<(String, u8, f32)> = vec![];

        for field in &mut self.fields {
            let mut term_frequencies = HashMap::new();

            let text = match document.data.get(&field.name) {
                Some(t) => t,
                None => continue,
            };

            let tokens = tokenize(text, true);
            let mut number_of_tokens = 0;

            for token in &tokens {
                on_token(token);

                if is_stop_word(token) {
                    continue;
                }

                let mut token = token.to_owned();
                if token.starts_with("%%") {
                    correlations.push((token.to_owned(), 2, 0.9));
                } else if token.starts_with('$') || token.starts_with('%') {
                    correlations.push((token.to_owned(), 1, 0.9));
                } else {
                    token = stem(&token);
                }

                number_of_tokens += 1;
                self.term_id += 1;

                let mut index_entry = self.terms
                    .entry(token.to_owned())
                    .or_insert_with(TermEntry::new);
                let count = *term_frequencies.get(&token).unwrap_or(&0);
                term_frequencies.insert(token.to_owned(), count + 1);

                if count == 0 {
                    self.trie.insert(&token, document._id);
                    index_entry.register(field.name.to_owned(), document._id);
                }

                index_entry.add_token_position(document._id, self.term_id);
            }

            // After each field, bump by one to prevent accidental adjacency.
            self.term_id += 1;

            field.total_tokens += number_of_tokens;
            field.documents.insert(
                document._id,
                DocumentEntry::new(number_of_tokens, term_frequencies, document.weight),
            );
        }

        for (token, prefix_size, closeness) in correlations {
            self.correlate_word(&token, &token[prefix_size as usize..], closeness);
        }

        self.document_weights.insert(document._id, document.weight);
        self.doc_id += 1;

        document._id
    }

    pub fn finish(&mut self) {
        for field in &mut self.fields {
            field.compute_length_weight();
        }

        self.dirty = false;
    }

    fn get_neighbors(&self, base_set: &mut HashMap<DocID, SearchMatch>, doc_id: DocID) {
        let url = match self.id_to_url.get(&doc_id) {
            Some(url) => url,
            None => return,
        };

        let outgoing_neighbors = match self.outgoing_neighbors.get(doc_id as usize) {
            Some(v) => v.to_owned(),
            None => {
                let mut outgoing_neighbors_set = HashSet::new();
                if let Some(links) = self.link_graph.get(url) {
                    for link in links {
                        if let Some(descendent_id) = self.url_to_id.get(link) {
                            outgoing_neighbors_set.insert(descendent_id);
                        }
                    }
                }

                let mut outgoing_neighbors: Vec<_> =
                    outgoing_neighbors_set.drain().cloned().collect();
                // self.outgoing_neighbors[doc_id as usize] = outgoing_neighbors.to_owned();
                outgoing_neighbors
            }
        };

        let incoming_neighbors = match self.incoming_neighbors.get(doc_id as usize) {
            Some(v) => v.to_owned(),
            None => {
                let mut incoming_neighbors_set = HashSet::new();
                if let Some(urls) = self.inverse_link_graph.get(url) {
                    for ancestor_url in urls {
                        if let Some(ancestor_id) = self.url_to_id.get(ancestor_url) {
                            incoming_neighbors_set.insert(ancestor_id);
                        }
                    }
                }

                let incoming_neighbors: Vec<_> = incoming_neighbors_set.drain().cloned().collect();
                // self.incoming_neighbors[doc_id as usize] = incoming_neighbors.to_owned();
                incoming_neighbors
            }
        };

        for neighbor_id in &incoming_neighbors {
            let new_match = base_set
                .entry(*neighbor_id)
                .or_insert_with(|| SearchMatch::new(*neighbor_id));
            // search_match.incoming_neighbors.push(new_match);
        }

        for neighbor_id in &outgoing_neighbors {
            let new_match = base_set
                .entry(*neighbor_id)
                .or_insert_with(|| SearchMatch::new(*neighbor_id));
            // search_match.outgoing_neighbors.push(new_match);
        }
    }

    fn collect_matches_from_trie<'a, I>(&self, terms: I) -> Vec<(DocID, Vec<String>)>
    where
        I: iter::Iterator<Item = &'a String>,
    {
        let mut result_set = vec![];
        for term in terms {
            for (doc_id, terms) in self.trie.search(term, true) {
                result_set.push((doc_id, terms.to_owned()));
            }
        }

        result_set
    }

    pub fn search(&self, query: Query, use_hits: bool) -> Vec<SearchMatch> {
        let mut match_set: HashMap<DocID, SearchMatch> = HashMap::new();
        let original_terms: HashSet<_> = query.terms.iter().collect();
        let original_terms: Vec<_> = original_terms.into_iter().collect();
        let mut stemmed_terms = self.collect_correlations(&original_terms);

        let mut added_terms = vec![];
        for term in stemmed_terms.keys() {
            let correlations = match self.word_correlations.get(term) {
                Some(c) => c,
                None => continue,
            };

            for &(ref correlation, weight) in correlations {
                let new_weight = stemmed_terms.get(correlation).unwrap_or(&0.0).max(weight);
                added_terms.push((correlation.to_owned(), new_weight));
            }
        }

        for (correlation, weight) in added_terms.drain(..) {
            stemmed_terms.insert(correlation, weight);
        }

        let mut keys = stemmed_terms.keys();
        for (doc_id, ref terms) in self.collect_matches_from_trie(&mut keys) {
            if !query.filter(doc_id) {
                continue;
            }

            for term in terms {
                let term_entry = &self.terms[term];

                let mut term_relevancy_score: f32 = 0.0;
                for field in &self.fields {
                    let doc_entry = match field.documents.get(&doc_id) {
                        Some(e) => e,
                        None => continue,
                    };

                    let term_weight = *(stemmed_terms.get(term).unwrap_or(&0.1));
                    let term_frequency_in_doc =
                        *(doc_entry.term_frequencies.get(term).unwrap_or(&0));
                    let term_probability =
                        *(term_entry.times_appeared.get(&field.name).unwrap_or(&0)) as f32
                            / cmp::max(field.total_tokens, 500) as f32;

                    // Larger fields yield larger scores, but we want fields to have roughly
                    // equal weight. field.lengthWeight is stupid, but yields good results.
                    term_relevancy_score += dirichlet_plus(
                        term_weight,
                        term_frequency_in_doc,
                        term_probability,
                        doc_entry.len,
                        original_terms.len() as u32,
                    ) * field.weight
                        * field.length_weight
                        * self.document_weights[&doc_id];
                }

                let search_match = match_set
                    .entry(doc_id)
                    .or_insert_with(|| SearchMatch::new(doc_id));
                search_match.relevancy_score += term_relevancy_score;
                search_match.terms.insert(term.to_owned());
            }
        }

        // Create a root set of the core relevant results

        let root_set = match_set.drain().map(|(_, v)| v);
        let mut root_set: Vec<_> = if query.phrases.is_empty() {
            root_set.collect()
        } else {
            root_set
                .filter(|search_match| {
                    let mut tokens = HashMap::new();
                    for term in &search_match.terms {
                        let term_entry = match self.terms.get(term) {
                            Some(v) => v,
                            None => return false,
                        };

                        let positions = match term_entry.positions.get(&search_match._id) {
                            Some(v) => v,
                            None => return false,
                        };

                        tokens.insert(term, positions.as_slice());
                    }
                    query.check_phrases(&tokens)
                })
                .collect()
        };

        if !use_hits {
            root_set.sort_unstable_by(|a, b| {
                a.relevancy_score.partial_cmp(&b.relevancy_score).unwrap()
            });
            root_set.truncate(MAX_MATCHES);
            return root_set;
        }

        // Expand our root set's neighbors to create a base set: the set of all
        // relevant pages, as well as pages that link TO or are linked FROM those pages.
        let root_ids: Vec<DocID> = root_set.iter().map(|m| m._id).collect();
        let mut base_set: HashMap<DocID, SearchMatch> = HashMap::new();
        for search_match in root_set.drain(..) {
            base_set.insert(search_match._id, search_match);
        }

        for mut doc_id in &root_ids {
            self.get_neighbors(&mut base_set, *doc_id);
        }

        let base_set: Vec<_> = base_set.drain().map(|(k, v)| v).collect();

        // Run HITS to re-sort our results based on authority
        hits(base_set, 0.00001, 200)
    }
}
