// parse packwiz files and collect all their urls and then download
use std::{fs::File, io::read_to_string};
mod composer;
mod package;
mod packwiz_compat;
use package::JadeLock;
use tokio::runtime;
fn main() {
    println!("Hello, world!");
    let test_f = File::open("/home/kyle/.jade/mods/rubidium.pw.toml").unwrap();
    let jade_flake = File::open("./tufx.jade.toml").unwrap();

    let rubidium: packwiz_compat::PackWizMod =
        toml::from_str(&read_to_string(test_f).unwrap()).unwrap();
    println!("{rubidium:#?}");

    let tufx = toml::from_str::<package::JadeFlake>(&read_to_string(jade_flake).unwrap()).unwrap();
    println!("{tufx:#?}");

    let client = reqwest::Client::new();
    let jade_root = "/home/kyle/.jade/";
    let lock_file = format!("{jade_root}/jade.lock.toml");
    let lock = true;

    let mut lock_file_obj = if lock {
        Some(JadeLock::load(&lock_file).unwrap())
    } else {
        None
    };

    let mut generic_rubidium = composer::Package::from_packwiz(rubidium).unwrap();
    let mut generic_tufx = composer::Package::from_jade_flake(tufx, &lock_file_obj).unwrap();

    let async_runtime = runtime::Runtime::new().unwrap();
    async_runtime.block_on(async {
        let composed_artifacts = composer::compose(
            jade_root,
            vec![generic_tufx, generic_rubidium],
            &mut lock_file_obj,
            true,
        )
        .await
        .unwrap();

        if let Some(lockf) = lock_file_obj {
            lockf.write_to_disk().unwrap();
        };
        // let store_tufx = generic_tufx
        //     .fetch_package(&client, store, true, false)
        //     .await
        //     .unwrap();
        // let store_rubidium = generic_rubidium
        //     .fetch_package(&client, store, true, false)
        //     .await
        //     .unwrap();

        // println!("rubidium: {store_rubidium}\ntufx: {store_tufx}");
    })
}
