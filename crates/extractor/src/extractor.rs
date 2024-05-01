use super::compiler::Compiler;
use super::executor::Executor;
use super::registry::Registry;
use super::executable::Executable;
use super::perf_config_item::PerfConfigItem;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
use super::ast_node::ASTNode;
use super::context::Context;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem;

pub struct Extractor {
    initial_members: Vec<Executable>,
    state: ExtractorState,
}

struct ExtractorState {
    step: usize,
    context: Context,
}

impl Extractor {
    pub fn new(context: Context) -> Self {
        Self {
            initial_members: Vec::new(),
            state: ExtractorState {
                step: 0,
                context,
            },
        }
    }

    pub fn init_members_from_compiler(&mut self, base_slots_index: &HashMap<String,String>) {
        let mut base_slots: Vec<(String, Value, String)> = Vec::new();
        // println!("{:?}", self.state.context.registry.types.clone());
        for (_type, _value) in self.state.context.registry.types.as_object().unwrap() {
            for (_baseslot_name, _slot) in base_slots_index {
                if _type.contains(_baseslot_name) {
                    base_slots.push((_type.clone(), _value.clone(), _slot.clone()));
                }
            }
        }


        // Create Member objects from base_slots and storage_layout
        let mut i = 9999999999; // to avoid astId conflict
        let mut initial_members = HashMap::new();
        let mut absolute_slots = HashMap::new();
        for (_type, slot_info, _slot) in base_slots {
            let label = slot_info["label"].as_str().unwrap();
            let fulltype = _type;
            let type_kind = TypeKind::NaiveStruct;

            let member = Executable::new(
                i, // astId
                label.to_string(), // label of the current node
                String::from(fulltype.clone()), // fulltype
                None, // Pass self as the belongs_to parameter
                type_kind,
                fulltype.clone(),
                0, // Add the offset parameter
                0.to_string(),
                None, // Add the mapping_key parameter
                None, // Initialize iter as None, it will be populated later if needed
            );
            initial_members.insert(i, member.clone());
            absolute_slots.insert(i, _slot.clone());
            

            i -= 1;
        }
        self.state.context.registry.bulk_set_absolute_slots(&absolute_slots); // Note: use it for knowing parent slot
        self.state.context.registry.bulk_enqueue_children_execution(0, &initial_members); // Note: use it for knowing parent slot

    }
    pub async fn listen(&mut self) {
        self.scan_contract().await;
    }


    pub async fn scan_contract(&mut self) {
        while self.state.step <= 15 {
            match Executor::bulk_exec_and_reload(self.state.step, self.state.context.clone()).await {
                Ok(()) => (),
                Err(err) => {
                    println!("Error reloading context: {}", err);
                    break;
                }
            };

            self.state.step += 1;
        }
    }
  
}


