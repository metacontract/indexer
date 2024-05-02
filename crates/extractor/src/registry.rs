use super::compiler::Compiler;
use super::extractor::Extractor;
use super::executor::Executor;
// use super::registry::Registry;
use super::executable::Executable;
use super::perf_config_item::PerfConfigItem;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
use super::ast_node::ASTNode;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;



#[derive(Clone)]
pub struct Registry {
    pub queue_per_step: Vec<Vec<Executable>>,
    perf_config_items: HashMap<usize, PerfConfigItem>, // key=ast_id
    pub iterish_from_to: HashMap<usize, (usize, usize)>, // key=ast_id
    pub output_flatten: HashMap<usize, Executable>, // key=ast_id
    pub types: Value, // ast info
    pub absolute_slots: HashMap<usize, String>, // key=step, ast_id
    pub values: HashMap<usize, String>, // key=ast_id
}

impl Registry {
    pub fn new(blob:Value, perf_config_items: HashMap<usize, PerfConfigItem>, bundle: String) -> Self {

        Self {
            queue_per_step: Vec::new(),
            perf_config_items,
            iterish_from_to: HashMap::new(),
            output_flatten: HashMap::new(),
            types: blob["contracts"][format!("src/{}/storages/Dummy.sol", bundle.clone())]["Dummy"]["storageLayout"]["types"].clone(),
            absolute_slots: HashMap::new(),
            values: HashMap::new(),
        }
    }



    pub fn set_primitives(&mut self, primitives: HashMap<usize, Executable>) -> &mut Self {       
        for (id, e) in primitives.iter() {
            self.output_flatten.insert(*id, e.clone());
        };
        self
    }
    #[allow(unused_mut)]
    pub fn bulk_fill_from_to(&mut self, pending_fillable_iterish: &HashMap<usize, Executable>) -> &mut Self {
        for (id, _) in pending_fillable_iterish {
            let (parsed_from, parsed_to) = self.get_parsed_index(*id);
            self.iterish_from_to.insert(*id, (parsed_from, parsed_to));
        };
        self
    }
    pub fn get_parsed_index(&self, ast_id: usize)-> (usize, usize) {
        let perf_config_item = self.get_perf_config_item(ast_id);

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

    pub fn bulk_enqueue_execution(&mut self, step:usize, executables: HashMap<usize, Executable>) -> &mut Self {
        for (_, e) in executables.iter() {
            self.queue_per_step.insert(step, vec![e.clone()]);
        };
        self
    }
    fn enqueue_children_execution(&mut self, step:usize, executable: &Executable) -> &mut Self
    {
        let mut _self = self;
        {
            let from_to = _self.iterish_from_to.get(&executable.id);
            let children = executable.children(&_self.clone(), from_to.clone()).unwrap();
            _self.queue_per_step.insert(step, children);
            _self
        }
    }
    pub fn bulk_enqueue_children_execution(&mut self, step:usize, filled_queueable_iterish: &HashMap<usize, Executable>) -> &mut Self {
        for (_, e) in filled_queueable_iterish.iter() {
            self.enqueue_children_execution(step, e);
        };
        self
    }
    pub fn bulk_set_absolute_slots(&mut self, absolute_slots: &HashMap<usize, String>) -> &mut Self {
        for (id, slot) in absolute_slots.iter() {
            self.absolute_slots.insert(*id, slot.clone());
        };
        self
    }
    pub fn bulk_save_values(&mut self, values:HashMap<usize, String>) -> &mut Self {
        for (id, value) in values.iter() {
            self.values.insert(*id, value.clone());
        };
        self
    }

    pub fn get_perf_config_item(&self, id: usize) -> Option<&PerfConfigItem> {
        self.perf_config_items.get(&id)
    }

    pub fn visit_ast(&self, fulltype: &str) -> Option<Value> {
        return self.types.get(fulltype).cloned();
    }


}