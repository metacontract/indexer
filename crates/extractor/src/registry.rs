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



#[derive(Clone)]
pub struct Registry<'a> {
    pub queue_per_step: Vec<Vec<&'a Executable<'a>>>,
    pub executed_per_step: Vec<Vec<&'a mut Executable<'a>>>,
    perf_config_items: HashMap<usize, PerfConfigItem>, // key=astId
    output_flatten: HashMap<usize, &'a Executable<'a>>, // key=astId
}

impl Registry<'_> {
    pub fn new(perf_config_items: HashMap<usize, PerfConfigItem>) -> Self {
        Self {
            queue_per_step: Vec::new(),
            executed_per_step: Vec::new(),
            perf_config_items,
            output_flatten: HashMap::new(),
        }
    }



    pub fn set_primitives(&mut self, primitives: HashMap<usize, Executable>) -> () {        
        for (id, e) in primitives.iter() {
            self.output_flatten.insert(*id, e.clone());
        }
    }

    pub fn get_output(&self, id: usize) -> Option<&&Executable> {
        self.output_flatten.get(&id)
    }

    pub fn get_perf_config_item(&self, id: usize) -> Option<&PerfConfigItem> {
        self.perf_config_items.get(&id)
    }

    pub fn flush_queue(&mut self, step: usize) {
        self.queue_per_step[step].clear();
    }

    pub fn flush_executed(&mut self, step: usize) {
        self.executed_per_step[step].clear();
    }

}