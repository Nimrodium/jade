use serde::{Deserialize, Serialize};
use toml;
/*
 * PackageDerivation format
 * formula="format name" # defaults to toplevel cfg
 * url="https://driver.domain/package/artifact" # required
 * depends=["list","of","depends"]
 * tag=["list","of","tags"]
 */
#[derive(Serialize, Deserialize)]
pub struct Package {
    driver: String,
    pkg: String,
    ver: Option<String>,
    // pkg acts as name.
    // url:Option<String>,

    // formula overrides
    input: Option<String>,
    zip_target: Option<String>,
    zip_target_dirall: Option<bool>,
    target: Option<String>,
}
impl Package {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
    pub fn has_tag(&self) -> bool {
        todo!()
    }
    /// adds a tag, returns Err if tag already present (tho it isnt really an error and might make it into an Option)
    pub fn add_tag(&self, tag: &str) -> Result<(), String> {
        todo!()
    }
    pub fn rm_tag(&self, tag: &str) -> Result<(), String> {
        todo!()
    }
}
