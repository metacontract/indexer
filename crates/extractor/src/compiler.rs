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


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;


pub struct Compiler {
    solc_path: String,
}

impl Compiler {
    pub fn new(solc_path: String) -> Self {
        Self {
            solc_path,
        }
    }

    pub fn prepare_base_slots(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let solc_opts = "./solcBaseSlotsOpts.json";
        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(solc_opts)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        Ok(parsed)
    }

    pub fn prepare_storage_layout(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let solc_opts = "./solcLayoutOpts.json";
        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(solc_opts)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_storage_layout() {
        let mut compiler = Compiler::new("solc".to_string());
        let storage_layout_blob = match compiler.prepare_storage_layout() {
            Ok(blob) => blob,
            Err(err) => {
                panic!("Error preparing storage layout: {}", err);
            }
        };

        assert_eq!(storage_layout_blob, 1);
    }
}