use super::compiler::Compiler;
use super::extractor::Extractor;
use super::executor::Executor;
// use super::registry::Registry;
use super::executable::Executable;
use super::perf_config_item::PerfConfigItem;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
use super::iterator_meta::IteratorMeta;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
use super::ast_node::ASTNode;
use super::ast_node::Node;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;

pub struct Registry<'registry_lifetime> {
    perf_config_items: HashMap<String, PerfConfigItem>,
    output_flatten: HashMap<String, &'registry_lifetime Executable<'registry_lifetime>>,
    ast_node: ASTNode,
}

impl Registry<'_> {
    pub fn new(perf_config_items: HashMap<String, PerfConfigItem>, storage_layout: Value) -> Self {
        let ast_node = ASTNode::from_storage_layout(storage_layout);
        Self {
            perf_config_items,
            output_flatten: HashMap::new(),
            ast_node,
        }
    }

    pub fn set_output(&mut self, edfs: String, e: &Executable) {
        self.output_flatten.insert(edfs, e);
    }

    pub fn get_output(&self, edfs: &str) -> Option<&&Executable> {
        self.output_flatten.get(edfs)
    }

    pub fn get_output_flatten(&self) -> &HashMap<String, &Executable> {
        &self.output_flatten
    }

    pub fn get_perf_config_item(&self, edfs: String) -> Option<&PerfConfigItem> {
        self.perf_config_items.get(&edfs)
    }

    pub fn get_node(&self, edfs: &str) -> Option<&Node> {
        self.ast_node.get_node(edfs)
    }
}