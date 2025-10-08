use std::{
    fs,
    io::{Write, stdin, stdout},
    path::Path,
};

use colorize::AnsiColor;
use sha2::{Digest, Sha256, Sha512};

// pub fn verify_hash(bytes: &[u8], hash: &str, hashfmt: &str) -> Result<bool, String> {
//     match hashfmt {
//         "nix" => {
//             let hashed = hash_stream(bytes);
//             if hash == hashed { Ok(true) } else { Ok(false) }
//         }
//         "sha512" => {
//             let hashed = format!("{:x}", Sha512::digest(bytes));
//             // .map_err(|e| format!("failed to convert sha512 hash to string {e}"))?;
//             // println!("hash1: {hash}\nhash2: {hashed}");
//             if hash == hashed { Ok(true) } else { Ok(false) }
//         }

//         _ => Err(format!("unknown hash format {hashfmt}")),
//     }
// }

pub fn hash_stream(byte_stream: &[u8]) -> String {
    nix_base32::to_nix_base32(&Sha256::digest(byte_stream)[..])
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

pub fn remove_fs_entity(p: &str) -> Result<(), String> {
    let path = Path::new(p);
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(|e| format!("failed to remove dir/file `{p}`: {e}"))?;
    Ok(())
}
