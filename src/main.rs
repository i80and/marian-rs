extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate mime;
extern crate num_cpus;
extern crate qp_trie;
extern crate regex;
#[macro_use]
extern crate serde_json;
extern crate unicase;

mod fts;
mod porter2;
mod query;
mod stemmer;
mod trie;

use std::mem;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use futures::future::Future;
use futures_cpupool::CpuPool;
use hyper::header::{self, HttpDate};
use hyper::server::{Http, Request, Response, Service};
use hyper::{Method, StatusCode};
use regex::Regex;
use unicase::Ascii;
use fts::{DocumentBuilder, FTSIndex};
use query::Query;

const MAXIMUM_QUERY_LENGTH: usize = 100;


lazy_static! {
    static ref PAT_QUERY_STRING: Regex = Regex::new(r#"([:alnum:]+)=([^&]*)"#)
        .expect("Failed to compile query string regex");
}

fn parse_query(queryst: &str) -> HashMap<&str, &str> {
    let mut result = HashMap::new();
    for group in PAT_QUERY_STRING.captures_iter(queryst) {
        let key = group.get(1).map_or("", |m| m.as_str());
        let value = group.get(2).map_or("", |m| m.as_str());
        result.insert(key, value);
    }

    result
}

fn default_fields() -> Vec<fts::Field> {
    vec![
        fts::Field::new("text", 1.0),
        fts::Field::new("headings", 5.0),
        fts::Field::new("title", 10.0),
        fts::Field::new("tags", 75.0),
    ]
}

fn handle_search(index: &Arc<RwLock<FTSIndex>>, request: &Request) -> Response {
    let query = match request.query() {
        Some(fq) => fq,
        None => {
            return Response::new().with_status(StatusCode::BadRequest);
        }
    };

    if query.len() > MAXIMUM_QUERY_LENGTH {
        return Response::new().with_status(StatusCode::BadRequest);
    }

    let query = parse_query(query);

    let search_query = match query.get("q") {
        Some(s) => s,
        None => {
            return Response::new().with_status(StatusCode::BadRequest);
        }
    };

    let search_properties = query.get("searchProperties").unwrap_or(&"");

    let txn = index.read().unwrap();
    let response = Response::new()
        .with_header(header::LastModified(HttpDate::from(txn.finished_time())))
        .with_header(header::ContentType(mime::APPLICATION_JSON))
        .with_header(header::Vary::Items(
            vec![Ascii::new("Accept-Encoding".to_owned())],
        ))
        .with_header(header::CacheControl(vec![
            header::CacheDirective::Public,
            header::CacheDirective::MaxAge(120),
            header::CacheDirective::MustRevalidate,
        ]))
        .with_header(header::AccessControlAllowOrigin::Any);

    let parsed_query = Query::new(search_query, search_properties);

    let results: Vec<serde_json::Value> = txn.search(parsed_query)
        .iter()
        .map(|doc| {
            json![{
                    "title": doc.title(),
                    "preview": doc.preview(),
                    "url": &doc.url
                }]
        })
        .collect();

    let serialized = serde_json::to_string(&results).unwrap();
    response.with_body(serialized)
}

fn handle_refresh(index: &Arc<RwLock<FTSIndex>>) -> Response {
    let mut new_index = FTSIndex::new(default_fields());
    new_index.add(
        DocumentBuilder::new("https://foxquill.com".to_owned()),
        |_token| (),
    );
    new_index.finish();

    let mut txn = index.write().unwrap();
    mem::replace(&mut *txn, new_index);
    Response::new()
}

struct MarianService {
    index: Arc<RwLock<FTSIndex>>,
    workers: CpuPool,
}

impl MarianService {
    fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(FTSIndex::new(default_fields()))),
            workers: CpuPool::new(num_cpus::get()),
        }
    }

    fn status(&self) -> Response {
        let serialized = serde_json::to_string(&json![{}]).unwrap();
        Response::new()
            .with_header(header::ContentType(mime::APPLICATION_JSON))
            .with_header(header::Vary::Items(
                vec![Ascii::new("Accept-Encoding".to_owned())],
            ))
            .with_body(serialized)
    }
}

impl Service for MarianService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let response = match (req.method(), req.path()) {
            (&Method::Get, "/search") => {
                let index = Arc::clone(&self.index);
                return Box::new(self.workers.spawn_fn(move || {
                    Box::new(futures::future::ok(handle_search(&index, &req)))
                }));
            }
            (&Method::Get, "/status") => self.status(),
            (&Method::Post, "/refresh") => {
                let index = Arc::clone(&self.index);
                return Box::new(self.workers.spawn_fn(move || {
                    Box::new(futures::future::ok(handle_refresh(&index)))
                }));
            }
            (_, "/search") | (_, "/status") | (_, "/refresh") => {
                Response::new().with_status(StatusCode::MethodNotAllowed)
            }
            _ => Response::new().with_status(StatusCode::NotFound),
        };

        Box::new(futures::future::ok(response))
    }
}


fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new()
        .bind(&addr, || Ok(MarianService::new()))
        .unwrap();
    server.run().unwrap();
}
