// use super::compiler::Compiler;
// use super::extractor::Extractor;
// use super::executor::Executor;
// use super::registry::Registry;
// use super::executable::Executable;
// use super::perf_config_item::PerfConfigItem;
// use super::type_kind::TypeKind;
// use super::eth_call::EthCall;
// use super::perf_expression_evaluator::PerfExpressionEvaluator;
// use super::ast_node::ASTNode;
use super::mc_repo_fetcher::MCRepoFetcher;


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::env;
use std::path::PathBuf;


pub struct Compiler {
    solc_path: String,
    base_path: PathBuf,
    local_repo_path: PathBuf,
}

impl Compiler {
    pub fn new(solc_path: String, base_path: PathBuf, identifier: String) -> Self {
        Self {
            solc_path,
            base_path: base_path.clone(),
            local_repo_path: base_path.clone().join(identifier),
        }
    }

    pub fn prepare_base_slots(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let standard_json_input_path = self.base_path.join(env::var("STANDARD_JSON_INPUT_BASESLOTS_NAME").unwrap());

        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(standard_json_input_path)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        Ok(parsed)
    }

    pub fn prepare_storage_layout(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let standard_json_input_path = self.base_path.join(env::var("STANDARD_JSON_INPUT_LAYOUT_NAME").unwrap());

        match Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(standard_json_input_path.clone())
            .arg("--allow-paths")
            .arg(self.base_path.clone())
            .arg("--base-path")
            .arg(self.local_repo_path.clone())
            .output() {
                Ok(output)=>{
                    let stdout = String::from_utf8(output.stdout)?;
                    println!("standard_json_input_path:{:?}", standard_json_input_path.clone());

                    match serde_json::from_str(&stdout) {
                        Ok(parsed)=>{
                            Ok(parsed)            
                        },
                        Err(err)=>{
                            panic!("{}", err);
                        }
                    }
                },
                Err(err)=>{
                    panic!("{}", err);
                }
            }
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_prepare_storage_layout() {
        dotenv::dotenv().ok();

        let tempdir = tempdir().unwrap();
        let pathbuf_temppath = tempdir.into_path();

        let fetcher = MCRepoFetcher::new(env::var("REPO_IDENTIFIER").unwrap(), env::var("BUNDLE_NAME").unwrap(), Some(pathbuf_temppath.clone()));

        if !fetcher.local_repo_path.exists() {
            std::fs::create_dir_all(fetcher.local_repo_path.clone()).unwrap();
        }
        if !fetcher.standard_json_input_layout_path.exists() {
            let copy_source = env::current_dir().unwrap().join(PathBuf::from(env::var("REPO_PATH").unwrap()).join(env::var("STANDARD_JSON_INPUT_LAYOUT_SAMPLE_NAME").unwrap()));
            fs::copy(copy_source.clone(), &fetcher.standard_json_input_layout_path).unwrap();    
        }

        fetcher.clone_repo().unwrap();
        fetcher.gen_standard_json_input().unwrap();

        let mut compiler = Compiler::new("solc".to_string(), fetcher.local_repo_path.clone(), fetcher.identifier.clone());
        let storage_layout_blob = match compiler.prepare_storage_layout() {
            Ok(blob) => blob,
            Err(err) => {
                panic!("Error preparing storage layout: {}", err);
            }
        };

        println!("{:?}", storage_layout_blob);
        assert!(storage_layout_blob["contracts"][&fetcher.dummy_path.to_str().unwrap()]["Dummy"]["storageLayout"]["types"].is_object());
    }

}