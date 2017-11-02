extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::collections::HashMap;

mod fts;
mod query;
mod trie;

fn main() {
    let mut fts = fts::FTSIndex::new(vec![
        fts::Field::new("text", 1.0),
        fts::Field::new("headings", 5.0),
        fts::Field::new("title", 10.0),
        fts::Field::new("tags", 75.0),
    ]);

    fts.add(
        fts::Document {
            _id: 0,
            url: String::from("https://foxquill.com"),
            links: vec![],
            weight: 1.0,
            data: HashMap::new(),
        },
        |_| (),
    );

    fts.finish();

    fts.search(query::Query::new(), true);
}
