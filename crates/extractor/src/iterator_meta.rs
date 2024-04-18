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


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;

#[derive(Clone)]
pub struct IteratorMeta {
    key_type: Option<String>,
}

impl IteratorMeta {
    pub fn new(
        key_type: Option<String>,
    ) -> Self {
        IteratorMeta {
            key_type,
        }
    }
}