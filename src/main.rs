/*
 * Components
 * PackageFormula       -- Description of the layout of a package, described in [[Formula]]
 * SourceDriver         -- Driver for a given package source
 * PackageDerivation    -- Derivation of a single package
 * PackageTree          -- Top level descriptor for jade
 */

/*
 * PackageFormula format
 * name="format name"
 * fileformat=<"zip","file"> # default file
 * zip_target="/path/to/target/dir/in/zip" # default none, required for zip format
 */

/*
 * PackageDerivation format
 * formula="format name" # defaults to toplevel cfg
 * url="https://driver.domain/package/artifact" # required
 * depends=["list","of","depends"]
 * tag=["list","of","tags"]
 */

/*
 * PackageTree format
 * [[Formula]]
 * ...
 * [Cfg]
 * default_formula="format name"
 * tree_name="name"
 * version="1.0"
 * [[Package]]
 *
 *
*/
mod driver;
mod drivers;
mod lock;
mod manifest;
mod package;
mod source_driver;
mod store;
mod utils;
fn main() {
    println!("Hello, world!");
}
