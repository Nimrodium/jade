use chrono::Utc;
use colorize::AnsiColor;
use sha2::{Digest, Sha256, Sha512};
use std::{
    fs,
    io::{Write, stdin, stdout},
    path::Path,
};

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

pub fn normalize(s: &str) -> String {
    let mut n = s.to_ascii_lowercase();
    n.retain(|c| c.is_ascii_alphanumeric());
    n
}

pub fn confirm(prompt: &str, default_resp: bool) -> Result<bool, String> {
    let yn_resp = match default_resp {
        true => "[Y/n]",
        false => "[y/N]",
    };
    print!("{}", format!("{prompt} {yn_resp}: ").bold());
    stdout().flush();
    let mut response = String::new();
    stdin()
        .read_line(&mut response)
        .map_err(|e| format!("failed to read stdin: {e}"))?;
    println!();
    match response.trim().to_lowercase().as_str() {
        "y" => Ok(true),
        "n" => Ok(false),
        _ => Ok(default_resp),
    }
}

pub fn select_index(
    prompt: &str,
    default_resp: isize,
    start: isize,
    end: isize,
) -> Result<isize, String> {
    let int_range = format!("[{start}..{end}] ({default_resp})");
    loop {
        print!("{}", &format!("{prompt} {int_range}: ").bold());
        stdout().flush();
        let mut resp = String::new();
        stdin()
            .read_line(&mut resp)
            .map_err(|e| format!("failed to read stdin: {e}"))?;
        println!("{resp}");
        let resp = resp.trim();
        if resp.is_empty() {
            break Ok(default_resp);
        }
        let n: isize = match resp
            .parse()
            .map_err(|e| format!("response must be an integer {e}"))
        {
            Ok(n) => n,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };
        if n < start || n > end {
            println!("{n} not within range {int_range}");
            continue;
        } else {
            break Ok(n);
        }
    }
}

pub fn verify_hash(bytes: &[u8], hash: &str, hashfmt: &str) -> Result<bool, String> {
    match hashfmt {
        "nix" => {
            let hashed = hash_stream(bytes);
            if hash == hashed { Ok(true) } else { Ok(false) }
        }
        "sha512" => {
            let hashed = format!("{:x}", Sha512::digest(bytes));
            // .map_err(|e| format!("failed to convert sha512 hash to string {e}"))?;
            // println!("hash1: {hash}\nhash2: {hashed}");
            if hash == hashed { Ok(true) } else { Ok(false) }
        }

        _ => Err(format!("unknown hash format {hashfmt}")),
    }
}

pub fn hash_stream(byte_stream: &[u8]) -> String {
    nix_base32::to_nix_base32(&Sha256::digest(byte_stream)[..])
}
