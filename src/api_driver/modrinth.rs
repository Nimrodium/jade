use std::io::Write;
use std::io::stdout;

use crate::api::APIDriver;
use crate::api::HTTPSQuery;
use crate::api::ModResult;
use crate::package::Derivation;
use crate::store::Store;
use serde_json;
use serde_json::Value;
use toml::Table;
const HOSTNAME: &str = "api.modrinth.com";
const preamble1: &str = "api response did not contain key";
const preamble2: &str = "api response contained key";
pub struct ModrinthDriver {
    loader: String,
    versions: Vec<String>,
    limit: String,
}
impl ModrinthDriver {
    pub fn new(cfg: &Table) -> Result<Self, String> {
        // println!("{cfg:?}");
        let cfg = cfg.get("modrinth").unwrap().as_table().unwrap();
        // println!("{cfg:?}");

        Ok(Self {
            loader: cfg
                .get("loader")
                .ok_or(format!("missing config parameter `loader`"))?
                .as_str()
                .ok_or("config parameter `loader` present but not string")?
                .to_string(),
            versions: cfg
                .get("versions")
                .ok_or(format!("missing config parameter `versions`"))?
                .as_array()
                .ok_or("config parameter `versions` present but not array")?
                .to_owned()
                .iter()
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        return format!("versions contained a non-string `{v}`");
                    }
                })
                .collect(),
            limit: cfg
                .get("limit")
                .ok_or(format!("missing config parameter `limit`"))?
                .as_integer()
                .ok_or("config parameter `limit` present but not integer")?
                .to_string(),
        })
    }
    fn get_facets(&self) -> String {
        let versions_str = {
            let mut s = String::new();
            for version in &self.versions {
                s.push_str(&format!(",\"versions:{}\"", version));
            }
            s
        };
        let facets = format!("[[\"categories:{}\"{}]]", self.loader, versions_str);
        // println!("[MODRINTH_API_DRIVER_DEBUG] facets={facets}");
        facets
    }

    fn build_derivation_for(
        &self,
        pkg_id: &str,
        ver_id: Option<&str>,
        seen: &mut Vec<(String, Option<String>)>,
    ) -> Result<Vec<Derivation>, String> {
        let mut formulated_derives: Vec<Derivation> = Vec::new();
        if let Some(seen_pkg) = seen.iter().find(|v| v.0 == pkg_id) {
            println!("{pkg_id} already installed");
            if let Some(ver) = ver_id {
                if let Some(installed_ver) = &seen_pkg.1 {
                    if ver != installed_ver {
                        println!(
                            "warning: version mismatch between {pkg_id}: installed version {ver} but package requested {installed_ver}"
                        );
                    }
                }
            }
            return Ok(formulated_derives);
        }
        print!(
            "deriving {pkg_id}{}... ",
            if let Some(v) = ver_id {
                format!("/{v}")
            } else {
                "".to_string()
            }
        );
        stdout().flush();
        let versions_str = {
            let mut s = String::new();
            for (i, version) in self.versions.iter().enumerate() {
                if i != 0 {
                    s.push(',');
                }
                s.push_str(&format!("\"{}\"", version));
            }
            s
        };
        // let facets = format!("[[\"loader:{}\"{}]]", self.loader, versions_str);
        let base_package = HTTPSQuery::new(HOSTNAME, &format!("v2/project/{pkg_id}"))
            .send()?
            .parse::<Value>()
            .map_err(|e| format!("could not parse api json response {e}"))?
            .as_object()
            .ok_or(format!("api response was not an object"))?
            .to_owned();
        let name = base_package
            .get("slug")
            .ok_or(format!("{preamble1} `slug`"))?
            .as_str()
            .ok_or(format!("{preamble2} `slug` but was not a string"))?;
        let categories = base_package
            .get("categories")
            .ok_or(format!("{preamble1} `categories`"))?
            .as_array()
            .ok_or(format!("{preamble2} `categories` but was not an array"))?
            .to_owned()
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| {
                        format!("{preamble2} `categories` but an element was not a string: {v}")
                    })
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        print!("{name} ");
        stdout().flush();
        let version = if let Some(specific) = ver_id {
            let url = HTTPSQuery::new(HOSTNAME, &format!("v2/project/{pkg_id}/version/{specific}"))
                .add_parameter("loaders", &format!("[\"{}\"]", self.loader))?
                .add_parameter("game_versions", &format!("[{}]", versions_str))?;
            let response = url
                .send()?
                .parse::<Value>()
                .map_err(|e| format!("could not parse api response json {e}"))?;

            response
                .as_object()
                .ok_or(format!("api response was not an object"))?
                .to_owned()
        } else {
            let url = HTTPSQuery::new(HOSTNAME, &format!("v2/project/{pkg_id}/version"))
                .add_parameter("loaders", &format!("[\"{}\"]", self.loader))?
                .add_parameter("game_versions", &format!("[{}]", versions_str))?;
            let response = url
                .send()?
                .parse::<Value>()
                .map_err(|e| format!("could not parse api response json {e}"))?;

            // later maybe make this user selected
            response
                .as_array()
                .ok_or(format!("api response was not an array"))?
                .get(0)
                .ok_or(format!(
                    "no results for {name} with loader {} and versions {:?}",
                    self.loader, self.versions
                ))?
                .as_object()
                .ok_or(format!("api response was not an object"))?
                .to_owned()
        };
        println!("✓");
        let version_id = version
            .get("id")
            .ok_or(format!("{preamble1} `id`"))?
            .as_str()
            .ok_or(format!("{preamble2} `files` but was not a string"))?;
        let files = version
            .get("files")
            .ok_or(format!("{preamble1} `files`"))?
            .as_array()
            .ok_or(format!("{preamble2} `files` but was not an array"))?
            .to_owned()
            .iter()
            .map(|v| {
                v.as_object().cloned().ok_or_else(|| {
                    format!("{preamble2} `files` but an element was not an object: {v}")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let file = files[0].clone(); // might change this to user selected
        let url = file
            .get("url")
            .ok_or(format!("{preamble1} `url`"))?
            .as_str()
            .ok_or(format!("{preamble2} `url` but was not a string"))?;
        let file_name = file
            .get("filename")
            .ok_or(format!("{preamble1} `filename`"))?
            .as_str()
            .ok_or(format!("{preamble2} `filename` but was not a string"))?;
        let hash = Some(
            file.get("hashes")
                .ok_or(format!("{preamble1} `hashes`"))?
                .as_object()
                .ok_or(format!("{preamble2} `hashes` but was not an object"))?
                .get("sha512")
                .ok_or(format!("in `hashes` {preamble1} `sha512`"))?
                .as_str()
                .ok_or(format!(
                    "in `hashes` {preamble2} `sha512` but was not a string"
                ))?
                .to_string(),
        );
        let depend_ids = version
            .get("dependencies")
            .ok_or(format!("{preamble1} `dependencies`"))?
            .as_array()
            .ok_or(format!("{preamble2}"))?;

        let mut depends: Vec<String> = Vec::new();
        for depend in depend_ids {
            let required = match depend
                .get("dependency_type")
                .ok_or(format!("{preamble1} `dependency_type`"))?
                .as_str()
                .ok_or(format!(
                    "{preamble2} `dependency_type` but was not a string"
                ))? {
                "required" => true,
                _ => false,
            };
            if required {
                let project_id = depend
                    .get("project_id")
                    .ok_or(format!("{preamble1} `project_id`"))?
                    .as_str()
                    .ok_or(format!("{preamble2} `project_id` but was not a string"))?;
                let version_id = depend
                    .get("version_id")
                    .ok_or(format!("{preamble1} `version_id`"))?
                    .as_str();
                // .map(|s| s.to_string());
                // .ok_or(format!("{preamble2} `version_id` but was not a string"))?;
                let derived = self.build_derivation_for(project_id, version_id, seen)?;

                for derive in &derived {
                    depends.push(derive.name.clone());
                }
                formulated_derives.extend(derived);
            }
        }
        // let hash = hash.map(|h| nix_base32::to_nix_base32(&h.into_bytes()));
        // println!("HASH: {hash:?}");
        let master_derive = Derivation::new(
            url,
            name,
            file_name,
            false,
            None,
            hash,
            depends,
            categories,
            Some(pkg_id.to_string()),
            Some(version_id.to_string()),
        );
        formulated_derives.push(master_derive);
        Ok(formulated_derives)
    }
}

impl APIDriver for ModrinthDriver {
    // fn configure(&mut self, cfg: &toml::Table) -> Result<(), String> {
    //     todo!()
    // }

    fn search(&self, query: &str) -> Result<Vec<crate::api::ModResult>, String> {
        println!("searching `{query}`...");
        let url = HTTPSQuery::new("api.modrinth.com", "v2/search")
            .add_parameter("query", query)?
            .add_parameter("facets", &self.get_facets())?
            .add_parameter("limit", &self.limit)?;
        let response: Value = url
            .send()?
            .parse()
            .map_err(|e| format!("could not parse api response json {e}"))?;
        let hits: Vec<Value> = response
            .get("hits")
            .ok_or(format!("{preamble1} `hits`\n{response}"))?
            .as_array()
            .ok_or(format!(
                "{preamble2} `hits` but was not an array\n{response}"
            ))?
            .to_owned();
        let mut mod_results: Vec<ModResult> = Vec::new();
        for hit in hits {
            let id = hit
                .get("project_id")
                .ok_or(format!("{preamble1} `project_id`\n{response}"))?
                .as_str()
                .ok_or(format!("{preamble2} `project_id` but was not a string"))?
                .to_string();
            let slug = hit
                .get("slug")
                .ok_or(format!("{preamble1} `slug`\n{response}"))?
                .as_str()
                .ok_or(format!("{preamble2} `slug` but was not a string"))?
                .to_string();
            let desciption = hit
                .get("description")
                .ok_or(format!("{preamble1} `description`\n{response}"))?
                .as_str()
                .ok_or(format!("{preamble2} `description` but was not a string"))?
                .to_string();
            let author = hit
                .get("author")
                .ok_or(format!("{preamble1} `author`\n{response}"))?
                .as_str()
                .ok_or(format!("{preamble2} `author` but was not a string"))?
                .to_string();
            let categories = hit
                .get("categories")
                .ok_or(format!("{preamble1} `categories`\n{response}"))?
                .as_array()
                .ok_or(format!("{preamble2} `categories` but was not an array"))?
                .to_owned()
                .iter()
                .map(|v| {
                    v.as_str()
                        .ok_or_else(|| {
                            format!("{preamble2} `categories` but an element was not a string: {v}")
                        })
                        .map(|s| s.to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let downloads = hit
                .get("downloads")
                .ok_or(format!("{preamble1} `downloads`\n{response}"))?
                .as_u64()
                .ok_or(format!("{preamble2} `downloads` but was not an integer"))?
                as usize;

            let mod_result = ModResult {
                id: id,
                slug: slug,
                description: desciption,
                downloads: downloads,
                tags: categories,
                author: author,
            };
            mod_results.push(mod_result);
        }
        Ok(mod_results)
    }

    fn get_derivations_for(
        &self,
        pkg_id: &str,
        seen: &mut Vec<(String, Option<String>)>,
        hash: bool,
        store: &Store,
    ) -> Result<Vec<crate::package::Derivation>, String> {
        let mut derivations = self.build_derivation_for(pkg_id, None, seen)?;
        if hash {
            for derive in &mut derivations {
                let file_path = derive.download(
                    &store.temp,
                    derive.hash.clone(),
                    Some("sha512".to_string()),
                )?;
                if store.is_package_in_store(&derive).is_none() {
                    derive.install_to_store(store, &file_path)?;
                }
            }
        }
        Ok(derivations)
    }
}

// macro_rules! extract_key {
//     (table:expr,key:expr,func:expr,datatype:expr) => {
//         {
//             $table.get($key).ok_or($key + "not present")
//         }
//     };
// }
