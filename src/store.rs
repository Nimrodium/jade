use std::{
    fmt::Display,
    fs,
    path::Path,
    sync::{Arc, mpsc},
    thread::{self, JoinHandle},
};

use crate::package::Derivation;
#[derive(Clone)]
pub struct Store {
    store_path: String,
    temp: String,
}
impl Store {
    pub fn new(store_path: &str, temp: &str) -> Self {
        Self {
            store_path: store_path.to_string(),
            temp: temp.to_string(),
        }
    }

    fn make_package_store_path(&self, package_signature: &str) -> StorePath {
        StorePath::new(&format!("{}/{package_signature}", self.store_path))
    }
    /// returns address if present, else None
    fn is_package_in_store(&self, package: &Derivation) -> Option<StorePath> {
        if package.hash.is_some() {
            let package_store_path =
                self.make_package_store_path(&package.generate_hash_signature());
            if package_store_path.exists() {
                Some(package_store_path)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn realize_derivation(
        &self,
        derivation: Derivation,
    ) -> Result<(StorePath, Derivation), String> {
        let mut derivation = derivation.clone();
        if let Some(path) = self.is_package_in_store(&derivation) {
            Ok((path, derivation))
        } else {
            let cache_file = {
                let path = derivation.download(&self.temp)?;
                if derivation.extract {
                    derivation.extract_package(&path)?
                } else {
                    path
                }
            };
            Ok((
                derivation.install_to_store(&self.store_path, &cache_file)?,
                derivation,
            ))
        }
    }
    /// fetches derivation store paths, executing derivation if not present
    pub fn realize_derivations(
        &self,
        derivations: Vec<Derivation>,
    ) -> Result<(Vec<StorePath>, Vec<Derivation>), String> {
        let (sender, receiver) = mpsc::channel();
        // let mut handles: Vec<JoinHandle<()>> = Vec::new();
        for derivation in derivations {
            let cloned_self = self.clone();
            let cloned_sender = sender.clone();
            thread::spawn(move || {
                let result = cloned_self.realize_derivation(derivation);
                cloned_sender.send(result).unwrap();
            });
        }
        drop(sender);
        let mut realized = Vec::<StorePath>::new();
        let mut new_derivations = Vec::<Derivation>::new();
        for recieved in receiver {
            let (store_path, new_derivation) = recieved?;
            realized.push(store_path);
            new_derivations.push(new_derivation);
        }
        Ok((realized, new_derivations))
    }
    /// backup for if the threaded one is being stupid, not actually intended to be used
    pub fn realize_derivation_sequential(
        &self,
        derivations: Vec<Derivation>,
    ) -> Result<(Vec<StorePath>, Vec<Derivation>), String> {
        let mut realized = Vec::<StorePath>::new();
        let mut new_derivations = Vec::<Derivation>::new();
        for derivation in derivations {
            let (store_path, new_derivation) = self.realize_derivation(derivation)?;
            realized.push(store_path);
            new_derivations.push(new_derivation);
        }
        Ok((realized, new_derivations))
    }
}

pub struct StorePath {
    path: String,
    // invoked_from: Derivation,
}

impl Display for StorePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl StorePath {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
    fn exists(&self) -> bool {
        Path::new(&self.path).exists()
    }
    pub fn get_artifact(&self) -> String {
        format!("{}/artifact", self.path)
    }
    pub fn copy_to(&self, dest: &str) -> Result<(), String> {
        let artifact = self.get_artifact();
        let path = Path::new(&artifact);
        if path.is_dir() {
            copy_dir::copy_dir(&artifact, dest).map_err(|e| {
                format!("failed to copy artifact (`{artifact}`) to dest (`{dest}`): {e}")
            })?;
        } else {
            fs::copy(&artifact, dest).map_err(|e| {
                format!("failed to copy artifact (`{artifact}`) to dest (`{dest}`): {e}")
            })?;
        }
        Ok(())
    }
    #[cfg(target_os = "windows")]
    pub fn symlink_to(&self, dest: &str) -> Result<(), String> {
        let artifact = self.get_artifact();
        let path = Path::new(&artifact);
        if path.is_dir() {
            std::os::windows::fs::symlink_dir(artifact, dest).map_err(|e|format!("failed to symlink dir `{artifact}` to `{dest}`: {e} (try passing the --copy flag to copy instead of symlink.)"))?;
        } else {
            std::os::windows::fs::symlink_file(artifact, dest).map_err(|e|format!("failed to symlink dir `{artifact}` to `{dest}`: {e} (try passing the --copy flag to copy instead of symlink.)"))?;
        }
        Ok(())
    }

    #[cfg(unix)]
    pub fn symlink_to(&self, dest: &str) -> Result<(), String> {
        let artifact = self.get_artifact();
        std::os::unix::fs::symlink(&artifact, dest).map_err(|e|format!("failed to symlink dir `{artifact}` to `{dest}`: {e} (try passing the --copy flag to copy instead of symlink.)"))?;
        Ok(())
    }

    fn install_to(&self, dest: &str, symlink: bool) -> Result<(), String> {
        if symlink {
            self.symlink_to(dest)
        } else {
            self.copy_to(dest)
        }
    }
}
