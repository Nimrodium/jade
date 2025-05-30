use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
// #[serde(deny_unknown_fields)]
pub struct PackWizMod {
    pub filename: String,
    pub name: String,
    pub side: String,
    pub download: Download,
    pub option: Option<PackWizOption>,
}

#[derive(Deserialize, Debug)]
pub struct Download {
    #[serde(rename = "hash-format")]
    pub hash_format: String,
    pub hash: String,
    pub url: String,
}

#[derive(Deserialize)]
struct Update {}

#[derive(Deserialize, Debug)]
struct PackWizOption {
    optional: bool,
    default: bool,
    description: String,
}
