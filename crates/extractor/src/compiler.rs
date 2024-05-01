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
    bundle: String,
    solc_path: String,
    base_path: PathBuf,
    local_repo_path: PathBuf,
}

impl Compiler {
    pub fn new(solc_path: String, base_path: PathBuf, identifier: String, bundle:String) -> Self {
        Self {
            bundle: bundle.clone(),
            solc_path,
            base_path: base_path
                .join(env::var("REPO_PATH").unwrap())
                .clone(),
            local_repo_path: base_path
                .join(env::var("REPO_PATH").unwrap())
                .join(identifier.clone())
                .clone(),
        }
    }


    pub fn prepare_storage_layout(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let standard_json_input_path = self.base_path
                                                        // .join(env::var("REPO_PATH").unwrap())
                                                        .join(env::var("STANDARD_JSON_INPUT_LAYOUT_NAME").unwrap());


        match Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(standard_json_input_path.clone())
            .arg("--allow-paths")
            .arg(self.base_path.clone())
            .arg("--base-path")
            .arg(self.local_repo_path.clone())
            // .arg("--devdoc")
            .output() {
                Ok(output)=>{
                    let stdout = String::from_utf8(output.stdout)?;
                    if stdout.len() == 0 {
                        panic!("solc compilation for storage layout generated null result.");
                    }

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

    pub fn prepare_base_slots(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let standard_json_input_path = self.base_path
                                                        // .join(env::var("REPO_PATH").unwrap())
                                                        .join(env::var("STANDARD_JSON_INPUT_BASESLOTS_NAME").unwrap());

        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(standard_json_input_path.clone())
            .arg("--allow-paths")
            .arg(self.base_path.clone())
            .arg("--base-path")
            .arg(self.local_repo_path.clone())
            // .arg("--devdoc")
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        if stdout.len() == 0 {
            panic!("solc compilation for baseslots generated null result.");
        }

        let baseslots_blob: Value = serde_json::from_str(&stdout)?;

        let bytecode = baseslots_blob
                                .get("contracts").unwrap()
                                .get(&format!("src/{}/storages/BaseSlots.sol", self.bundle.clone())).unwrap()
                                .get("BaseSlots").unwrap()
                                .get("evm").unwrap()
                                .get("deployedBytecode").unwrap()
                                .get("object").unwrap()
                                .as_str()
                                .expect("Failed to extract bytecode");

        let baseslots = match Compiler::get_slots(&bytecode) {
            Ok(baseslots) => baseslots,
            Err(err) => panic!("{}", err)
        };

        Ok(baseslots)
    }

    #[allow(dead_code)]
    pub fn get_slots(bytecode: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut baseslots_raw = Vec::new();
        let specifier = "5f1b81565b7f";
        let mut i = 0;
        let signature_size = 12;
        let padding = 76;
        while i < bytecode.len() - signature_size {
            if &bytecode[i..i+signature_size] == specifier {
                let slot_value = &bytecode[i+signature_size..i+padding];
                baseslots_raw.push(slot_value);
                i += padding;
            } else {
                i += 2;
            }
        }
        if baseslots_raw.len() == 0 {
            panic!("baseslots are not detected in BaseSlots.sol");
        }
        let mut baseslots = Vec::new();
        for slot in baseslots_raw {
            baseslots.push(format!("0x{}", slot));
        }
        Ok(baseslots)       
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
        if !fetcher.standard_json_input_baseslots_path.exists() {
            let copy_source = env::current_dir().unwrap().join(PathBuf::from(env::var("REPO_PATH").unwrap()).join(env::var("STANDARD_JSON_INPUT_BASESLOTS_SAMPLE_NAME").unwrap()));
            fs::copy(copy_source.clone(), &fetcher.standard_json_input_baseslots_path).unwrap();    
        }

        fetcher.clone_repo().unwrap();
        fetcher.gen_standard_json_input().unwrap();

        let mut compiler = Compiler::new("solc".to_string(), fetcher.base_path.clone(), fetcher.identifier.clone(), fetcher.bundle.clone());
        let storage_layout_blob = match compiler.prepare_storage_layout() {
            Ok(blob) => blob,
            Err(err) => {
                panic!("Error preparing storage layout: {}", err);
            }
        };

        let types = storage_layout_blob
                                .get("contracts").unwrap()
                                .get(&format!("src/{}/storages/Dummy.sol", fetcher.bundle.clone())).unwrap()
                                .get("Dummy").unwrap()
                                .get("storageLayout").unwrap()
                                .get("types").unwrap();

        assert!(types.is_object());
    }

    #[test]
    fn test_prepare_base_slots() {
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
        if !fetcher.standard_json_input_baseslots_path.exists() {
            let copy_source = env::current_dir().unwrap().join(PathBuf::from(env::var("REPO_PATH").unwrap()).join(env::var("STANDARD_JSON_INPUT_BASESLOTS_SAMPLE_NAME").unwrap()));
            fs::copy(copy_source.clone(), &fetcher.standard_json_input_baseslots_path).unwrap();    
        }

        fetcher.clone_repo().unwrap();
        fetcher.gen_standard_json_input().unwrap();

        let mut compiler = Compiler::new("solc".to_string(), fetcher.base_path.clone(), fetcher.identifier.clone(), fetcher.bundle.clone());
        let baseslots = match compiler.prepare_base_slots() {
            Ok(blob) => blob,
            Err(err) => {
                panic!("Error preparing baseslots: {}", err);
            }
        };


        assert!(baseslots.len() > 0);

    }
}