// generic API trait for driving metadata fetch
use urlencoding;
#[derive(Debug)]
pub struct ModResult {
    pub id: String,
    pub slug: String,
    pub description: String,
    pub author: String,
    pub downloads: usize,
    pub tags: Vec<String>,
}

// pub struct Mod {
//     id: String,
//     slug: String,
//     description: String,
//     downloads: usize,

// }

pub struct DownloadOptions {
    sha512: Option<String>,
    file_name: String,
    url: String,
    dependencies: Vec<String>, // slugs
}

pub trait APIDriver {
    // fn configure(&mut self, cfg: &Table) -> Result<(), String>;

    fn search(&self, query: &str) -> Result<Vec<ModResult>, String>;

    fn get_derivations_for(&self, pkg_id: &str) -> Result<Vec<Derivation>, String>;
}

use std::{collections::HashMap, fmt::Display};

use toml::Table;

use crate::{api_driver::modrinth::ModrinthDriver, package::Derivation};
pub fn get_api_driver(name: &str, cfg: &Table) -> Result<Box<dyn APIDriver>, String> {
    // ADD DRIVERS HERE
    match name {
        "modrinth" => Ok(Box::new(
            ModrinthDriver::new(cfg).map_err(|e| format!("[{name}_config_error] {e}"))?,
        )),
        _ => Err(format!("unknown api driver: {name}")),
    }
}

pub struct HTTPSQuery {
    hostname: String,
    endpoint: String,
    parameters: HashMap<String, String>,
}
impl HTTPSQuery {
    pub fn serialize_array(array: &[&dyn Display]) -> String {
        let mut s = String::new();
        s.push('[');
        // let mut toggle = true;
        for (i, e) in array.iter().enumerate() {
            s.push_str(&e.to_string());
            if i != array.len() - 1 {
                s.push(',');
            }
        }
        s.push(']');
        s
    }
    pub fn new(hostname: &str, endpoint: &str) -> Self {
        Self {
            hostname: hostname.to_string(),
            endpoint: endpoint.to_string(),
            parameters: HashMap::new(),
        }
    }
    pub fn add_parameter(mut self, parameter: &str, value: &str) -> Result<Self, String> {
        self.parameters
            .insert(parameter.to_string(), value.to_string());
        Ok(self)
    }

    pub fn formulate(&self) -> String {
        let base = format!("https://{}/{}?", self.hostname, self.endpoint);
        let mut parameter_str = String::new();
        for (parameter, value) in &self.parameters {
            if !parameter_str.is_empty() {
                parameter_str.push('&');
            }
            parameter_str.push_str(&format!(
                "{}={}",
                urlencoding::encode(parameter),
                urlencoding::encode(value)
            ));
        }
        base + &parameter_str
    }

    pub fn send(&self) -> Result<String, String> {
        let url = self.formulate();
        println!("URL: {url}");
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("web request failure: {e}"))?
            .text()
            .map_err(|e| format!("web request decoding error: {e}"))?;
        Ok(response)
    }
}
