use super::compiler::Compiler;
use super::extractor::Extractor;
// use super::executor::Executor;
use super::registry::Registry;
use super::executable::Executable;
use super::config_util::ConfigUtil;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
use super::ast_node::ASTNode;
use super::context::Context;


use std::result::Result;
use std::result::Result::{Ok, Err};
use std::error::Error;
use std::future::Future;
use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::rc::Rc;
use std::cell::RefCell;
use std::env;


pub struct Executor;

impl Executor {

    #[allow(unused_mut)]
    pub async fn bulk_exec_and_reload(step: usize, registry: &mut Registry) -> Result<(), Box<dyn Error>> {

        let mut absolute_slots: HashMap<usize, String> = HashMap::new();
        let mut primitives: HashMap<usize, Executable> = HashMap::new();
        let mut pending_fillable_iterish: HashMap<usize, Executable> = HashMap::new();
        let mut filled_queueable_iterish: HashMap<usize, Executable> = HashMap::new();


        // [exec]
        // - get absolute_slot
        // - get value
        // - preserve them
        for e in registry.queue_per_step[step].clone() {
            // match e.belongs_to {
            //     Some(ref belongs_to) => {
            //         println!("parent: {:?}  e:{:?} {:?}", belongs_to.name, e.fulltype, e.name);
            //     },
            //     None => {
            //         println!("parent: ---  e:{:?} {:?}", e.fulltype, e.name);
            //     }
            // }
            absolute_slots.insert(e.id, e.clone().calculate_absolute_slot(&registry));
        }
        registry.bulk_set_absolute_slots(&absolute_slots); // Note: use it for knowing parent slot

        let _contract_addr = match env::var("CONTRACT_ADDR") {
            Ok(addr) => addr,
            Err(_) => panic!("{}", "CONTRACT_ADDR was not provided."),
        };
        let _contract_code = match env::var("CONTRACT_CODE") {
            Ok(code) => code,
            Err(_) => panic!("{}", "CONTRACT_CODE was not provided."),
        };
        let values = EthCall::get_values_by_slots(&absolute_slots, "mainnet", &_contract_addr, &_contract_code).await?;
        registry.bulk_save_values(values.clone());
        registry.bulk_save_visited(registry.queue_per_step[step].clone());


        // [reload]
        // - enqueue executables by each type for next step
        for e in registry.queue_per_step[step].clone() {
            if e.type_kind == TypeKind::Primitive {
                // If the executable is a primitive, push it to the output
                primitives.insert(e.id, e.clone());
            } else if e.is_iterish() {
                if e.is_iter_readied(&registry) {
                    filled_queueable_iterish.insert(e.id, e.clone());
                } else {
                    pending_fillable_iterish.insert(e.id, e.clone());
                }
            } else if e.type_kind == TypeKind::NaiveStruct {
                filled_queueable_iterish.insert(e.id, e.clone());
            }
        }

        registry
            .set_primitives(primitives.clone())
            .bulk_fill_from_to(&pending_fillable_iterish)
            .bulk_enqueue_execution(step+1, pending_fillable_iterish.clone())
            .bulk_enqueue_children_execution(step+1, &filled_queueable_iterish);

        Ok(())
    }
}