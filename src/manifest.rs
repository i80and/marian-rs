use futures::{Future, Stream};
use hyper::header::HttpDate;
use rusoto_core;
use rusoto_s3::{self, S3};
use serde_json;
use std::borrow::Cow;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Deserialize)]
pub struct ManifestDocument {
    pub slug: String,
    pub title: String,
    pub tags: String,
    pub headings: Vec<String>,
    pub text: String,
    pub preview: String,
    pub links: Vec<String>,

    #[serde(skip)]
    pub url: String,
}

impl ManifestDocument {
    pub fn get(&self, key: &str) -> Option<Cow<str>> {
        match key {
            "title" => Some(Cow::Borrowed(&self.title)),
            "text" => Some(Cow::Borrowed(&self.text)),
            "headings" => Some(Cow::Owned(self.headings.join(" "))),
            "tags" => Some(Cow::Borrowed(&self.tags)),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
pub struct ManifestData {
    #[serde(rename = "includeInGlobalSearch")]
    pub include_in_global_search: bool,

    #[serde(default)]
    pub aliases: Vec<String>,

    pub documents: Vec<ManifestDocument>,
    pub url: String,
}

pub struct Manifest {
    pub body: ManifestData,
    pub last_modified: SystemTime,
    pub search_property: String,
}

pub struct ManifestError {
    pub search_property: String,
    pub message: String,
}

impl ManifestError {
    fn new<S1: Into<String>, S2: Into<String>>(search_property: S1, message: S2) -> Self {
        Self {
            search_property: search_property.into(),
            message: message.into(),
        }
    }

    fn new_from_err<S: Into<String>>(search_property: S, src_error: &Error) -> Self {
        Self {
            search_property: search_property.into(),
            message: src_error.description().to_owned(),
        }
    }
}

pub trait ManifestLoader: Send + Sync {
    fn load(&self) -> Result<Vec<Result<Manifest, ManifestError>>, String>;
    fn parts(&self) -> Vec<String>;
}

pub struct FileManifestLoader {
    path: PathBuf,
}

impl FileManifestLoader {
    pub fn new<S: Into<PathBuf>>(path: S) -> Self {
        Self { path: path.into() }
    }
}

impl ManifestLoader for FileManifestLoader {
    fn load(&self) -> Result<Vec<Result<Manifest, ManifestError>>, String> {
        let mut manifests = vec![];

        for entry in WalkDir::new(&self.path) {
            let entry = entry.or_else(|_| {
                Err(format!(
                    "Error scanning input directory: {}",
                    &self.path.display()
                ))
            })?;
            let metadata = entry.metadata().or_else(|_| {
                Err(format!(
                    "Failed to get metadata of manifest: {}",
                    &entry.path().display()
                ))
            })?;
            if !metadata.is_file() {
                continue;
            }

            let mtime = metadata.modified().or_else(|_| {
                Err(format!(
                    "Failed to get mtime of file: {}",
                    &entry.path().display()
                ))
            })?;
            let mut file = File::open(&entry.path()).or_else(|_| {
                Err(format!(
                    "Failed to open manifest file: {}",
                    &entry.path().display()
                ))
            })?;
            let mut body_string = String::with_capacity(metadata.len() as usize);
            file.read_to_string(&mut body_string).or_else(|_| {
                Err(format!(
                    "Failed to read manifest file: {}",
                    &entry.path().display(),
                ))
            })?;
            let body = serde_json::from_str(&body_string).or_else(|msg| {
                Err(format!(
                    "Failed to parse manifest file: {}\n{}",
                    &entry.path().display(),
                    msg
                ))
            })?;

            let search_property = match entry.path().file_stem() {
                Some(stem) => stem.to_string_lossy().to_string(),
                None => String::new(),
            };

            manifests.push(Ok(Manifest {
                body,
                last_modified: mtime,
                search_property,
            }));
        }

        Ok(manifests)
    }

    fn parts(&self) -> Vec<String> {
        return vec![self.path.to_string_lossy().into_owned()];
    }
}

pub struct S3ManifestLoader {
    bucket: String,
    prefix: String,
}

impl S3ManifestLoader {
    pub fn new(src: &str) -> Result<Self, ()> {
        let parts = src.splitn(2, '/').collect::<Vec<_>>();
        let bucket_name = parts[0].trim();
        let prefix = parts[1].trim();
        if bucket_name.is_empty() || prefix.is_empty() {
            return Err(());
        }

        Ok(Self {
            bucket: bucket_name.to_owned(),
            prefix: prefix.to_owned(),
        })
    }
}

impl ManifestLoader for S3ManifestLoader {
    fn load(&self) -> Result<Vec<Result<Manifest, ManifestError>>, String> {
        let client = rusoto_s3::S3Client::simple(rusoto_core::region::Region::default());
        let mut request = rusoto_s3::ListObjectsV2Request::default();
        request.bucket = self.bucket.to_owned();
        request.prefix = Some(self.prefix.to_owned());
        let response = client
            .list_objects_v2(&request)
            .sync()
            .map_err(|err| err.description().to_owned())?;
        if response.is_truncated == Some(true) {
            // This would indicate something awry, since we shouldn't
            // ever have more than 1000 properties. And if we ever did,
            // everything would need to be rearchitected.
            return Err(String::from("Got truncated response from S3"));
        }

        let mut objects = response.contents.unwrap_or_else(|| vec![]);
        let manifests: Vec<Result<Manifest, ManifestError>> = objects
            .drain(..)
            .filter(|object| {
                // Skip redirects and other weird-looking files
                object.size != None && object.size != Some(0)
            })
            .map(|object| {
                let key = object
                    .key
                    .ok_or_else(|| ManifestError::new("<unknown>", "S3 object lacked a key"))?;

                let search_property = {
                    let key_path = Path::new(&key);
                    let stem = key_path
                        .file_stem()
                        .ok_or_else(|| ManifestError::new(key.as_ref(), "Missing file stem"))?;
                    stem.to_string_lossy().to_string()
                };

                let mut get_request = rusoto_s3::GetObjectRequest::default();
                get_request.bucket = self.bucket.to_owned();
                get_request.key = key;

                let response = client
                    .get_object(&get_request)
                    .sync()
                    .map_err(|err| ManifestError::new_from_err(get_request.key.as_ref(), &err))?;
                let body = response.body.ok_or_else(|| {
                    ManifestError::new(get_request.key.as_ref(), "Missing response body")
                })?;
                let body = body
                    .concat2()
                    .wait()
                    .map_err(|err| ManifestError::new_from_err(get_request.key.as_ref(), &err))?;
                let body = String::from_utf8(body)
                    .map_err(|err| ManifestError::new_from_err(get_request.key.as_ref(), &err))?;
                let body = serde_json::from_str(&body)
                    .map_err(|err| ManifestError::new_from_err(get_request.key.as_ref(), &err))?;

                let mtime = match object.last_modified {
                    Some(s) => HttpDate::from_str(&s).ok(),
                    _ => None,
                }.map(|d| SystemTime::from(d))
                    .unwrap_or_else(|| SystemTime::now());

                Ok(Manifest {
                    body,
                    last_modified: mtime,
                    search_property,
                })
            })
            .collect();

        Ok(manifests)
    }

    fn parts(&self) -> Vec<String> {
        return vec![self.bucket.to_owned(), self.prefix.to_owned()];
    }
}

pub fn parse_manifest_source(source: &str) -> Result<Box<ManifestLoader>, String> {
    if source.starts_with("dir:") {
        Ok(Box::new(FileManifestLoader::new(&source[4..])))
    } else if source.starts_with("bucket:") {
        match S3ManifestLoader::new(&source[7..]) {
            Ok(loader) => Ok(Box::new(loader)),
            Err(_) => Err(String::from("Invalid S3 source format")),
        }
    } else {
        Err(format!("Unknown manifest source protocol: {}", source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bucket() {
        assert_eq!(
            parse_manifest_source("bucket:example/a/prefix")
                .unwrap()
                .parts(),
            vec!["example".to_owned(), "a/prefix".to_owned()]
        );
    }

    #[test]
    fn test_unknown_protocol() {
        assert!(parse_manifest_source("di:foobar").is_err());
    }
}
