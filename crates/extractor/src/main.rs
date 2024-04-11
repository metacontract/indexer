mod compiler;
mod extractor;
mod executor;
mod registry;
mod executable;
mod perf_config_item;
mod type_kind;
mod eth_call;
mod iterator_meta;
mod perf_expression_evaluator;
mod ast_node;

use compiler::Compiler;
use extractor::Extractor;
use executor::Executor;
use registry::Registry;
use executable::Executable;
use perf_config_item::PerfConfigItem;
use type_kind::TypeKind;
use eth_call::EthCall;
use iterator_meta::IteratorMeta;
use perf_expression_evaluator::PerfExpressionEvaluator;
use ast_node::ASTNode;
use ast_node::Node;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;



fn main() {
    let mut extractor = Extractor::new();
    extractor.init_members_from_compiler();
    extractor.listen();
}