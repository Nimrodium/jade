use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde_derive::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    store::{self, Store, StorePath},
    util::{self, normalize},
};
#[derive(Deserialize)]
pub struct RawDerivation {
    url: String,
    extract: Option<bool>,
    extract_target: Option<String>,
    name: Option<String>,
    file_name: Option<String>,
    hash: Option<String>,
    depends: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}
fn is_false(b: &bool) -> bool {
    !b
}
#[derive(Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Derivation {
    pub url: String,
    pub name: String,
    pub file_name: String,
    #[serde(skip_serializing_if = "is_false")]
    pub extract: bool,
    pub extract_target: Option<String>,
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depends: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing)]
    pub backing_file: String,
}

impl Derivation {
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let mut contents = String::new();
        let mut file = File::open(path)
            .map_err(|e| format!("failed to open derivation `{}`: {e}", path.display()))?;
        file.read_to_string(&mut contents)
            .map_err(|e| format!("failed to read derivation `{}`: {e}", path.display()))?;
        let raw_derivation: RawDerivation = toml::from_str(&contents)
            .map_err(|e| format!("failed to parse derivation `{}`: {e}", path.display()))?;
        Ok(Self::from_raw(raw_derivation, &path.display().to_string()))
    }

    fn from_raw(derivation: RawDerivation, p: &str) -> Self {
        let url_extracted_name = derivation
            .url
            .rsplit_once('/')
            .expect("invalid url")
            .1
            .to_string();

        let name = if let Some(name) = derivation.name {
            name
        } else {
            if let Some(file_name) = &derivation.file_name {
                file_name.clone()
            } else {
                url_extracted_name.clone()
            }
        };

        Self {
            url: derivation.url,
            extract: if let Some(extract) = derivation.extract {
                extract
            } else {
                false
            },
            extract_target: derivation.extract_target,
            file_name: {
                if let Some(file_name) = derivation.file_name {
                    file_name.clone()
                } else {
                    url_extracted_name
                }
            },
            name,
            hash: derivation.hash,
            depends: if let Some(depends) = derivation.depends {
                depends
            } else {
                vec![]
            },
            // side: if let Some(side) = derivation.side {
            //     side
            // } else {
            //     "both".to_string()
            // },
            tags: if let Some(tags) = derivation.tags {
                tags
            } else {
                vec![]
            },
            backing_file: p.to_string(),
        }
    }

    /// blocking, so should be ran from threads
    /// returns downloaded file path in cache
    pub fn download(&mut self, tmp: &str) -> Result<String, String> {
        let path = format!("{tmp}/{}", self.file_name);
        println!("downloading {} to {path}", self.url);
        let mut file = fs::File::create(&path)
            .map_err(|e| format!("failed to create temporary file `{path}`: {e}"))?;

        let response = reqwest::blocking::get(&self.url)
            .map_err(|e| format!("failed to download artifact for {}: {e}", self.name))?;

        let bytes = response
            .bytes()
            .map_err(|e| format!("failed to read downloaded data for {}: `{e}`", self.name))?;
        self.hash = Some(hash_stream(&bytes));
        file.write_all(&bytes)
            .map_err(|e| format!("failed to write to disk `{path}`: {e}"))?;
        Ok(path)
    }

    pub fn extract_package(&self, cache_file_path: &str) -> Result<String, String> {
        // let file = File::open(cache_file_path).map_err(|e|format!("failed to open downloaded archive `{cache_file_path}`: {e}"))?;
        let dest = format!("{cache_file_path}.extracted");
        println!("extracting {cache_file_path} to {dest}");
        zip_extensions::zip_extract(
            &Path::new(cache_file_path).to_path_buf(),
            &Path::new(&dest).to_path_buf(),
        )
        .map_err(|e| format!("failed to extract downloaded archive `{cache_file_path}`: {e}"))?;
        Ok(dest)
    }

    pub fn generate_hash_signature(&self) -> String {
        format!("{}-{}", self.hash.as_ref().unwrap(), self.name)
    }

    pub fn install_to_store(&self, store: &Store, cache_f: &str) -> Result<StorePath, String> {
        let store_path = store.make_package_store_path(self);
        fs::create_dir_all(store_path.to_string())
            .map_err(|e| format!("failed to create store path `{store_path}`: {e}"))?;
        println!("installing {} to store (`{}`)", self.name, store.store_path);
        let install_path = store_path.get_artifact();
        fs::rename(cache_f, install_path).map_err(|e| {
            format!(
                "failed to install `{}`(`{cache_f}`) to store: {e}",
                self.name
            )
        })?;
        Ok(store_path)
    }

    pub fn write_back(&self) -> Result<(), String> {
        let serialized = toml::to_string(&self)
            .map_err(|e| format!("failed to serialize derivation for {}: {e}", self.name))?;
        let mut file = File::create(&self.backing_file).map_err(|e| {
            format!(
                "failed to open derivation {} for write-back: `{e}`",
                self.backing_file
            )
        })?;
        file.write_all(&serialized.as_bytes()).map_err(|e| {
            format!(
                "failed to write back derivation file `{}`: {e}",
                self.backing_file
            )
        })?;
        Ok(())
    }
    fn normalize_name(&mut self) {
        self.name = util::normalize(&self.name);
    }
}

fn hash_file(f: &str) -> Result<String, String> {
    let mut file =
        File::open(f).map_err(|e| format!("failed to open file `{f}` for hashing: {e}"))?;
    let mut bytes = Vec::<u8>::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("failed to read file `{f}` for hashing: {e}"))?;

    Ok(hash_stream(&bytes))
}

fn hash_stream(byte_stream: &[u8]) -> String {
    nix_base32::to_nix_base32(&Sha256::digest(byte_stream)[..])
}

// fn extract_zip(p: &str) -> Result<String, String> {}

/// recursively load derivations
pub fn load_derivations_from_directory(dir: &Path) -> Result<Vec<Derivation>, String> {
    let display_dir = dir.display();
    let mut derivations = Vec::<Derivation>::new();
    if !dir.is_dir() {
        return Err(format!("{display_dir} is not a directory"));
    }
    for result in dir
        .read_dir()
        .map_err(|e| format!("failed to read directory {display_dir}: {e}"))?
    {
        let entry = result.map_err(|e| format!("failed to read directory {display_dir}: {e}"))?;
        if entry.path().is_dir() {
            // enter subdir
            let sub_derivations = load_derivations_from_directory(&entry.path())?;
            derivations.extend(sub_derivations);
        } else if entry.path().is_file() {
            derivations.push(Derivation::load(&entry.path())?);
        }
    }
    Ok(derivations)
}

pub struct Derivations {
    pub derivations: Vec<Derivation>,
}
impl Derivations {
    pub fn new(derivations: Vec<Derivation>) -> Self {
        Self { derivations }
    }
    pub fn load_derivations_from_directory(dir_s: &str) -> Result<Self, String> {
        let dir = Path::new(dir_s);
        let display_dir = dir.display();
        let mut derivations = Vec::<Derivation>::new();
        if !dir.is_dir() {
            return Err(format!("{display_dir} is not a directory"));
        }
        for result in dir
            .read_dir()
            .map_err(|e| format!("failed to read directory {display_dir}: {e}"))?
        {
            let entry =
                result.map_err(|e| format!("failed to read directory {display_dir}: {e}"))?;
            if entry.path().is_dir() {
                // enter subdir
                let sub_derivations = load_derivations_from_directory(&entry.path())?;
                derivations.extend(sub_derivations);
            } else if entry.path().is_file() {
                derivations.push(Derivation::load(&entry.path())?);
            }
        }
        Ok(Self::new(derivations))
    }
    pub fn dedup(&mut self) {
        let mut tmp = HashSet::<Derivation>::new();
        for derivation in std::mem::take(&mut self.derivations) {
            tmp.insert(derivation);
        }
        self.derivations = tmp.into_iter().collect();
    }
    pub fn get_derivation_by_fuzzy_name(&self, name: &str) -> Result<&Derivation, String> {
        let mut found_derivation = None;
        let normalized_name = normalize(name);
        for derivation in &self.derivations {
            if util::normalize(&derivation.name).contains(&normalized_name) {
                found_derivation = Some(derivation);
                break;
            }
        }
        if let Some(derivation) = found_derivation {
            Ok(derivation)
        } else {
            return Err(format!("{name} could not be found"));
        }
    }
}

// impl IntoIterator for Derivations {
//     type Item = Derivation;

//     type IntoIter = std::vec::IntoIter<Derivation>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.derivations.into_iter()
//     }
// }
// fn get_derivation_by_fuzzy_name(name:&str,derivations:&Derivation)
