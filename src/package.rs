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
}
