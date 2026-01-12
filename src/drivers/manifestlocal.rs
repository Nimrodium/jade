// awful name will change when i think of something better
// grabs the files from the manifest directory

use std::path::{Path, PathBuf};

use crate::driver::Driver;

pub struct ManifestLocalDriver {
    manifest_dir: PathBuf,
}
impl ManifestLocalDriver {
    fn new(manifest_path: &Path) -> Self {
        Self {
            manifest_dir: manifest_path.to_owned(),
        }
    }
}

impl Driver for ManifestLocalDriver {
    fn search(
        &self,
        slug: &str,
        results: usize,
    ) -> Result<Vec<crate::driver::DriverResult>, String> {
        // i guess it would just list the available contents ?
        todo!()
    }

    fn download(
        &self,
        result: &crate::driver::DriverResult,
        version: &str,
    ) -> Result<PathBuf, String> {
        // copy the artifact to TMP
        todo!()
    }

    fn build_package(
        &self,
        result: &crate::driver::DriverResult,
        version: crate::driver::Version,
    ) -> Result<crate::package::Package, String> {
        // generate a [[Package]] entry for when installing imperitively ?
        todo!()
    }

    fn derive_package(&self, pkg: &crate::package::Package) -> Result<PathBuf, String> {
        todo!()
    }
}
