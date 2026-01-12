use crate::driver::Driver;
use modrinth_api;
use serde::{Deserialize, Serialize};
use toml::Table;
const DOMAIN: &str = "api.modrinth.com";

#[derive(Serialize, Deserialize)]
struct Config {
    mcloader: String,
    mcver: String,
}

pub struct ModrinthDriver {
    mcloader: String,
    mcver: String,
}
impl ModrinthDriver {
    pub fn new(config: &Table) -> Result<Self, String> {
        let cfg: Config = config
            .clone()
            .try_into()
            .map_err(|e| format!("failed to deserialize modrinth config: {e}"))?;
        Ok(Self {
            mcloader: cfg.mcloader,
            mcver: cfg.mcver,
        })
    }
    // fn resolve_url()
}
impl Driver for ModrinthDriver {
    fn search(
        &self,
        slug: &str,
        results: usize,
    ) -> Result<Vec<crate::driver::DriverResult>, String> {
        todo!()
    }

    fn download(
        &self,
        result: &crate::driver::DriverResult,
        version: &str,
    ) -> Result<std::path::PathBuf, String> {
        todo!()
    }

    fn build_package(
        &self,
        result: &crate::driver::DriverResult,
        version: crate::driver::Version,
    ) -> Result<crate::package::Package, String> {
        todo!()
    }

    fn derive_package(&self, pkg: &crate::package::Package) -> Result<std::path::PathBuf, String> {
        todo!()
    }
}

struct SearchResults {}
