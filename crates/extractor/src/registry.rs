use super::compiler::Compiler;
use super::extractor::Extractor;
use super::executor::Executor;
// use super::registry::Registry;
use super::executable::Executable;
use super::config_util::ConfigUtil;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
use super::ast_node::ASTNode;
use super::mc_repo_fetcher::MCRepoFetcher;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::error::Error;
use std::result::Result;
use std::result::Result::{Ok, Err};


#[derive(Clone)]
pub struct Registry {
    pub queue_per_step: Vec<Vec<Executable>>,
    pub visited: HashMap<usize, Executable>,
    pub constraints: HashMap<usize, HashMap<String, usize>>, // constraint_cid, from|to, target_cid
    pub iterish_from_to: HashMap<usize, (usize, usize)>, // key=ast_id
    pub output_flatten: HashMap<usize, Executable>, // key=ast_id
    pub types: Value, // ast info
    pub absolute_slots: HashMap<usize, String>, // key=step, ast_id
    pub values: HashMap<usize, String>, // key=ast_id
}

impl Registry {
    pub fn new(blob:Value, constraints: HashMap<usize, HashMap<String, usize>>, bundle: String) -> Self {

        Self {
            queue_per_step: Vec::new(),
            visited: HashMap::new(),
            constraints,
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
        for (id, e) in pending_fillable_iterish {
            let (parsed_from, parsed_to) = self.get_parsed_index(e.clone());
            self.iterish_from_to.insert(*id, (parsed_from, parsed_to));
        };
        self
    }
    fn get_iid(&self, from_length_target_cid: usize) -> Result<usize, Box<dyn Error>> {
        for (iid, e) in self.visited.clone() { // ast_instance_id
            let _ancestors = e.ancestors();
            let fullname = _ancestors.iter().map(|executable| executable.name.clone()).collect::<Vec<_>>().join("");
            let class_paths = ConfigUtil::to_class_paths(fullname);

            let visited_cid = ConfigUtil::calc_id(class_paths);
            if visited_cid == from_length_target_cid {
                return Ok(iid);
            }
        }
        panic!("target_cid:{} hasn't visited yet.", from_length_target_cid);
    }
    pub fn get_parsed_index(&self, e: Executable)-> (usize, usize) {
        // Ref: mc_repo_fetcher:L137
        let fullname = e.fullname();
        // TODO: iter.child[i] is like ["iter", "child", "child[i]"] in Executable. But what we want is ["iter", "child", "[i]"]
        let constraint_cid = ConfigUtil::calc_id(ConfigUtil::to_class_paths(fullname));

        let from_length_target_cid = self.constraints[&constraint_cid]["from"];
        let from_length = match self.get_iid(from_length_target_cid) {
            Ok(from_length_target_iid) => self.values[&from_length_target_iid].clone(),
            Err(err) => panic!("{}", err),
        };

        let to_length_target_cid = self.constraints[&constraint_cid]["to"];
        let to_length = match self.get_iid(to_length_target_cid) {
            Ok(to_length_target_iid) => self.values[&to_length_target_iid].clone(),
            Err(err) => panic!("{}", err),
        };
        let to_length: usize = to_length.parse().unwrap();
        let from_length: usize = from_length.parse().unwrap();

        (from_length, to_length)
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
    pub fn bulk_save_visited(&mut self, visited:Vec<Executable>) -> &mut Self {
        for e in visited.iter() {
            self.visited.insert(e.id, e.clone());
        };
        self
    }


    pub fn visit_ast(&self, fulltype: &str) -> Option<Value> {
        return self.types.get(fulltype).cloned();
    }


}