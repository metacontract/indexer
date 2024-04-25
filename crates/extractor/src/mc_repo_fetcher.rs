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

pub struct MCRepoFetcher {
    pub url: String,
    pub local_repo_path: PathBuf,
    pub schema_path: PathBuf,
    pub dummy_path: PathBuf,
    pub standard_json_input_path: PathBuf,
}

impl MCRepoFetcher {
    pub fn new(identifier: String, bundle: String) -> Self {
        let local_repo_path = PathBuf::from(format!(".repo/{}", identifier));
        let schema_path = local_repo_path.join(format!("src/{}/storages/Schema.sol", bundle));
        let dummy_path = local_repo_path.join(format!("src/{}/storages/Dummy.sol", bundle));
        let standard_json_input_path = PathBuf::from(env::var("STANDARD_JSON_INPUT_LAYOUT_SAMPLE_PATH").unwrap_or_else(|_| "./standard_json_input_layout_sample.json".to_string()));

        Self {
            url: format!("https://github.com/{}.git", identifier),
            local_repo_path,
            schema_path,
            dummy_path,
            standard_json_input_path,
        }
    }

    pub fn clone_repo(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Check if the target directory already exists
        if self.local_repo_path.exists() && self.local_repo_path.join("lib/mc").exists() {
            if let Err(err) = std::fs::remove_dir_all(&self.local_repo_path) {
                panic!("Failed to remove directory: {}", err);
            }
        }

        // Clone the repository
        let repo = Repository::clone(&self.url, &self.local_repo_path)?;
        println!("Cloned repository: {}", repo.path().display());
        Ok(())
    }

    pub fn gen_standard_json_input(&self) -> Result<(), Box<dyn Error>> {
        let sample_json_content = fs::read_to_string(&self.standard_json_input_path)?;
        let mut sample_json: serde_json::Value = serde_json::from_str(&sample_json_content)?;
        sample_json["sources"] = serde_json::json!({
            self.schema_path.to_str().unwrap(): {
                "urls": [self.schema_path.to_str().unwrap()]
            },
            self.dummy_path.to_str().unwrap(): {
                "urls": [self.dummy_path.to_str().unwrap()]
            }
        });

        let output_json_path = self.standard_json_input_path.with_file_name("standard_json_input_layout.json");
        fs::write(&output_json_path, serde_json::to_string_pretty(&sample_json)?)?;

        println!("Generated standard_json_input_layout.json");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    

    #[test]
    fn test_new() {
        dotenv::dotenv().ok();

        let identifier = env::var("REPO_IDENTIFIER").unwrap();
        let bundle = env::var("BUNDLE_NAME").unwrap();
        let fetcher = MCRepoFetcher::new(identifier.clone(), bundle.clone());

        assert_eq!(fetcher.url, format!("https://github.com/{}.git", identifier));
        assert_eq!(fetcher.local_repo_path, PathBuf::from(format!(".repo/{}", identifier)));
        assert_eq!(fetcher.schema_path, PathBuf::from(format!(".repo/{}/src/{}/storages/Schema.sol", identifier, bundle)));
        assert_eq!(fetcher.dummy_path, PathBuf::from(format!(".repo/{}/src/{}/storages/Dummy.sol", identifier, bundle)));
    }

    #[test]
    fn test_clone_repo() {
        dotenv::dotenv().ok();

        let identifier = env::var("REPO_IDENTIFIER").unwrap();
        let bundle = env::var("BUNDLE_NAME").unwrap();
        let fetcher = MCRepoFetcher::new(identifier.clone(), bundle.clone());
    
        fetcher.clone_repo().unwrap();
    
        assert!(fetcher.local_repo_path.join(".git").exists());
        assert!(fetcher.local_repo_path.join("src").join(bundle.clone()).join("storages").join("Schema.sol").exists());
        assert!(fetcher.local_repo_path.join("src").join(bundle.clone()).join("storages").join("Dummy.sol").exists());
    }

    #[test]
    fn test_gen_standard_json_input() {
        dotenv::dotenv().ok();

        let temp_dir = tempdir().unwrap();
        let standard_json_input_path = temp_dir.path().join("standard_json_input_layout_sample.json");
        fs::write(&standard_json_input_path, r#"{"sources": {}}"#).unwrap();

        let fetcher = MCRepoFetcher {
            url: "".to_string(),
            local_repo_path: temp_dir.path().to_path_buf(),
            schema_path: temp_dir.path().join("src/test_bundle/storages/Schema.sol"),
            dummy_path: temp_dir.path().join("src/test_bundle/storages/Dummy.sol"),
            standard_json_input_path,
        };

        fetcher.gen_standard_json_input().unwrap();

        let output_json_path = temp_dir.path().join("standard_json_input_layout.json");
        let output_json_content = fs::read_to_string(&output_json_path).unwrap();
        let output_json: serde_json::Value = serde_json::from_str(&output_json_content).unwrap();

        assert_eq!(output_json["sources"].as_object().unwrap().len(), 2);
        assert!(output_json["sources"].as_object().unwrap().contains_key(fetcher.schema_path.to_str().unwrap()));
        assert!(output_json["sources"].as_object().unwrap().contains_key(fetcher.dummy_path.to_str().unwrap()));
    }


}