use super::executor::Executor;
use super::registry::Registry;
use super::ast_node::ASTNode;
use std::collections::HashMap;
use serde_json::Value;
use git2::Repository;
use std::fs;
use std::error::Error;
use std::env;
use std::path::PathBuf;
use std::path::Path;


#[allow(unused)]
pub struct MCRepoFetcher {
    pub url: String,
    pub base_path: PathBuf,
    pub identifier: String,
    pub bundle: String,
    pub local_repo_path: PathBuf,
    pub identifier_path: PathBuf,
    pub schema_path: PathBuf,
    pub dummy_path: PathBuf,
    pub standard_json_input_layout_sample_path: PathBuf,
    pub standard_json_input_layout_path: PathBuf,
    pub standard_json_input_baseslots_sample_path: PathBuf,
    pub standard_json_input_baseslots_path: PathBuf,
}

impl MCRepoFetcher {
    pub fn new(identifier: String, bundle: String, base_path: Option<PathBuf>) -> Self {
        let _base_path = base_path.unwrap_or_else(|| env::current_dir().unwrap() );
        let local_repo_path = _base_path.join(format!("{}", env::var("REPO_PATH").unwrap()));
        let identifier_path = local_repo_path.join(format!("{}", identifier));
        let storage_path = identifier_path.join(format!("src/{}/storages", bundle));
        let schema_path = storage_path.join(format!("Schema.sol"));
        let dummy_path = storage_path.join(format!("Dummy.sol"));
        let standard_json_input_layout_sample_path = local_repo_path.join(format!("{}", env::var("STANDARD_JSON_INPUT_LAYOUT_SAMPLE_NAME").unwrap_or_else(|_| "standard_json_input_layout_sample.json".to_string())));
        let standard_json_input_layout_path = local_repo_path.join(format!("{}", env::var("STANDARD_JSON_INPUT_LAYOUT_NAME").unwrap_or_else(|_| "standard_json_input_layout.json".to_string())));
        let standard_json_input_baseslots_sample_path = local_repo_path.join(format!("{}", env::var("STANDARD_JSON_INPUT_BASESLOTS_SAMPLE_NAME").unwrap_or_else(|_| "standard_json_input_baseslots_sample.json".to_string())));
        let standard_json_input_baseslots_path = local_repo_path.join(format!("{}", env::var("STANDARD_JSON_INPUT_BASESLOTS_NAME").unwrap_or_else(|_| "standard_json_input_baseslots.json".to_string())));

        Self {
            url: format!("https://github.com/{}.git", identifier),
            base_path: _base_path,
            identifier,
            bundle,
            local_repo_path,
            identifier_path,
            schema_path,
            dummy_path,
            standard_json_input_layout_sample_path,
            standard_json_input_layout_path,
            standard_json_input_baseslots_sample_path,
            standard_json_input_baseslots_path,
        }
    }

    pub fn clone_repo(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Check if the target directory already exists
        if self.local_repo_path.exists() && self.identifier_path.join("lib/mc").exists() {
            if let Err(err) = std::fs::remove_dir_all(&self.identifier_path) {
                panic!("Failed to remove directory: {}", err);
            }
        }

        // Clone the repository
        let repo = Repository::clone(&self.url, &self.identifier_path)?;
        println!("Cloned repository: {}", repo.path().display());
        Ok(())
    }

    pub fn gen_standard_json_input(&self) -> Result<(), Box<dyn Error>> {
        if self.standard_json_input_layout_path.exists() {
            match fs::read_to_string(&self.standard_json_input_layout_path) {
                Ok(content) => {
                    let sample_json: serde_json::Value = serde_json::from_str(&content)?;
                    let output_json_path: &std::path::Path = self.standard_json_input_layout_path.as_ref();
                    
                    match fs::write(output_json_path, serde_json::to_string_pretty(&sample_json)?) {
                        Ok(_) => {
                            println!("Generated standard_json_input_layout.json");
                        },
                        Err(err) => {
                            panic!("Error writing standard_json_input_layout_path: {}", err);
                        }
                    }
                },
                Err(err) => {
                    panic!("Error reading standard_json_input_layout_path: {}", err);
                }
            };
        } else {
            panic!("standard_json_input_layout_path({}) isn't exist.", self.standard_json_input_layout_path.to_str().unwrap());            
        }
        if self.standard_json_input_baseslots_path.exists() {
            match fs::read_to_string(&self.standard_json_input_baseslots_path) {
                Ok(content) => {
                    let sample_json: serde_json::Value = serde_json::from_str(&content)?;
                    let output_json_path: &std::path::Path = self.standard_json_input_baseslots_path.as_ref();
                    
                    match fs::write(output_json_path, serde_json::to_string_pretty(&sample_json)?) {
                        Ok(_) => {
                            println!("Generated standard_json_input_baseslots.json");
                        },
                        Err(err) => {
                            panic!("Error writing standard_json_input_baseslots_path: {}", err);
                        }
                    }
                },
                Err(err) => {
                    panic!("Error reading standard_json_input_baseslots_path: {}", err);
                }
            };
        } else {
            panic!("standard_json_input_baseslots_path({}) isn't exist.", self.standard_json_input_baseslots_path.to_str().unwrap());            
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    fn initialize() -> MCRepoFetcher {
        dotenv::dotenv().ok();

        let tempdir = tempdir().unwrap();
        let pathbuf_temppath = tempdir.into_path();

        let identifier = env::var("REPO_IDENTIFIER").unwrap();
        let bundle = env::var("BUNDLE_NAME").unwrap();

        let fetcher = MCRepoFetcher::new(identifier.clone(), bundle.clone(), Some(pathbuf_temppath));


        if let Err(_err) = std::fs::remove_dir_all(&fetcher.local_repo_path) {
            
        }

        fetcher
    }

    #[test]
    fn test_new() {
        let fetcher = initialize();
    
        assert_eq!(fetcher.url, format!("https://github.com/{}.git", fetcher.identifier));
        assert_eq!(fetcher.local_repo_path, fetcher.base_path.join(format!(".repo")));
        assert_eq!(fetcher.schema_path, fetcher.base_path.join(format!(".repo/{}/src/{}/storages/Schema.sol", fetcher.identifier, fetcher.bundle)));
        assert_eq!(fetcher.dummy_path, fetcher.base_path.join(format!(".repo/{}/src/{}/storages/Dummy.sol", fetcher.identifier, fetcher.bundle)));
    }

    #[test]
    fn test_clone_repo() {
        let fetcher = initialize();
    
        fetcher.clone_repo().unwrap();
    
        assert!(fetcher.identifier_path.join(".git").exists());
        assert!(fetcher.identifier_path.join("src").join(fetcher.bundle.clone()).join("storages").join("Schema.sol").exists());
        assert!(fetcher.identifier_path.join("src").join(fetcher.bundle.clone()).join("storages").join("Dummy.sol").exists());
    }

    #[test]
    fn test_gen_standard_json_input() {
        let fetcher = initialize();

        if !fetcher.local_repo_path.exists() {
            std::fs::create_dir_all(fetcher.local_repo_path.clone()).unwrap();
        }
        if !fetcher.standard_json_input_layout_path.exists() {
            let copy_source = env::current_dir().unwrap().join(PathBuf::from(env::var("REPO_PATH").unwrap()).join(env::var("STANDARD_JSON_INPUT_LAYOUT_SAMPLE_NAME").unwrap()));
            fs::copy(copy_source.clone(), &fetcher.standard_json_input_layout_path).unwrap();    
        }
        if !fetcher.standard_json_input_baseslots_path.exists() {
            let copy_source = env::current_dir().unwrap().join(PathBuf::from(env::var("REPO_PATH").unwrap()).join(env::var("STANDARD_JSON_INPUT_BASESLOTS_SAMPLE_NAME").unwrap()));
            fs::copy(copy_source.clone(), &fetcher.standard_json_input_baseslots_path).unwrap();    
        }

        fetcher.gen_standard_json_input().unwrap();

        let output_json_path = fetcher.local_repo_path.join(env::var("STANDARD_JSON_INPUT_LAYOUT_NAME").unwrap());
        let output_json_content = fs::read_to_string(&output_json_path).unwrap();
        let output_json: serde_json::Value = serde_json::from_str(&output_json_content).unwrap();

        assert_eq!(output_json["sources"].as_object().unwrap().len(), 2);
        assert!(output_json["sources"].as_object().unwrap().contains_key(&format!("src/{}/storages/Schema.sol", fetcher.bundle)));
        assert!(output_json["sources"].as_object().unwrap().contains_key(&format!("src/{}/storages/Dummy.sol", fetcher.bundle)));

    }


}