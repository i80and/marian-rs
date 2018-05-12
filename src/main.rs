extern crate brotli2;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
extern crate mime;
extern crate num_cpus;
extern crate percent_encoding;
extern crate qp_trie;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate simple_logging;
extern crate smallvec;
extern crate time;
extern crate unicase;
extern crate walkdir;

mod fts;
mod manifest;
mod porter2;
mod protocol;
mod query;
mod queryst;
mod stemmer;
mod trie;

use brotli2::read::BrotliEncoder;
use fts::FTSIndex;
use futures::future::Future;
use futures_cpupool::CpuPool;
use hyper::header::{self, HttpDate, IfModifiedSince};
use hyper::server::{Http, NewService, Request, Response, Service};
use hyper::{Method, StatusCode};
use manifest::ManifestLoader;
use percent_encoding::percent_decode;
use query::Query;
use queryst::parse_query;
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use std::{env, mem, process};
use unicase::Ascii;

const MAXIMUM_QUERY_LENGTH: usize = 100;

fn timespec_from(st: &SystemTime) -> time::Timespec {
    if let Ok(dur_since_epoch) = st.duration_since(std::time::UNIX_EPOCH) {
        time::Timespec::new(
            dur_since_epoch.as_secs() as i64,
            dur_since_epoch.subsec_nanos() as i32,
        )
    } else {
        time::Timespec::new(0, 0)
    }
}

/// Find an acceptable compression format for the client, and return a compressed
/// version of the content if possible. Otherwise return the original input text.
fn compress(response: Response, req: &Request, content: String) -> Response {
    let accept_encodings = match req.headers().get::<header::AcceptEncoding>() {
        Some(h) => h,
        None => return response.with_body(content),
    };

    for quality_item in accept_encodings.iter() {
        if quality_item.quality == header::q(0) {
            continue;
        }

        if quality_item.item == header::Encoding::Brotli {
            let mut compressed = Vec::with_capacity(content.len());
            let mut encoder = BrotliEncoder::new(content.as_bytes(), 6);
            if encoder.read_to_end(&mut compressed).is_err() {
                return response.with_status(StatusCode::InternalServerError);
            }
            let response =
                response.with_header(header::ContentEncoding(vec![header::Encoding::Brotli]));
            return response.with_body(compressed);
        }
    }

    response.with_body(content)
}

fn default_fields() -> Vec<fts::Field> {
    vec![
        fts::Field::new("text", 1.0),
        fts::Field::new("headings", 5.0),
        fts::Field::new("title", 10.0),
        fts::Field::new("tags", 75.0),
    ]
}

fn handle_search(marian: &Marian, request: &Request) -> Response {
    let query = match request.query() {
        Some(fq) => fq,
        None => {
            return Response::new().with_status(StatusCode::BadRequest);
        }
    };

    let query = match percent_decode(query.as_bytes()).decode_utf8() {
        Ok(q) => q,
        Err(_) => {
            return Response::new().with_status(StatusCode::BadRequest);
        }
    };

    if query.len() > MAXIMUM_QUERY_LENGTH {
        return Response::new().with_status(StatusCode::BadRequest);
    }

    let query = parse_query(query.as_ref());
    let search_query = match query.get("q") {
        Some(s) => s,
        None => {
            return Response::new().with_status(StatusCode::BadRequest);
        }
    };

    let txn = marian.index.read().unwrap();

    if let Some(header) = request.headers().get::<IfModifiedSince>() {
        let if_modified_since = timespec_from(&SystemTime::from(header.0));

        // HTTP dates truncate the milliseconds.
        let mut last_sync_date = txn.finished;
        last_sync_date.nsec = 0;

        if if_modified_since >= last_sync_date {
            return Response::new().with_status(StatusCode::NotModified);
        }
    }

    let search_properties: Vec<_> = query
        .get("searchProperties")
        .unwrap_or(&"")
        .split(',')
        .collect();
    let finished_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(0);
    let response = Response::new()
        .with_header(header::LastModified(HttpDate::from(finished_time)))
        .with_header(header::ContentType(mime::APPLICATION_JSON))
        .with_header(header::Vary::Items(vec![Ascii::new(
            "Accept-Encoding".to_owned(),
        )]))
        .with_header(header::CacheControl(vec![
            header::CacheDirective::Public,
            header::CacheDirective::MaxAge(120),
            header::CacheDirective::MustRevalidate,
        ]))
        .with_header(header::AccessControlAllowOrigin::Any);

    let parsed_query = Query::new(search_query, &search_properties);

    let results: Vec<serde_json::Value> = txn.search(&parsed_query)
        .iter()
        .map(|doc| {
            json![{
                    "title": doc.title,
                    "preview": doc.preview,
                    "url": &doc.url
                }]
        })
        .collect();

    let results = json![{ "results": results }];

    let serialized = serde_json::to_string(&results).unwrap();
    compress(response, request, serialized)
}

fn handle_refresh(marian: &Marian) -> Result<(), String> {
    let manifest_loader = &*marian.manifest_loader;

    let mut manifests = manifest_loader.load()?;
    let mut new_index = FTSIndex::new(default_fields());

    for manifest in &mut manifests {
        while manifest.body.url.ends_with('/') {
            manifest.body.url.pop();
        }

        for alias in manifest.body.aliases.drain(..) {
            new_index.alias_search_property(alias.to_owned(), manifest.search_property.to_owned());
        }

        let include_in_global_search = manifest.body.include_in_global_search;
        let search_property = manifest.search_property.to_owned();

        for mut doc in manifest.body.documents.drain(..) {
            while doc.slug.ends_with('/') {
                doc.slug.pop();
            }
            doc.url = format!("{}/{}", manifest.body.url, doc.slug);
            new_index.add(doc, include_in_global_search, search_property.to_owned());
        }
    }

    new_index.finish();

    let mut txn = marian.index.write().unwrap();
    mem::replace(&mut *txn, new_index);
    Ok(())
}

pub struct Marian {
    index: RwLock<FTSIndex>,
    workers: CpuPool,
    manifest_loader: Box<ManifestLoader>,
}

impl Marian {
    fn new(manifest_loader: Box<ManifestLoader>) -> Result<Self, String> {
        let service = Self {
            index: RwLock::new(FTSIndex::new(default_fields())),
            workers: CpuPool::new(num_cpus::get()),
            manifest_loader,
        };

        handle_refresh(&service)?;

        Ok(service)
    }
}

struct MarianServiceFactory {
    pub marian: Arc<Marian>,
}

impl NewService for MarianServiceFactory {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Instance = MarianService;

    fn new_service(&self) -> Result<Self::Instance, std::io::Error> {
        Ok(MarianService {
            ctx: Arc::clone(&self.marian),
        })
    }
}

struct MarianService {
    ctx: Arc<Marian>,
}

impl MarianService {
    fn status(&self) -> Response {
        let serialized = protocol::create_status_string(&*self.ctx);

        Response::new()
            .with_header(header::ContentType(mime::APPLICATION_JSON))
            .with_header(header::Vary::Items(vec![Ascii::new(
                "Accept-Encoding".to_owned(),
            )]))
            .with_body(serialized)
    }
}

impl Service for MarianService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let response =
            match (req.method(), req.path()) {
                (&Method::Get, "/search") => {
                    let marian = Arc::clone(&self.ctx);
                    return Box::new(self.ctx.workers.spawn_fn(move || {
                        Box::new(futures::future::ok(handle_search(&marian, &req)))
                    }));
                }
                (&Method::Get, "/status") => self.status(),
                (&Method::Post, "/refresh") => {
                    let marian = Arc::clone(&self.ctx);
                    return Box::new(self.ctx.workers.spawn_fn(move || {
                        let response = match handle_refresh(&marian) {
                            Ok(_) => Response::new(),
                            Err(msg) => {
                                error!("Error loading manifests: {}", msg);
                                Response::new().with_status(StatusCode::InternalServerError)
                            }
                        };
                        Box::new(futures::future::ok(response))
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

fn usage(exit_code: i32) -> ! {
    eprintln!("Usage: marian-rust <dir|bucket>:<...>");
    process::exit(exit_code);
}

fn main() {
    simple_logging::log_to_stderr(log::LevelFilter::Info);

    let manifest_source = match env::args().nth(1) {
        Some(s) => s,
        None => usage(1),
    };

    let manifest_source = match manifest::parse_manifest_source(&manifest_source) {
        Ok(s) => s,
        Err(msg) => {
            error!("{}", msg);
            usage(1)
        }
    };

    let marian = match Marian::new(manifest_source) {
        Ok(m) => m,
        Err(msg) => {
            error!("{}", msg);
            process::exit(1)
        }
    };

    let factory = MarianServiceFactory {
        marian: Arc::new(marian),
    };

    let interface = "127.0.0.1:3000";
    info!("Listening on http://{}", interface);
    let addr = interface.parse().unwrap();
    let server = Http::new().bind(&addr, factory).unwrap();
    server.run().unwrap();
}
