use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

use toml::Table;

use crate::{drivers, package::Package};

pub trait Driver {
    /// get a list of results for a query
    fn search(&self, slug: &str, results: usize) -> Result<Vec<DriverResult>, String>;
    /// resolve a DriverResult into a downloadable url.
    // fn resolve(&self, result: &DriverResult) -> Result<String, String>;
    /// download from source and produce a path.
    fn download(&self, result: &DriverResult, version: &str) -> Result<PathBuf, String>;
    fn build_package(&self, result: &DriverResult, version: Version) -> Result<Package, String>;
}
pub enum Version {
    Latest,
    V(String),
}
pub struct DriverResult {
    pkg: String,
    info: String,
    author: String,
    downloads: String,
}
// pub struct DriverRegistry;
// impl DriverRegistry {

// }

fn get_driver(driver_id: &str) -> Option<Box<dyn Driver>> {
    match driver_id {
        "modrinth" => Some(Box::new(drivers::modrinth::ModrinthDriver)),
        _ => None,
    }
}

pub struct HTTPSQuery {
    hostname: String,
    endpoint: String,
    parameters: HashMap<String, String>,
}
impl HTTPSQuery {
    pub fn serialize_array(array: &[&dyn fmt::Display]) -> String {
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
        // println!("URL: {url}");
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("web request failure: {e}"))?
            .text()
            .map_err(|e| format!("web request decoding error: {e}"))?;
        Ok(response)
    }
}
