use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;
use serde_json;
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

    #[serde(skip)] pub url: String,
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
    #[serde(rename = "includeInGlobalSearch")] pub include_in_global_search: bool,

    #[serde(default)] pub aliases: Vec<String>,

    pub documents: Vec<ManifestDocument>,
    pub url: String,
}

pub struct Manifest {
    pub body: ManifestData,
    pub last_modified: SystemTime,
    pub search_property: String,
}

pub trait ManifestLoader: Send + Sync {
    fn load(&self) -> Result<Vec<Manifest>, String>;
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
    fn load(&self) -> Result<Vec<Manifest>, String> {
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

            manifests.push(Manifest {
                body: body,
                last_modified: mtime,
                search_property: String::new(),
            });
        }

        Ok(manifests)
    }
}

pub struct S3ManifestLoader {
    bucket: String,
}

impl S3ManifestLoader {
    pub fn new<S: Into<String>>(bucket: S) -> Self {
        Self {
            bucket: bucket.into(),
        }
    }
}

impl ManifestLoader for S3ManifestLoader {
    fn load(&self) -> Result<Vec<Manifest>, String> {
        Err(String::from("S3 manifest loader not yet implemented"))
    }
}

pub fn parse_manifest_source(source: &str) -> Result<Box<ManifestLoader>, String> {
    if source.starts_with("dir:") {
        Ok(Box::new(FileManifestLoader::new(&source[4..])))
    } else if source.starts_with("bucket:") {
        Ok(Box::new(S3ManifestLoader::new(&source[7..])))
    } else {
        Err(format!("Unknown manifest source protocol: {}", source))
    }
}
