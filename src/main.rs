// parse packwiz files and collect all their urls and then download
use std::{fs::File, io::read_to_string, path::Path};

use package::load_derivations_from_directory;
use preprocessor::dedup;
use store::Store;
// mod _composer;
// mod _package;
mod package;
mod packwiz_compat;
mod preprocessor;
mod store;
// use gpackage::{JadeLock, JadePack};

fn main() {
    let store = Store::new("/home/kyle/.jade/store", "/home/kyle/.jade/staging");

    let derivations = dedup(
        load_derivations_from_directory(Path::new("/home/kyle/.jade/vanilla+/derives/")).unwrap(),
    );

    let paths = store.realize_derivations(derivations).unwrap();
    for path in paths.0 {
        println!("{path}");
    }
}
