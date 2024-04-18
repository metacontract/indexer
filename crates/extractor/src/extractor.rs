use super::compiler::Compiler;
use super::executor::Executor;
use super::registry::Registry;
use super::executable::Executable;
use super::perf_config_item::PerfConfigItem;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
use super::iterator_meta::IteratorMeta;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
use super::ast_node::ASTNode;
use super::context::Context;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem;

pub struct Extractor<'a> {
    initial_members: Vec<Executable<'a>>,
    state: ExtractorState<'a>,
}

struct ExtractorState<'a> {
    step: usize,
    context: Context<'a>,
}

impl<'a> Extractor<'a> {
    pub fn new(context: Context<'a>) -> Self {
        Self {
            initial_members: Vec::new(),
            state: ExtractorState {
                step: 0,
                context,
            },
        }
    }

    pub fn init_members_from_compiler(&mut self, base_slots: &Value) {

        // Create Member objects from base_slots and storage_layout
        let mut i = 9999999999; // to avoid astId conflict
        for (name, slot_info) in base_slots.as_object().unwrap() {
            let fulltype = slot_info["type"].as_str().unwrap();
            let type_kind = match fulltype {
                "t_mapping" => TypeKind::Mapping,
                "t_array" => TypeKind::Array,
                "t_struct" => TypeKind::NaiveStruct,
                _ => TypeKind::Primitive,
            };

            let value_type = slot_info["valueType"].as_str().unwrap().to_string();
            let relative_slot = slot_info["slot"].as_str().unwrap().to_string();

            let member = Executable::new(
                i, // astId
                name.to_string(), // label of the current node
                String::from(fulltype), // fulltype
                None, // Pass self as the belongs_to parameter
                type_kind,
                value_type,
                0, // Add the offset parameter
                relative_slot,
                None, // Add the mapping_key parameter
                None, // Initialize iter as None, it will be populated later if needed
            );
            self.initial_members.push(member.clone());
            i -= 1;
        }
    }
    pub async fn listen(&'a mut self) {
        self.scan_contract();
    }


    pub async fn scan_contract(&mut self) {
        while self.state.step <= 15 {
            let context = mem::replace(&mut self.state.context, Context::dummy());
            let new_context = match Executor::bulk_exec_and_reload(self.state.step, context.clone()).await {
                Ok(context) => context,
                Err(err) => {
                    println!("Error reloading context: {}", err);
                    break;
                }
            };
            mem::replace(&mut self.state.context, new_context);

            self.state.step += 1;
        }
    }
  
}


