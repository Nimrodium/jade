// parse packwiz files and collect all their urls and then download
const VERSION: &str = "1.0";
use std::{
    collections::HashSet,
    env,
    fs::{self, File, create_dir, create_dir_all},
    io::{Write, read_to_string},
    mem,
    path::Path,
    process::{self, exit},
};
mod api;
mod api_driver;
mod util;
mod verbose;
use colorize::AnsiColor;
use manifest::Manifest;
use package::{Derivations, load_derivations_from_directory};
// use preprocessor::dedup;
use store::Store;
// mod _composer;
// mod _package;
// mod _boostrap;
mod manifest;
mod package;
// mod _packwiz_compat;
// mod _preprocessor;
mod store;
use clap::Parser;
use util::normalize;

use crate::util::{confirm, select_index};
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
    #[arg(long)]
    cwd: Option<String>,
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
    /// test
    Init {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        derives: Option<String>,
        #[arg(short, long)]
        api: Option<String>,
        #[arg(short, long)]
        target: Option<String>,
        directory: Option<String>,
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
    Search {
        query: String,
    },
    Install {
        mods: Vec<String>,
    },
    List {
        filter: Option<String>,
    },
    Version,
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

fn load_context(dir: &str, args: &Args) -> Result<(Manifest, String), String> {
    let manifest = if let Some(manifest) = &args.manifest {
        Manifest::load(&manifest)?
    } else {
        let manifest_path = format!("{dir}/{MANIFEST}");
        if let Some(flag_manifest) = &args.manifest {
            Manifest::load(&flag_manifest)?
        } else if Path::new(&manifest_path).exists() {
            Manifest::load(&manifest_path)?
        } else {
            return Err(format!(
                "could not locate manifest {manifest_path}: try passing the --manifest flag to manually specify pack manifest"
            ))?;
        }
    };

    let fallback = format!("{dir}/derives/");
    let derives = if let Some(derives) = &args.derives {
        derives.clone()
    } else if let Some(derives) = &manifest.main.derives {
        derives.clone()
    } else if {
        let derives_path = Path::new(&fallback);
        derives_path.exists() && derives_path.is_dir()
    } {
        fallback
    } else {
        return Err(format!(
            "could not find derives, either create the ./derives directory in your pack, set this in the project manfiest (derives = \"/path/to/derives\" or try passing the --derives flag to manually specify directory"
        ));
    };

    Ok((manifest, derives))
}
// fn get_temp()
fn entry(args: Args) -> Result<(), String> {
    if let Some(cwd) = &args.cwd {
        env::set_current_dir(cwd)
            .map_err(|e| format!("failed to set current working directory: {e}"))?;
    }
    let root = if let Some(root) = &args.root {
        root.to_string()
    } else {
        get_jade_root()?
    };
    let store_path = if let Some(store) = &args.store {
        store.to_string()
    } else {
        format!("{root}/store/")
    };

    let store = Store::new(&store_path, &format!("{root}/staging"));

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

    match args.command {
        Commands::Init {
            directory,
            name,
            derives,
            api,
            target,
        } => {
            let directory = if let Some(dir) = directory {
                dir
            } else {
                "./".to_string()
            };
            let manifest = Manifest::init(&name, derives, api, target);
            create_dir_all(format!("{directory}/derives"));
            let manifest_serial = toml::to_string(&manifest)
                .map_err(|e| format!("failed to serialize manifest file {e}"))?;
            let mut manifest_f = File::create_new(format!("{directory}/{MANIFEST}"))
                .map_err(|e| format!("failed to create manifest file {e}"))?;
            manifest_f
                .write_all(manifest_serial.as_bytes())
                .map_err(|e| format!("failed to write to manifest file {e}"))?;
        }
        Commands::BootStrap { manifest } => todo!(),
        Commands::Compose { ref target } => {
            let (manifest, derives) = load_context("./", &args)?;
            let target = if let Some(target) = target {
                target.to_string()
            } else if let Some(target) = manifest.main.target {
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
        }
        Commands::Edit {
            ref modname,
            ref editor,
        } => {
            let (manifest, derives) = load_context("./", &args)?;
            let default_editor = if cfg!(windows) {
                "notepad".to_string()
            } else {
                "nano".to_string()
            };
            let editor = if let Some(editor) = editor {
                editor.to_string()
            } else {
                env::var("EDITOR").unwrap_or(default_editor)
                // .map_err(|e| format!("error could not access {editor_print}: {e}, either set this environment variable or try passing the --editor flag to manually specify editora"))?
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
        Commands::Search { ref query } => {
            let (manifest, derives) = load_context("./", &args)?;
            let api_name = if let Some(name) = manifest.main.api {
                name
            } else {
                return Err(format!("no api driver specified"));
            };
            let driver = api::get_api_driver(&api_name, &manifest.api_cfg)?;
            let results = driver.search(query)?;
            println!("{} result(s) for {query}", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{} {result}\n--", format!("{i})").red());
            }
        }
        Commands::Install { ref mods } => {
            let (manifest, derives) = load_context("./", &args)?;
            let api_name = if let Some(name) = manifest.main.api {
                name
            } else {
                return Err(format!("no api driver specified"));
            };
            let driver = api::get_api_driver(&api_name, &manifest.api_cfg)?;

            let mut derivations = Derivations::load_derivations_from_directory(&derives)?;

            let mut mod_set = HashSet::new();
            for slug in mods {
                let mut install = true;
                for derive in &derivations.derivations {
                    if derive.name.contains(&normalize(slug)) {
                        if !confirm(
                            &format!(
                                "{slug} potentionally already installed in tree ({}) reinstall?",
                                derive.backing_file
                            ),
                            false,
                        )? {
                            install = false;
                        }
                    }
                }
                if install {
                    mod_set.insert(slug);
                }
            }

            let mut pkg_ids = Vec::new();
            for (i, slug) in mod_set.iter().enumerate() {
                let results = driver.search(slug)?;
                if results.is_empty() {
                    return Err(format!("no results for {slug}"));
                }
                if results.len() == 1 {
                    println!("{}", results[0]);
                    let resp = confirm(&format!("install {}?", results[0].slug), true)?;
                    if resp {
                        pkg_ids.push(results[0].id.clone());
                    }
                } else {
                    for (i, result) in results.iter().enumerate() {
                        println!("{} {result}\n--", format!("{i})").red());
                    }
                    let n = select_index(
                        &format!("({}/{}) select mod result to install", i + 1, mod_set.len(),),
                        0,
                        0,
                        results.len() as isize - 1,
                    )?;
                    pkg_ids.push(results[n as usize].id.clone());
                }
            }
            let mut new_derivations = Vec::new();
            for id in pkg_ids {
                let derives = driver.get_derivations_for(
                    &id,
                    &mut derivations.get_api_pkg_id_list(),
                    true,
                    &store,
                )?;
                new_derivations.extend(derives);
            }
            let mut install_derives = Vec::new();
            for mut derive in new_derivations {
                if let Some((found, installed)) = derivations.find_unmanaged_matches(&derive) {
                    let prompt = if installed {
                        format!(
                            "\nderivation for `{}` already installed ({}) and managed by {} driver\noverride?",
                            derive.name, found.backing_file, api_name
                        )
                    } else {
                        format!(
                            "\nderivation for `{}` found in tree ({}) but unmanaged by {} driver,\nupdate derivation with api metadata?",
                            derive.name, found.backing_file, api_name
                        )
                    };
                    if confirm(&prompt, true)? {
                        derive.backing_file = found.backing_file.clone();
                        install_derives.push(derive);
                    }
                } else {
                    derive.backing_file = format!("{derives}/{}.jade.toml", derive.name);
                    install_derives.push(derive);
                }
            }
            for (i, derive) in install_derives.iter().enumerate() {
                println!(
                    "({}/{}) installing derivation for {}",
                    i + 1,
                    install_derives.len(),
                    derive.name
                );
                derive.write_back()?;
            }
            if install_derives.is_empty() {
                println!("no changes made")
            } else {
                println!("complete! ")
            }
        }
        Commands::List { ref filter } => {
            let (manifest, derives) = load_context("./", &args)?;
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
        Commands::Version => println!("{}", get_version()),
    }
    Ok(())
}
fn main() {
    let args = Args::parse();
    // println!("{args:?}");
    // panic!();

    match entry(args) {
        Ok(()) => (),
        Err(e) => println!("Error: {e}"),
    };
}

fn get_version() -> String {
    format!("version {}-{}", VERSION, current_platform::CURRENT_PLATFORM)
}
