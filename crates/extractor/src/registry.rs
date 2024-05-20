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

use bnf::ParseTree;


#[derive(Clone)]
pub struct Constraint {
    pub cid: usize,
    pub from: Option<Box<Vec<ParseTree>>>,
    pub to: Option<Box<Vec<ParseTree>>>,
}
impl Constraint {
    pub fn new(cid: usize) -> Self {
        Self {
            cid,
            from: None,
            to: None,
        }
    }
}

#[derive(Clone)]
pub struct Registry {
    pub queue_per_step: Vec<Vec<Executable>>,
    pub visited: HashMap<usize, Executable>,
    pub constraints: HashMap<usize, Constraint>, // constraint_cid, from|to, target_cid
    pub iterish_from_to: HashMap<usize, (usize, usize)>, // key=ast_id
    pub output_flatten: HashMap<usize, Executable>, // key=ast_id
    pub types: Value, // ast info
    pub absolute_slots: HashMap<usize, String>, // key=step, ast_id
    pub values: HashMap<usize, String>, // key=ast_id
}

impl Registry {
    pub fn new(blob:Value, constraints: HashMap<usize, Constraint>, bundle: String) -> Self {

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
            match self.eval_config(e.clone()) {
                Ok((op_from_str, op_to_str)) => {
                    self.iterish_from_to.insert(*id, (op_from_str.unwrap().parse::<usize>().unwrap(), op_to_str.unwrap().parse::<usize>().unwrap()));
                },
                Err(err) => panic!("{}", err),
            };
        };
        self
    }
    fn get_iid(&self, from_length_target_cid: usize) -> Result<usize, Box<dyn Error>> {
        for (iid, e) in self.visited.clone() { // ast_instance_id
            if e.cid() == from_length_target_cid {
                return Ok(iid);
            }
        }
        panic!("target_cid:{} hasn't visited yet.", from_length_target_cid);
    }
    pub fn eval_config(&self, e: Executable)-> Result<(Option<String>, Option<String>), Box<dyn Error>> {
        // Ref: mc_repo_fetcher:L137
        let constraint_cid = e.cid();
        if e.is_iterish() && !self.constraints.contains_key(&constraint_cid) {
            panic!("{} is iterish node in the guest protocol schema and was not in constraints definition in Indexer.yaml of the guest protocol repo. Please consider adding {} to Indexer.yaml", e.fullname(), e.fullname_in_conf());        
        } else if !self.constraints.contains_key(&constraint_cid) {
            panic!("{} was not in constraints definition in Indexer.yaml of the guest protocol repo.", e.fullname());
        }

        let from_stack = ConfigUtil::eval_parse_tree(self.constraints[&constraint_cid].from, None);
        let to_stack = ConfigUtil::eval_parse_tree(self.constraints[&constraint_cid].to, None);
        Ok((from_stack.get(0).cloned(), to_stack.get(0).cloned()))
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
            let children = executable.children(&_self.clone(), None).unwrap();

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