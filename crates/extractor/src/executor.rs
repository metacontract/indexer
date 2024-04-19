use super::compiler::Compiler;
use super::extractor::Extractor;
// use super::executor::Executor;
use super::registry::Registry;
use super::executable::Executable;
use super::perf_config_item::PerfConfigItem;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
use super::iterator_meta::IteratorMeta;
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


pub struct Executor;

impl Executor {

    #[allow(unused_mut)]
    pub async fn bulk_exec_and_reload<'b>(step: usize, mut context: Context<'_>) -> Result<Context, Box<dyn Error>> {
        #[allow(unused_mut)]
        let registry_clone1 = context.registry.clone();
        let mut registry = context.registry;

        // [exec]
        // Get values by slots from EthCall
        let values = EthCall::get_values_by_slots(&registry.absolute_slots, "mainnet", "0x1234567890123456789012345678901234567890", "0x1234567890123456789012345678901234567890").await?;

        // Flush the queue for the current step
        registry.flush_queue(step);

        // [reload]
        // Process each executed executable
        let mut absolute_slots: HashMap<usize, String> = HashMap::new();
        let mut primitives: HashMap<usize, Executable> = HashMap::new();
        let mut pending_fillable_iterish: HashMap<usize, Executable> = HashMap::new();
        let mut filled_queueable_iterish: HashMap<usize, Executable> = HashMap::new();
        let executeds = registry.executed_per_step[step].clone();


        for executed in executeds {
            let executed_clone = executed.clone();
            absolute_slots.insert(executed.id, executed_clone.calculate_absolute_slot(&registry_clone1));

            registry = registry.bulk_save_values(values.clone());

            /******************
                Enqueue next activatable executables
            *******************/
            if executed.type_kind == TypeKind::Primitive {
                // If the executable is a primitive, push it to the output
                primitives.insert(executed.id, executed.clone());
            } else if executed.is_iterish() {
                let registry_clone2 = registry.clone(); // read only
                if executed.is_iter_readied(&registry_clone2) {
                    filled_queueable_iterish.insert(executed.id, executed.clone());
                } else {
                    pending_fillable_iterish.insert(executed.id, executed.clone());
                }
            }
        }
        registry = registry.bulk_set_absolute_slots(&absolute_slots);
        registry = registry.set_primitives(&primitives);
        registry = registry.bulk_fill_from_to(&pending_fillable_iterish);
        registry = registry.bulk_enqueue_execution(step+1, pending_fillable_iterish.clone());
        registry = registry.bulk_enqueue_children_execution(step+1, filled_queueable_iterish.clone());
        // Flush the executed executables for the current step
        registry.flush_executed(step);

        // Update the context with the modified registry
        context.registry = registry.clone();

        Ok(context)
    }
}