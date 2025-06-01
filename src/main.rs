// parse packwiz files and collect all their urls and then download
use std::{
    env,
    fs::{self, File},
    io::read_to_string,
    path::Path,
    process,
};
mod util;
use manifest::Manifest;
use package::{Derivations, load_derivations_from_directory};
use preprocessor::dedup;
use store::Store;
// mod _composer;
// mod _package;
mod boostrap;
mod manifest;
mod package;
mod packwiz_compat;
mod preprocessor;
mod store;
use clap::Parser;
use util::normalize;
const MANIFEST: &str = "manifest.jade.toml";
const DERIVES_FALLBACK: &str = "./derives/";

// #[cfg(target_os = "windows")]
// const do_symlink: bool = false;
// #[cfg(not(target_os = "windows"))]
// const do_symlink: bool = true;

// use gpackage::{JadeLock, JadePack};
#[derive(Parser, Debug)]
#[command(
    name = "jade",
    about = "Declarive Mod Manager and Deployment Engine inspired by Nix"
)]

struct Args {
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    manifest: Option<String>,
    #[arg(long)]
    store: Option<String>,
    #[arg(long)]
    root: Option<String>,
    #[arg(long)]
    derives: Option<String>,
    // #[arg(long, default_value_t = do_symlink())]
    #[arg(long)]
    symlink: bool,
    #[arg(long)]
    copy: bool,
    #[arg(long)]
    complete: bool,
    #[command(subcommand)]
    command: Commands,
}
#[derive(clap::Subcommand, Debug)]
enum Commands {
    BootStrap {
        #[arg(short, long)]
        manifest: Option<String>,
    },
    Compose {
        // #[arg(short, long)]
        // source: Option<String>,
        #[arg(short, long)]
        target: Option<String>,
    },
    Edit {
        modname: String,
        #[arg(long)]
        editor: Option<String>,
    },
    Check {},
    Install {},
    List {
        filter: Option<String>,
    },
}

// #[cfg(target_os = "windows")]
fn get_jade_root() -> Result<String, String> {
    let root = if let Some(root) = env::var("JADEROOT").ok() {
        root
    } else {
        #[cfg(target_os = "windows")]
        {
            let appdata = env::var("APPDATA")
                .map_err(|e| format!("critical failure %APPDATA% could not be found: {e}"))?;
            let root = format!("{appdata}/Local/jade/");
            root
        }
        #[cfg(not(target_os = "windows"))]
        {
            let home = env::var("HOME")
                .map_err(|e| format!("critical failure $HOME could not be found: {e}"))?;
            let root = format!("{home}/.jade/");

            root
        }
    };
    fs::create_dir_all(&root);
    Ok(root)
}
// fn get_temp()
fn entry(args: Args) -> Result<(), String> {
    let root = if let Some(root) = args.root {
        root
    } else {
        get_jade_root()?
    };
    let store_path = if let Some(store) = args.store {
        store
    } else {
        format!("{root}/store/")
    };

    let store = Store::new(&store_path, &format!("root/staging"));
    let manifest = if let Some(manifest) = args.manifest {
        Manifest::load(&manifest)?
    } else {
        let manifest_path = format!("./{MANIFEST}");
        if let Some(flag_manifest) = args.manifest {
            Manifest::load(&flag_manifest)?
        } else if Path::new(&manifest_path).exists() {
            Manifest::load(&manifest_path)?
        } else {
            return Err(format!(
                "could not locate manifest {manifest_path}: try passing the --manifest flag to manually specify pack manifest"
            ))?;
        }
    };

    let derives = if let Some(derives) = args.derives {
        derives
    } else if let Some(derives) = manifest.derives {
        derives
    } else if {
        let derives_path = Path::new(DERIVES_FALLBACK);
        derives_path.exists() && derives_path.is_dir()
    } {
        DERIVES_FALLBACK.to_string()
    } else {
        return Err(format!(
            "could not find derives, either create the ./derives directory in your pack, set this in the project manfiest (derives = \"/path/to/derives\" or try passing the --derives flag to manually specify directory"
        ));
    };

    let symlink = if args.symlink {
        true
    } else if args.copy {
        false
    } else {
        #[cfg(target_os = "windows")]
        {
            false
        }
        #[cfg(not(target_os = "windows"))]
        {
            true
        }
    };
    // {
    //     #[cfg(target_os = "windows")]
    //     {
    //         false
    //     }
    //     #[cfg(not(target_os = "windows"))]
    //     {
    //         true
    //     }
    // }

    match args.command {
        Commands::BootStrap { manifest } => todo!(),
        Commands::Compose { target } => {
            let target = if let Some(target) = target {
                target
            } else if let Some(target) = manifest.target {
                target
            } else {
                return Err(format!(
                    "no target specified, either add this to the pack manifest (target = \"/path/to/target\") or manually specify with the --target flag"
                ));
            };
            let derivations = load_derivations_from_directory(&Path::new(&derives))?;
            let (paths, derivations) = store.realize_derivations(derivations)?;
            for path in paths {
                path.install_to(&target, symlink)?;
            }
            // let source = if let Some(source) = source {
            //     target
            // } else if let Some(source) = manifest.source {
            //     source
            // } else {
            //     return Err(format!(
            //         "no source specified, either add this to the pack manifest (target = \"/path/to/\") or manually specify with the --target flag"
            //     ));
            // };
        }
        Commands::Edit { modname, editor } => {
            let editor_print = if cfg!(windows) {
                "%EDITOR%".to_string()
            } else {
                "$EDITOR".to_string()
            };
            let editor = if let Some(editor) = editor {
                editor
            } else {
                env::var("EDITOR")
                    .map_err(|e| format!("error could not access {editor_print}: {e}, either set this environment variable or try passing the --editor flag to manually specify editora"))?
            };
            let normalized_modname = util::normalize(&modname);
            let path = {
                // find derivation based on name field
                // let derivations = load_derivations_from_directory(&Path::new(&derives))?;
                let derivations = Derivations::load_derivations_from_directory(&derives)?;
                derivations
                    .get_derivation_by_fuzzy_name(&normalized_modname)?
                    .backing_file
                    .clone()
            };
            process::Command::new(editor).arg(&path).output();
        }
        Commands::Check {} => todo!(),
        Commands::Install {} => todo!(),
        Commands::List { filter } => {
            let derivations = Derivations::load_derivations_from_directory(&derives)?;
            let name = if let Some(name) = filter {
                Some(normalize(&name))
            } else {
                None
            };
            for derivation in derivations.derivations {
                if let Some(name) = name.as_ref() {
                    if derivation.name.contains(name) {
                        println!("{}\t({})", derivation.name, derivation.backing_file);
                    }
                } else {
                    println!("{}\t({})", derivation.name, derivation.backing_file);
                }
            }
        }
    }
    Ok(())
}
fn main() {
    let args = Args::parse();
    println!("{args:?}");
    // panic!();
    match entry(args) {
        Ok(()) => (),
        Err(e) => println!("Error: {e}"),
    };
    // panic!();
    // let store = Store::new("/home/kyle/.jade/store", "/home/kyle/.jade/staging");

    // let derivations = dedup(
    //     load_derivations_from_directory(Path::new("/home/kyle/.jade/vanilla+/derives/")).unwrap(),
    // );

    // let (paths, derivations) = store.realize_derivations(derivations).unwrap();
    // for path in paths {
    //     println!("{path}");
    //     path.install_to("/home/kyle/.jade/vanilla+-deployed/mods/", true)
    //         .unwrap();
    // }
    // util::update_derives(
    //     &derivations,
    //     "/home/kyle/.jade/pack_backups/",
    //     "/home/kyle/.jade/vanilla+/",
    //     "vanilla+",
    // )
    // .unwrap();
}
