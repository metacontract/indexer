// // use compiler::Compiler;
// use extractor::Extractor;
// use executable::Executable;
// use registry::Registry;
// use eth_call::EthCall;
// use type_kind::TypeKind;
// use perf_config_item::PerfConfigItem;
// use executor::Executor;
// use perf_expression_evaluator::PerfExpressionEvaluator;
// use iterator_meta::IteratorMeta;
// use ast_node::Node;
// use ast_node::ASTNode;

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
