use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde_derive::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::store::StorePath;
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

#[derive(Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Derivation {
    pub url: String,
    pub extract: bool,
    pub extract_target: Option<String>,
    pub name: String,
    pub file_name: String,
    pub hash: Option<String>,
    pub depends: Vec<String>,
    pub tags: Vec<String>,
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
        Ok(Self::from_raw(raw_derivation))
    }

    fn from_raw(derivation: RawDerivation) -> Self {
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
                url_extracted_name
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
                    name.clone()
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
        }
    }

    /// blocking, so should be ran from threads
    /// returns downloaded file path in cache
    pub fn download(&mut self, tmp: &str) -> Result<String, String> {
        let path = format!("{tmp}/{}", self.file_name);
        let mut file = fs::File::create(&path)
            .map_err(|e| format!("failed to create temporary file `{path}`: {e}"))?;

        let response = reqwest::blocking::get(&self.url)
            .map_err(|e| format!("failed to download artifact for {}: {e}", self.name))?;

        let bytes = response
            .bytes()
            .map_err(|e| format!("failed to read downloaded data for {}: `{e}`", self.name))?;
        self.hash = Some(hash_stream(&bytes));
        file.write_all(&bytes);
        Ok(path)
    }

    pub fn extract_package(&self, cache_file_path: &str) -> Result<String, String> {
        // let file = File::open(cache_file_path).map_err(|e|format!("failed to open downloaded archive `{cache_file_path}`: {e}"))?;
        let dest = format!("{cache_file_path}.extracted");
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

    pub fn install_to_store(&self, store: &str, cache_f: &str) -> Result<StorePath, String> {
        let store_path = StorePath::new(&format!("{store}/{}/", self.generate_hash_signature()));
        let install_path = store_path.get_artifact();
        fs::rename(cache_f, install_path)
            .map_err(|e| format!("failed to install `{}` to store: {e}", self.name))?;
        Ok(store_path)
    }

    pub fn write_back(&self, path: &str) -> Result<(), String> {
        let serialized = toml::to_string(&self)
            .map_err(|e| format!("failed to serialize derivation for {}: {e}", self.name))?;
        let mut file = File::open(path)
            .map_err(|e| format!("failed to open derivation {path} for write-back: `{e}`"))?;
        file.write_all(&serialized.as_bytes());
        Ok(())
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
fn load_derivations_from_directory(dir: &Path) -> Result<Vec<Derivation>, String> {
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
