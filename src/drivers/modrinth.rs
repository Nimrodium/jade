use crate::driver::Driver;
use modrinth_api;
use sha2::digest::typenum::Mod;
const DOMAIN: &str = "api.modrinth.com";
pub struct ModrinthDriver {
    mcloader: String,
    mcver: String,
}
impl ModrinthDriver {
    fn new(config: &toml::Table) -> Result<Self, String> {
        let cfg = config
            .get("modrinth")
            .ok_or(format!("Modrinth Driver Configuration not defined."))?
            .as_table()
            .ok_or(format!("Modrinth Driver Configuration Not TOML Table."))?;
        Ok(Self {
            mcloader: cfg
                .get("loader")
                .ok_or(format!("missing config parameter `loader`"))?
                .as_str()
                .ok_or("config parameter `loader` present but not string")?
                .to_string(),
            mcver: cfg
                .get("loader")
                .ok_or(format!("missing config parameter `loader`"))?
                .as_str()
                .ok_or("config parameter `loader` present but not string")?
                .to_string(),
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
}

struct SearchResults {}
