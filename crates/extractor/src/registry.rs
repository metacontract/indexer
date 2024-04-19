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
    pub queue_per_step: Vec<Vec<Executable<'a>>>,
    pub executed_per_step: Vec<Vec<Executable<'a>>>,
    perf_config_items: HashMap<usize, PerfConfigItem>, // key=astId
    pub iterish_from_to: HashMap<usize, (usize, usize)>, // key=astId
    output_flatten: HashMap<usize, Executable<'a>>, // key=astId
    pub types: Value,
    pub absolute_slots: HashMap<usize, String>, // key=astId
    pub values: HashMap<usize, String>, // key=astId
}

impl<'b, 'a: 'b> Registry<'a> {
    pub fn new(blob:Value, perf_config_items: HashMap<usize, PerfConfigItem>) -> Self {
        Self {
            queue_per_step: Vec::new(),
            executed_per_step: Vec::new(),
            perf_config_items,
            iterish_from_to: HashMap::new(),
            output_flatten: HashMap::new(),
            types: blob["contracts"]["src/_utils/Dummy.sol"]["Dummy"]["storageLayout"]["types"].clone(),
            absolute_slots: HashMap::new(),
            values: HashMap::new(),
        }
    }



    pub fn set_primitives(mut self, primitives: &HashMap<usize, Executable<'a>>) -> Self {        
        for (id, e) in primitives.iter() {
            self.output_flatten.insert(*id, e.clone());
        }
        self
    }
    #[allow(unused_mut)]
    pub fn bulk_fill_from_to(mut self, pending_fillable_iterish: &HashMap<usize, Executable<'b>>) -> Self {
        for (id, _) in pending_fillable_iterish {
            let (parsed_from, parsed_to) = self.get_parsed_index(*id);
            self.iterish_from_to.insert(*id, (parsed_from, parsed_to));
        }
        self
    }
    pub fn get_parsed_index(&self, astId: usize)-> (usize, usize) {
        let perf_config_item = self.get_perf_config_item(astId);

        let from_expression = perf_config_item.as_ref().and_then(|item| item.from.clone());
        let to_expression = perf_config_item.as_ref().and_then(|item| item.to.clone());
        let parsed_from = if let Some(from_expression) = from_expression {
            PerfExpressionEvaluator::eval(from_expression, &self)
        } else {
            panic!("from_expression is None");
        };
        let parsed_to = if let Some(to_expression) = to_expression {
            PerfExpressionEvaluator::eval(to_expression, &self)
        } else {
            panic!("to_expression is None");
        };

        (parsed_from, parsed_to)
    }
    pub fn enqueue_execution(&mut self, step: usize, executable: Executable<'a>) -> () {
        self.queue_per_step.insert(step, vec![executable]);
    }
    fn enqueue_children_execution(&mut self, step:usize, executable: Executable<'a>){
        let (_, to) = match self.iterish_from_to.get(&executable.id) {
            Some((from, to)) => (*from, *to),
            None => {
                panic!("No from/to values found for executable with ID: {}", executable.id);
            }
        };
        let children = executable.children(to, self);
        for child in children {
            self.enqueue_execution(step, child.clone());
        }
    }
    pub fn bulk_enqueue_execution(mut self, step:usize, executables: HashMap<usize, Executable<'a>>) -> Self {
        for (_, e) in executables.iter() {
            self.enqueue_execution(step, e.clone());
        }
        self
    }
    pub fn bulk_enqueue_children_execution(mut self, step:usize, filled_queueable_iterish: HashMap<usize, Executable<'a>>) -> Self {
        for (_, e) in filled_queueable_iterish.iter() {
            self.enqueue_children_execution(step, e.clone());
        }
        self
    }
    pub fn bulk_set_absolute_slots(mut self, absolute_slots: &HashMap<usize, String>) -> Self {
        for (id, slot) in absolute_slots.iter() {
            self.absolute_slots.insert(*id, slot.clone());
        }
        self
    }
    pub fn bulk_save_values(mut self, values:HashMap<usize, String>) -> Self {
        for (id, value) in values.iter() {
            self.values.insert(*id, value.clone());
        }
        self        
    }

    pub fn get_output(&self, id: usize) -> Option<&Executable> {
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

    pub fn visitAST(&self, label: &str) -> Option<Value> {
        return self.types.get(label).cloned();
    }


}