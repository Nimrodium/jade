use std::{
    fmt::{Debug, Display},
    fs,
    path::Path,
    sync::{Arc, mpsc},
    thread::{self, JoinHandle},
};

use crate::package::{Derivation, Derivations};
#[derive(Clone)]
pub struct Store {
    pub store_path: String,
    pub temp: String,
}
impl Store {
    pub fn new(store_path: &str, temp: &str) -> Self {
        Self {
            store_path: store_path.to_string(),
            temp: temp.to_string(),
        }
    }

    pub fn make_package_store_path(&self, derivation: &Derivation) -> StorePath {
        StorePath::new(
            &format!(
                "{}/{}",
                self.store_path,
                derivation.generate_hash_signature(),
            ),
            &derivation.file_name,
            &derivation.hash.clone().expect(&format!(
                "cannot build store path for {} without hash",
                derivation.name
            )),
        )
        // StorePath::new(&format!("{}/{}", self.store_path))
    }
    /// returns address if present, else None
    pub fn is_package_in_store(&self, package: &Derivation) -> Option<StorePath> {
        if package.hash.is_some() {
            let package_store_path = self.make_package_store_path(&package);
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
                let path = derivation.download(&self.temp, None, None)?;
                if derivation.extract {
                    derivation.extract_package(&path)?
                } else {
                    path
                }
            };
            Ok((derivation.install_to_store(&self, &cache_file)?, derivation))
        }
    }
    /// fetches derivation store paths, executing derivation if not present
    pub fn realize_derivations(
        &self,
        derivations: Vec<Derivation>,
    ) -> Result<(Vec<StorePath>, Vec<Derivation>), String> {
        let (sender, receiver) = mpsc::channel();
        let mut realized = Vec::<StorePath>::new();
        let mut new_derivations = Vec::<Derivation>::new();
        // let mut handles: Vec<JoinHandle<()>> = Vec::new();
        for derivation in derivations {
            if let Some(store_path) = self.is_package_in_store(&derivation) {
                println!("package already in store {store_path}");
                realized.push(store_path);
                new_derivations.push(derivation);
                continue;
            }
            let cloned_self = self.clone();
            let cloned_sender = sender.clone();
            thread::spawn(move || {
                let result = cloned_self.realize_derivation(derivation);
                cloned_sender.send(result).unwrap();
            });
        }
        drop(sender);

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
        derivations: Derivations,
    ) -> Result<(Vec<StorePath>, Vec<Derivation>), String> {
        let mut realized = Vec::<StorePath>::new();
        let mut new_derivations = Vec::<Derivation>::new();
        for derivation in derivations.derivations {
            let (store_path, new_derivation) = self.realize_derivation(derivation)?;
            realized.push(store_path);
            new_derivations.push(new_derivation);
        }
        Ok((realized, new_derivations))
    }
}

pub struct StorePath {
    path: String,
    name: String,
    hash: String,
    // invoked_from: Derivation,
}

impl Display for StorePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}
impl Debug for StorePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl StorePath {
    pub fn new(path: &str, name: &str, hash: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            hash: hash.to_string(),
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
        println!("copying {artifact} -> {dest}");
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
        println!("symlinking {artifact} -> {dest}");
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
        println!("symlinking {artifact} -> {dest}");
        std::os::unix::fs::symlink(&artifact, dest).map_err(|e|format!("failed to symlink dir `{artifact}` to `{dest}`: {e} (try passing the --copy flag to copy instead of symlink.)"))?;
        Ok(())
    }

    pub fn install_to(&self, dest_dir: &str, symlink: bool) -> Result<(), String> {
        fs::create_dir_all(dest_dir)
            .map_err(|e| format!("failed to create destination `{dest_dir}`: {e}"))?;

        let dest = format!("{dest_dir}/{}", self.name);
        remove_fs_entity(&dest);
        if symlink {
            self.symlink_to(&dest)
        } else {
            self.copy_to(&dest)
        }
    }
}

fn remove_fs_entity(p: &str) -> Result<(), String> {
    let path = Path::new(p);
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(|e| format!("failed to remove dir/file `{p}`: {e}"))?;
    Ok(())
}
