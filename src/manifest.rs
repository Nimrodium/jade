use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/*
 *
 * Redistributable Manifest and Lockfile inputs for jade
 *
 */
use serde_derive::{Deserialize, Serialize};
use toml::{self, Table};
#[derive(Deserialize, Serialize)]
pub struct Manifest {
    #[serde(skip_serializing, skip_deserializing)]
    _manifest_path: Option<PathBuf>,
    default_tags: Vec<String>,
    #[serde(flatten)]
    pub driver: HashMap<String, Table>,
}
impl Manifest {
    pub fn give_path(&mut self, path: &Path) {
        // if Some(p) = self._manifest_path{
        //     panic!("attempted to set manifest path after already setting")
        // }else{

        // }
        if self._manifest_path.is_none() {
            self._manifest_path = Some(path.to_owned())
        } else {
            panic!("attempted to set manifest path after already setting")
        }
    }
    pub fn get_path(&self) -> &Path {
        if let Some(p) = &self._manifest_path {
            p
        } else {
            unreachable!("manifest path not set, this should be impossible.")
        }
    }
    pub fn get_driver_config(&self, driver_id: &str) -> Option<&Table> {
        self.driver.get(driver_id)
    }
}

pub struct LockFile {}
