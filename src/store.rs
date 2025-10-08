use crate::manifest::LockFile;
use crate::manifest::Manifest;
use crate::utils::remove_fs_entity;
use copy_dir;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::package::{Package, PackageTree};

pub struct StorePath {
    inner: PathBuf,
    hash: String,
    pkg_name: String,
}
impl fmt::Display for StorePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
impl StorePath {
    fn build_path_from_parts(store: &str, hash: &str, pkg_name: &str) -> String {
        format!("{store}/{hash}-{pkg_name}/")
    }
    fn new(store: &str, pkg_name: &str, hash: &str) -> Self {
        Self {
            inner: PathBuf::from(Self::build_path_from_parts(store, hash, pkg_name)),
            hash: hash.to_string(),
            pkg_name: pkg_name.to_string(),
        }
    }
    /// checks if the path exists on disk
    pub fn exists(&self) -> bool {
        self.inner.exists()
    }
    /// gets OSPath
    pub fn get_path(&self) -> &Path {
        &self.inner
    }
    /// deletes from disk
    pub fn delete(&self) -> Result<(), String> {
        if self.exists() {
            fs::remove_dir_all(&self.inner).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn get_artifact(&self) -> String {
        format!("{}/artifact", self)
    }
    /// copies artifact
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
            std::os::windows::fs::symlink_dir(&artifact, dest).map_err(|e|format!("failed to symlink dir `{artifact}` to `{dest}`: {e} (try passing the --copy flag to copy instead of symlink.)"))?;
        } else {
            std::os::windows::fs::symlink_file(&artifact, dest).map_err(|e|format!("failed to symlink dir `{artifact}` to `{dest}`: {e} (try passing the --copy flag to copy instead of symlink.)"))?;
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

        let dest = format!("{dest_dir}/{}", self.pkg_name);
        remove_fs_entity(&dest);
        if symlink {
            self.symlink_to(&dest)
        } else {
            self.copy_to(&dest)
        }
    }
}
/// file written to root of target describing which files are managed by jade
pub struct TargetManifest {
    files: Vec<String>,
}

pub struct Store {
    root: String,
    store: String,
    lock_file: String,
}
impl Store {
    /// evaluate a package and return a store path
    pub fn evaluate(package: &Package) -> StorePath {
        todo!()
    }
    fn download(pkg: &Package) -> StorePath {
        todo!()
    }
}

struct GarbageCollector {
    var: PathBuf,
}
impl GarbageCollector {
    /// cleans dereferenced paths
    fn clean_deref(&self) -> Result<(), String> {
        todo!()
    }
    fn clean_generations(&self, keep_last_n: usize) -> Result<(), String> {
        todo!()
    }
    fn delete_manifest(&self, manifest: String) -> Result<(), String> {
        todo!()
    }
}

struct VarManifest {
    manifest: Manifest,
    lockfile: LockFile,
    derived: Vec<StorePath>,
}

// ///
// pub struct LiveTree {
//     // trees: Vec<String>,
// }
// impl LiveTree {
//     fn get_trees() -> Vec<PackageTree> {
//         todo!()
//     }
//     fn get_live_paths(&self) -> Vec<StorePath> {
//         todo!()
//     }
//     fn get_dead_paths(&self) -> Vec<StorePath> {
//         todo!()
//     }
//     fn collect_garbage(&self) -> Result<(), String> {
//         todo!()
//     }
// }
