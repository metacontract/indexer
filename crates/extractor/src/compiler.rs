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
    base_path_buf: PathBuf,
}

impl Compiler {
    pub fn new(solc_path: String, base_path_buf: PathBuf) -> Self {
        Self {
            solc_path,
            base_path_buf,
        }
    }

    pub fn prepare_base_slots(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let standard_json_input_path = self.base_path_buf.join(env::var("STANDARD_JSON_INPUT_BASESLOTS_NAME").unwrap());

        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(standard_json_input_path)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        Ok(parsed)
    }

    pub fn prepare_storage_layout(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let standard_json_input_path = self.base_path_buf.join(env::var("STANDARD_JSON_INPUT_LAYOUT_NAME").unwrap());

        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(standard_json_input_path)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        Ok(parsed)
    }


}

#[cfg(test)]
mod tests {
    use super::*;

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
            fetcher.clone_repo().unwrap();

            let mut compiler = Compiler::new("solc".to_string(), pathbuf_temppath.clone());
            let storage_layout_blob = match compiler.prepare_storage_layout() {
                Ok(blob) => blob,
                Err(err) => {
                    panic!("Error preparing storage layout: {}", err);
                }
            };
    
            println!("storage_layout_blob: {:?}", storage_layout_blob);
            assert!(storage_layout_blob["contracts"]["src/_utils/Dummy.sol"]["Dummy"]["storageLayout"]["types"].is_object());
        }
    }

}