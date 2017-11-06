use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use std::{cmp, iter};
use query::Query;
use trie::Trie;
use stemmer::{is_stop_word, stem, tokenize};

const MAX_MATCHES: usize = 150;

pub type DocID = u32;
type TokenID = u32;

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
}

impl DocumentEntry {
    fn new(number_of_tokens: u32, term_frequencies: HashMap<String, u32>) -> Self {
        DocumentEntry {
            len: number_of_tokens,
            term_frequencies,
        }
    }
}

struct SearchMatch {
    _id: DocID,
    relevancy_score: f32,
    terms: HashSet<String>,

    score: f32,
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
            incoming_neighbors: vec![],
            outgoing_neighbors: vec![],
        }
    }

    fn compute_score(
        &mut self,
        max_relevancy_score: f32,
        authority_score: f32,
        max_authority_score: f32,
    ) {
        let normalized_relevancy_score = self.relevancy_score / max_relevancy_score + 1.0;
        let normalized_authority_score = authority_score / max_authority_score + 1.0;
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

pub struct DocumentBuilder {
    pub url: String,
    pub links: Vec<String>,
    pub weight: f32,
    pub data: HashMap<String, String>,
}

impl DocumentBuilder {
    pub fn new(url: String) -> Self {
        Self {
            url,
            links: vec![],
            weight: 1.0,
            data: hashmap![],
        }
    }
}

pub struct Document {
    pub _id: DocID,
    pub url: String,
    pub weight: f32,
    pub data: HashMap<String, String>,
}

impl Document {
    pub fn title(&self) -> &str {
        &self.data["title"]
    }

    pub fn preview(&self) -> &str {
        &self.data["preview"]
    }
}

struct MatchSet {
    matches: Vec<SearchMatch>,
    doc_id_to_match_id: HashMap<DocID, u32>,
}

impl MatchSet {
    fn new() -> Self {
        Self {
            matches: vec![],
            doc_id_to_match_id: HashMap::new(),
        }
    }

    fn insert(&mut self, mut search_match: SearchMatch, fts: &FTSIndex) {
        let match_id = self.matches.len() as u32;
        let doc_id = search_match._id;

        if let Some(v) = fts.incoming_neighbors.get(&doc_id) {
            search_match.incoming_neighbors.extend(v.iter().cloned());
        }

        if let Some(v) = fts.outgoing_neighbors.get(&doc_id) {
            search_match.outgoing_neighbors.extend(v.iter().cloned());
        }

        self.matches.push(search_match);
        self.doc_id_to_match_id.insert(doc_id, match_id);
    }

    fn get_neighbors(&mut self, doc_id: DocID) {
        let match_id = self.doc_id_to_match_id[&doc_id];
        let search_match = &mut self.matches[match_id as usize];

        let mut incoming_neighbors = vec![];
        let mut outgoing_neighbors = vec![];

        for &neighbor_id in &search_match.incoming_neighbors {
            // base_set
            //     .entry(neighbor_id)
            //     .or_insert_with(|| SearchMatch::new(neighbor_id));
            incoming_neighbors.push(neighbor_id);
        }

        for &neighbor_id in &search_match.outgoing_neighbors {
            // base_set
            //     .entry(neighbor_id)
            //     .or_insert_with(|| SearchMatch::new(neighbor_id));
            outgoing_neighbors.push(neighbor_id);
        }

        search_match
            .incoming_neighbors
            .extend(incoming_neighbors.iter());
        search_match
            .outgoing_neighbors
            .extend(outgoing_neighbors.iter());
    }

    fn finish(&mut self, root_set: &[DocID]) {
        for &doc_id in root_set {
            self.get_neighbors(doc_id);
        }
    }

    fn hits(&mut self, convergance_threshold: f32, max_iterations: u32) -> Vec<SearchMatch> {
        let mut last_authority_norm = 0.0;
        let mut last_hub_norm = 0.0;

        let mut authority_scores: Vec<f32> = vec![1.0; self.matches.len()];
        let mut hub_scores: Vec<f32> = vec![1.0; self.matches.len()];

        for _ in 0..max_iterations {
            let mut authority_norm: f32 = 0.0;
            // Update all authority scores
            for (match_id, search_match) in self.matches.iter().enumerate() {
                authority_scores[match_id] = 0.0;
                for &incoming_match_id in &search_match.incoming_neighbors {
                    authority_scores[match_id] += hub_scores[incoming_match_id as usize];
                }
                authority_norm += authority_scores[match_id].powf(2.0);
            }

            // Normalise the authority scores
            let authority_norm = authority_norm.sqrt();
            for authority_score in &mut authority_scores {
                *authority_score /= authority_norm;
            }

            // Update all hub scores
            let mut hub_norm: f32 = 0.0;
            for (match_id, mut search_match) in self.matches.iter().enumerate() {
                hub_scores[match_id] = 0.0;
                for &outgoing_match_id in &search_match.outgoing_neighbors {
                    hub_scores[match_id] += authority_scores[outgoing_match_id as usize];
                }
                hub_norm += hub_scores[match_id].powf(2.0);
            }

            // Normalise the hub scores
            let hub_norm = hub_norm.sqrt();
            for hub_score in &mut hub_scores {
                *hub_score /= hub_norm;
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
        let mut matches: Vec<SearchMatch> = self.matches
            .drain(..)
            .filter(|m| m.relevancy_score > 0.0)
            .collect();

        // Compute statistics for score normalization
        let mut max_relevancy_score: f32 = 0.0;
        let mut max_authority_score: f32 = 0.0;
        let relevancy_score_threshold = compute_relevancy_threshold(&matches);
        for (match_id, search_match) in matches.iter().enumerate() {
            if authority_scores[match_id].is_nan() {
                authority_scores[match_id] = 1e-10;
            }

            // Ignore anything with bad relevancy for the purposes of score normalization
            if search_match.relevancy_score < relevancy_score_threshold {
                continue;
            }

            if search_match.relevancy_score > max_relevancy_score {
                max_relevancy_score = search_match.relevancy_score;
            }
            if authority_scores[match_id] > max_authority_score {
                max_authority_score = authority_scores[match_id];
            }
        }

        // Compute the final ranking score
        for (match_id, mut search_match) in matches.iter_mut().enumerate() {
            let authority_score = authority_scores[match_id];
            search_match.compute_score(max_relevancy_score, authority_score, max_authority_score);

            // Penalize anything with especially poor relevancy
            if search_match.relevancy_score < relevancy_score_threshold * 2.5 {
                search_match.score -= relevancy_score_threshold / search_match.relevancy_score;
            }
        }

        matches.sort_unstable_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
        matches.truncate(MAX_MATCHES);
        matches
    }
}

pub struct FTSIndex {
    fields: Vec<Field>,
    trie: Trie,
    terms: HashMap<String, TermEntry>,
    doc_id: DocID,
    term_id: u32,

    documents: Vec<Document>,
    link_graph: HashMap<String, Vec<String>>,
    inverse_link_graph: HashMap<String, Vec<String>>,
    url_to_id: HashMap<String, DocID>,
    id_to_url: HashMap<DocID, String>,

    incoming_neighbors: HashMap<DocID, Vec<DocID>>,
    outgoing_neighbors: HashMap<DocID, Vec<DocID>>,

    word_correlations: HashMap<String, Vec<(String, f32)>>,

    finished: Option<SystemTime>,
}

impl FTSIndex {
    pub fn new(fields: Vec<Field>) -> Self {
        Self {
            fields,
            trie: Trie::new(),
            terms: HashMap::new(),
            doc_id: 0,
            term_id: 0,

            documents: vec![],
            link_graph: HashMap::new(),
            inverse_link_graph: HashMap::new(),
            url_to_id: HashMap::new(),
            id_to_url: HashMap::new(),

            incoming_neighbors: hashmap![],
            outgoing_neighbors: hashmap![],

            word_correlations: HashMap::new(),

            finished: None,
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

    pub fn add<F>(&mut self, mut document: DocumentBuilder, on_token: F)
    where
        F: Fn(&str),
    {
        self.finished = None;

        let doc_id = self.doc_id;
        self.doc_id += 1;
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
        self.url_to_id.insert(document.url.to_owned(), doc_id);
        self.id_to_url.insert(doc_id, document.url.to_owned());

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
                    self.trie.insert(&token, doc_id);
                    index_entry.register(field.name.to_owned(), doc_id);
                }

                index_entry.add_token_position(doc_id, self.term_id);
            }

            // After each field, bump by one to prevent accidental adjacency.
            self.term_id += 1;

            field.total_tokens += number_of_tokens;
            field.documents.insert(
                doc_id,
                DocumentEntry::new(number_of_tokens, term_frequencies),
            );
        }

        for (token, prefix_size, closeness) in correlations {
            self.correlate_word(&token, &token[prefix_size as usize..], closeness);
        }

        self.documents.push(Document {
            _id: doc_id,
            url: document.url,
            weight: document.weight,
            data: document.data,
        });
    }

    pub fn finished_time(&self) -> SystemTime {
        self.finished.expect("Must call FTSIndex::finish()")
    }

    pub fn finish(&mut self) {
        self.outgoing_neighbors.clear();
        self.incoming_neighbors.clear();

        for field in &mut self.fields {
            field.compute_length_weight();
        }

        for (&doc_id, url) in &self.id_to_url {
            let mut outgoing_neighbors_set = HashSet::new();
            if let Some(links) = self.link_graph.get(url) {
                for link in links {
                    if let Some(descendent_id) = self.url_to_id.get(link) {
                        outgoing_neighbors_set.insert(descendent_id);
                    }
                }
            }

            let mut outgoing_neighbors: Vec<_> = outgoing_neighbors_set.drain().cloned().collect();
            self.outgoing_neighbors.insert(doc_id, outgoing_neighbors);

            let mut incoming_neighbors_set = HashSet::new();
            if let Some(urls) = self.inverse_link_graph.get(url) {
                for ancestor_url in urls {
                    if let Some(ancestor_id) = self.url_to_id.get(ancestor_url) {
                        incoming_neighbors_set.insert(ancestor_id);
                    }
                }
            }

            let incoming_neighbors: Vec<_> = incoming_neighbors_set.drain().cloned().collect();
            self.incoming_neighbors.insert(doc_id, incoming_neighbors);
        }

        self.finished = Some(SystemTime::now());
    }

    fn collect_matches_from_trie<'a, I>(&self, terms: I) -> Vec<(DocID, Vec<&str>)>
    where
        I: iter::Iterator<Item = &'a String>,
    {
        let mut result_set = vec![];
        for term in terms {
            for (doc_id, terms) in self.trie.search(term) {
                result_set.push((doc_id, terms.to_owned()));
            }
        }

        result_set
    }

    pub fn search(&self, query: Query) -> Vec<&Document> {
        if self.finished.is_none() {
            panic!("Must call FTSIndex::finish()")
        }

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

            for &term in terms {
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
                        * self.documents[doc_id as usize].weight;
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

        // Expand our root set's neighbors to create a base set: the set of all
        // relevant pages, as well as pages that link TO or are linked FROM those pages.
        let root_ids: Vec<DocID> = root_set.iter().map(|m| m._id).collect();
        let mut match_set = MatchSet::new();
        for search_match in root_set.drain(..) {
            match_set.insert(search_match, self);
        }

        match_set.finish(&root_ids);

        // Run HITS to re-sort our results based on authority
        match_set
            .hits(0.00001, 200)
            .iter()
            .map(|search_match| &self.documents[search_match._id as usize])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fts() {
        let mut index = FTSIndex::new(vec![Field::new("text", 1.0), Field::new("title", 10.0)]);

        index.add(
            DocumentBuilder {
                url: "https://en.wikipedia.org/wiki/Fox".to_owned(),
                links: vec!["https://en.wikipedia.org/wiki/Red_fox".to_owned()],
                weight: 1.0,
                data: hashmap![
                "text".to_owned() => r#"Foxes are small-to-medium-sized, omnivorous mammals belonging to several genera of the family Canidae. Foxes have a flattened skull, upright triangular ears, a pointed, slightly upturned snout, and a long bushy tail (or brush)."#.to_owned(),
                "title".to_owned() => "Fox".to_owned(),
            ],
            },
            |_doc| (),
        );

        index.add(
            DocumentBuilder {
                url: "https://en.wikipedia.org/wiki/Red_fox".to_owned(),
                links: vec![],
                weight: 1.0,
                data: hashmap![
                "text".to_owned() => r#"The red fox (Vulpes vulpes), largest of the true foxes, has the greatest geographic range of all members of the Carnivora order, being present across the entire Northern Hemisphere from the Arctic Circle to North Africa, North America and Eurasia. It is listed as least concern by the IUCN.[1] Its range has increased alongside human expansion, having been introduced to Australia, where it is considered harmful to native mammals and bird populations. Due to its presence in Australia, it is included among the list of the "world's 100 worst invasive species"."#.to_owned(),
                "title".to_owned() => "Red fox".to_owned(),
            ],
            },
            |_doc| (),
        );

        // index.add(Document {
        //     _id: 2,
        //     url: "Omnivore".to_owned(),
        //     links: vec![],
        //     weight: 1.0,
        //     data: hashmap![
        //         "text".to_owned() => r#"Omnivore /ˈɒmnivɔər/ is a consumption classification for animals that have the capability to obtain chemical energy and nutrients from materials originating from plant and animal origin. Often, omnivores also have the ability to incorporate food sources such as algae, fungi, and bacteria into their diet as well."#.to_owned(),
        //         "title".to_owned() => "Omnivore".to_owned(),
        //     ],
        // }, |_doc| ());

        index.finish();
        index.search(Query::new("fox carnivora", ""));
    }
}
