// use super::compiler::Compiler;
// use super::extractor::Extractor;
// use super::executor::Executor;
// use super::registry::Registry;
// use super::executable::Executable;
// use super::perf_config_item::PerfConfigItem;
// use super::type_kind::TypeKind;
// use super::eth_call::EthCall;
// use super::iterator_meta::IteratorMeta;
// use super::perf_expression_evaluator::PerfExpressionEvaluator;
// use super::ast_node::ASTNode;


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;


pub struct Compiler {
    solc_path: String,
    base_slot_ast_cache: Option<String>,
    storage_layout_ast_cache: Option<String>,
}

impl Compiler {
    pub fn new(solc_path: String) -> Self {
        Self {
            solc_path,
            base_slot_ast_cache: None,
            storage_layout_ast_cache: None,
        }
    }

    pub fn prepare_base_slots(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(ref cache) = self.base_slot_ast_cache {
            return Ok(serde_json::from_str(cache)?);
        }

        let solc_opts = "./solcBaseSlotsOpts.json";
        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(solc_opts)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        self.base_slot_ast_cache = Some(stdout);

        Ok(parsed)
    }

    pub fn prepare_storage_layout(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(ref cache) = self.storage_layout_ast_cache {
            return Ok(serde_json::from_str(cache)?);
        }

        let solc_opts = "./solcLayoutOpts.json";
        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(solc_opts)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        self.storage_layout_ast_cache = Some(stdout);

        Ok(parsed)
    }
}
