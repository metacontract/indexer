use super::compiler::Compiler;
use super::extractor::Extractor;
use super::executor::Executor;
use super::registry::Registry;
use super::executable::Executable;
use super::perf_config_item::PerfConfigItem;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
// use super::iterator_meta::IteratorMeta;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
use super::ast_node::ASTNode;
use super::ast_node::Node;


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;

pub struct IteratorMeta<'iterator_meta> {
    key_type: Option<String>,
    perf_config: Option<PerfConfigItem>,
    items: Vec<Executable<'iterator_meta>>,
    from: usize,
    pub to: usize,
}

impl IteratorMeta<'_> {
    pub fn new(
        key_type: Option<String>,
        perf_config: Option<PerfConfigItem>,
        items: Vec<Executable>,
        from: usize,
        to: usize,
    ) -> Self {
        IteratorMeta {
            key_type,
            perf_config,
            items,
            from,
            to,
        }
    }


    pub fn set_from(&mut self, from: usize) {
        self.from = from;
    }

    pub fn set_to(&mut self, to: usize) {
        self.to = to;
    }
}