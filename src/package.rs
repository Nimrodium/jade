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
    formula: Option<String>,
    url: String,
    depends: Option<Vec<String>>,
    tag: Option<Vec<String>>,
}
impl Package {}

pub struct PackageFormula {}
impl PackageFormula {}

pub struct PackageTree {}
impl PackageTree {}
