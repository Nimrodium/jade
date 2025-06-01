use chrono::{DateTime, Utc};
use std::{fs, path::Path};

use crate::package::Derivation;

pub fn update_derives(
    derivations: &[Derivation],
    backup_dir: &str,
    dir: &str,
    pack_name: &str,
) -> Result<(), String> {
    backup_derives(pack_name, dir, backup_dir)?;
    for derivation in derivations {
        derivation.write_back()?;
    }
    Ok(())
}

pub fn backup_derives(pack_name: &str, pack_dir: &str, backup_dir: &str) -> Result<(), String> {
    fs::create_dir_all(backup_dir).map_err(|e| format!("failed to create `{backup_dir}`: {e}"))?;
    let path = Path::new(pack_dir);
    if !path.is_dir() {
        return Err(format!("{pack_dir} is not a directory"));
    }
    let now = Utc::now();

    let backup_folder_name = format!(
        "{backup_dir}/{pack_name}-{}.zip",
        now.format("%m-%d-%y_%H-%M")
    );
    zip_extensions::zip_create_from_directory(
        &Path::new(&backup_folder_name).to_path_buf(),
        &path.to_path_buf(),
    )
    .map_err(|e| format!("failed to backup pack `{pack_dir}` to `{backup_folder_name}`: {e:?}",))?;
    Ok(())
}
