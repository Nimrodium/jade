use serde_derive::{Deserialize, Serialize};
use std::{fs::File, io::Read};
use toml::Table;
// #[derive(Deserialize, Serialize)]
// pub struct Manifest {
//     pub name: String,
//     pub game: String,
//     pub api: Option<String>,
//     // #[serde(skip_serializing_if = "Option::is_none")]
//     pub derives: Option<String>, // path to derives default is $PWD/derives
//     pub source: Option<String>,
//     pub target: Option<String>,
//     pub enable_all: Option<bool>,
//     pub enabled_mods: Option<Table>,
// }

// impl Manifest {
//     pub fn load(p: &str) -> Result<Self, String> {
//         let mut contents = String::new();
//         let mut file = File::open(p).map_err(|e| format!("failed to open manifest `{p}`: {e}"))?;
//         file.read_to_string(&mut contents)
//             .map_err(|e| format!("failed to read manifest `{p}`: {e} "))?;
//         Ok(
//             toml::from_str(&contents)
//                 .map_err(|e| format!("failed to parse manifest `{p}`: {e}"))?,
//         )
//     }
// }
#[derive(Deserialize, Serialize)]
pub struct Manifest {
    pub main: ManifestMain,
    pub enabled: Option<Table>,
    #[serde(flatten)]
    pub api_cfg: Table, // modrinth:{},curseforge:{}
}

#[derive(Deserialize, Serialize)]
pub struct ManifestMain {
    pub name: String,
    pub pack_version: String,
    pub derives: Option<String>,
    pub api: Option<String>,
    pub enable_all: bool,
    pub target: Option<String>,
}

impl Manifest {
    pub fn load(p: &str) -> Result<Self, String> {
        let mut contents = String::new();
        let mut file = File::open(p).map_err(|e| format!("failed to open manifest `{p}`: {e}"))?;
        file.read_to_string(&mut contents)
            .map_err(|e| format!("failed to read manifest `{p}`: {e} "))?;
        Ok(
            toml::from_str(&contents)
                .map_err(|e| format!("failed to parse manifest `{p}`: {e}"))?,
        )
    }
}
