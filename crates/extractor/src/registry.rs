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

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;

pub struct Registry<'registry_lifetime> {
    perf_config_items: HashMap<usize, PerfConfigItem>, // key=astId
    output_flatten: HashMap<usize, &'registry_lifetime Executable<'registry_lifetime>>, // key=astId
    pub ast_node: ASTNode<'registry_lifetime>,
    pub executor: Option<&'registry_lifetime mut Executor<'registry_lifetime>>,
}

impl Registry<'_> {
    pub fn new(perf_config_items: HashMap<usize, PerfConfigItem>, blob: Value) -> Self {
        let ast_node = ASTNode::new(blob);
        Self {
            perf_config_items,
            output_flatten: HashMap::new(),
            ast_node,
            executor: None, // executor
        }
    }
    pub fn set_self_to_ast_node(&mut self) -> () {
        self.ast_node.set_registry(&self);
    }
    pub fn set_executor(&mut self, executor: &mut Executor) -> () {
        self.executor = Some(executor);
    }



    pub fn set_output(&mut self, id: usize, e: &Executable) {
        self.output_flatten.insert(id, e);
    }

    pub fn get_output(&self, id: usize) -> Option<&&Executable> {
        self.output_flatten.get(&id)
    }

    pub fn get_perf_config_item(&self, id: usize) -> Option<&PerfConfigItem> {
        self.perf_config_items.get(&id)
    }
}