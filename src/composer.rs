use crate::{
    package::{JadeFlake, JadeLock},
    packwiz_compat::PackWizMod,
};
use nix_base32::{self, to_nix_base32};
use reqwest;
use serde_derive::Deserialize;
use sha2::{Digest, Sha256, Sha512};
use std::{
    fs::{self, File},
    io::{Cursor, Read},
    path::Path,
};
use toml;
use zip_extensions;
type DownloadResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Debug)]
pub enum JadeError {
    // HTTPError{
    //   error:String,
    // },
    SyntaxError {
        file: String,
        error: String,
    },
    Error(String),
    FailedDownload {
        url: String,
        return_code: u16,
        package: String,
    },
    CheckSumError {
        expected_sum: String,
        real_sum: String,
        package: String,
    },
    DependencyMissingError {
        package: String,
        dependencies: Vec<String>,
    },
    IOError {
        package: String,
        io_error: String,
    },
}

enum HashFmt {
    Sha256,
    Sha512,
}
impl HashFmt {
    fn from_string(s: &str) -> Result<Self, JadeError> {
        match s {
            "sha256" => Ok(HashFmt::Sha256),
            "sha512" => Ok(HashFmt::Sha512),
            _ => {
                return Err(JadeError::Error(format!(
                    "{} is not a supported hash format, supported are sha256, sha512",
                    s,
                )));
            }
        }
    }

    fn check(&self, f: &str, expected_hash: &str) -> Result<String, JadeError> {
        let real_hash = self.hash(f)?;
        if real_hash == expected_hash {
            Ok(real_hash)
        } else {
            Err(JadeError::CheckSumError {
                expected_sum: expected_hash.to_string(),
                real_sum: real_hash,
                package: f.to_string(),
            })
        }
    }

    // fn checksum(&self,pkgname:&str,expected:&str,actual:&str) -> Result<(),JadeError>
    fn hash(&self, f: &str) -> Result<String, JadeError> {
        // returns hash
        let mut file = File::open(f).map_err(|e| JadeError::IOError {
            package: f.to_string(),
            io_error: e.to_string(),
        })?;
        let mut bytes = Vec::<u8>::new();
        file.read_to_end(&mut bytes)
            .map_err(|e| JadeError::IOError {
                package: f.to_string(),
                io_error: e.to_string(),
            })?;

        Ok(match self {
            HashFmt::Sha256 => to_nix_base32(&Sha256::digest(bytes)[..]),
            HashFmt::Sha512 => to_nix_base32(&Sha512::digest(bytes)[..]),
        })
    }
}

enum PackageFmt {
    File,
    Zip,
}

pub struct Package {
    name: String,
    file_name: String,
    url: String,
    dep: Vec<String>,
    hash_format: HashFmt,
    hash: Option<String>,
    tags: Option<Vec<String>>, // dest: String,
    dest: Option<String>,
    target: Option<String>,
    package_format: PackageFmt, // how to postprocess artifact
}

impl Package {
    pub fn from_packwiz(pack_wiz: PackWizMod) -> Result<Self, JadeError> {
        let url = pack_wiz.download.url;
        let hash_format = HashFmt::from_string(&pack_wiz.download.hash_format)?;
        let hash = pack_wiz.download.hash;
        let dep = vec![];
        let name = pack_wiz.name;
        Ok(Package {
            name,
            file_name: pack_wiz.filename,
            url,
            dep,
            hash_format: hash_format,
            hash: Some(hash),
            tags: None,
            dest: None,
            target: None,
            package_format: PackageFmt::File,
        })
    }
    pub fn from_jade_flake(
        jade_flake: JadeFlake,
        jade_lock: &Option<JadeLock>,
    ) -> Result<Self, JadeError> {
        let hash = if let Some(jade_lock) = jade_lock {
            jade_lock.get_lock(&jade_flake.mod_table.name)
        } else {
            println!("could not find {} in lock file", jade_flake.mod_table.name);
            None
        };

        Ok(Package {
            name: jade_flake.mod_table.name,
            file_name: jade_flake.download.filename,
            url: jade_flake.download.url,
            dep: jade_flake.mod_table.depends,
            hash_format: HashFmt::from_string(&jade_flake.download.hash_format)?,
            hash: hash,
            tags: jade_flake.mod_table.tags,
            target: jade_flake.download.target,
            dest: jade_flake.download.dest,
            // package_format: PackageFmt::from_string(&jade_flake.download.format)?,
            package_format: PackageFmt::Zip,
        })
    }

    // https://georgik.rocks/how-to-download-binary-file-in-rust-by-reqwest/
    pub async fn fetch_package(
        &mut self,
        client: &reqwest::Client,
        jade_root: &str,
        install_to_store: bool,
        // verify: bool,
        lock_file: &mut Option<JadeLock>,
    ) -> Result<String, JadeError> {
        // check if already present in the store
        if let Some(store_location) = self.get_store_location(jade_root) {
            if Path::new(&store_location).exists() {
                println!("{} already in store ({store_location})", self.name);
                return Ok(store_location);
            } else {
                println!("{store_location} not present")
            }
        }
        // } else {
        let staging_path = format!(
            "{jade_root}/staging/{}",
            self.file_name // self.url.rsplit_once('/').unwrap().1
        );
        self.download(client, &self.url, &staging_path).await?;
        let (mut artifact, hash) = self.post_process(&staging_path, lock_file.is_some())?;

        if let Some(lockf) = lock_file {
            lockf.add_lock(&self.name, &hash)?;
        }

        self.hash = Some(hash);
        if let Some(target) = &self.target {
            artifact = format!("{artifact}/{target}")
        }

        let store_location = self.get_store_location(jade_root).unwrap();
        // install to store
        if install_to_store {
            fs::create_dir_all(&store_location);
            match self.package_format {
                PackageFmt::Zip => fs::rename(artifact, &format!("{store_location}/artifact/")),
                PackageFmt::File => {
                    fs::create_dir(format!("{store_location}/artifact/"));
                    fs::rename(
                        artifact,
                        &format!("{store_location}/artifact/{}", self.file_name),
                    )
                }
            }
            .map_err(|e| JadeError::IOError {
                package: self.name.clone(),
                io_error: format!("failed to install to store: {e}"),
            })?;
            return Ok(store_location);
        } else {
            return Ok(artifact);
        }
    }

    // https://georgik.rocks/how-to-download-binary-file-in-rust-by-reqwest/
    /// downloads artifact from derivation
    async fn download(
        &self,
        client: &reqwest::Client,
        url: &str,
        file_name: &str,
    ) -> Result<(), JadeError> {
        println!("downloading {url} to {file_name}...");
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| JadeError::Error(e.to_string()))?;
        let mut file = std::fs::File::create(file_name).map_err(|e| JadeError::IOError {
            package: self.name.clone(),
            io_error: format!("error creating file {file_name}: {e}"),
        })?;
        let mut content = Cursor::new(response.bytes().await.map_err(|e| JadeError::IOError {
            package: self.name.clone(),
            io_error: e.to_string(),
        })?);
        println!("writing {file_name} to disk...");
        std::io::copy(&mut content, &mut file).map_err(|e| JadeError::IOError {
            package: self.name.clone(),
            io_error: e.to_string(),
        })?;
        Ok(())
    }
    // processes artifact after download returns new artifact path if it was a zip
    fn post_process(&self, artifact_path: &str, lock: bool) -> Result<(String, String), JadeError> {
        let hash = if lock {
            if let Some(expected) = &self.hash {
                self.hash_format.check(artifact_path, expected)?
            } else {
                // return Err(JadeError::Error(format!(
                //     "Artifact {} hash missing",
                //     self.name
                // )));
                println!(
                    "warning {} not present in lock file. use --unlock to allow ALLOWING ANYWAYS TESTING GENERATING HASH...",
                    self.name
                );
                self.hash_format.hash(artifact_path)?
            }
        } else {
            self.hash_format.hash(artifact_path)?
        };

        match self.package_format {
            PackageFmt::File => Ok((artifact_path.to_string(), hash)),
            PackageFmt::Zip => {
                let src_archive = Path::new(&artifact_path).to_path_buf();
                if !zip_extensions::is_zip(&src_archive) {
                    return Err(JadeError::IOError {
                        package: self.name.clone(),
                        io_error: format!("{artifact_path} is not a zip archive file"),
                    });
                }
                let dest_str = format!("{artifact_path}.extracted");
                let dest = Path::new(&dest_str).to_path_buf();
                println!("Unzipping {artifact_path}");
                zip_extensions::zip_extract(&src_archive, &dest).map_err(|e| {
                    JadeError::IOError {
                        package: self.name.clone(),
                        io_error: e.to_string(),
                    }
                })?;
                fs::remove_file(artifact_path).map_err(|e| JadeError::IOError {
                    package: self.name.clone(),
                    io_error: format!("failed to delete temporary artifact {artifact_path}: {e}"),
                })?;
                Ok((dest_str, hash))
            }
        }
    }
    fn get_store_location(&self, jade_root: &str) -> Option<String> {
        if let Some(hash) = &self.hash {
            let base = format!("{jade_root}/store/{}-{}", hash, self.name);
            Some(base)
            // match self.package_format{
            //     PackageFmt::File => {
            //         let ext = self
            //     },
            //     PackageFmt::Zip => todo!(),
            // }
        } else {
            None
        }
    }
}
// returns list
pub async fn compose(
    jade_root: &str,
    packages: Vec<Package>,
    lock_file: &mut Option<JadeLock>,
    install_to_store: bool,
) -> Result<Vec<String>, JadeError> {
    fs::create_dir_all(format!("{jade_root}/staging"));
    fs::create_dir_all(format!("{jade_root}/store"));

    let mut composed_artifacts: Vec<String> = Vec::new();
    let client = reqwest::Client::new();
    for mut pkg in packages {
        composed_artifacts.push(
            pkg.fetch_package(&client, jade_root, install_to_store, lock_file)
                .await?,
        );
    }

    Ok(composed_artifacts)
}
