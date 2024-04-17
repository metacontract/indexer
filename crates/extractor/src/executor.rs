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

    pub async fn bulk_exec_and_reload(step: usize, mut context: Context<'_>) -> Result<Context<'_>, Box<dyn Error>> {
        let registry = &mut context.registry;

        // [exec]
        // Get values by slots from EthCall
        let queue = &registry.queue_per_step;
        let slots: HashMap<usize, String> = queue[step].iter().map(|executable| (executable.id, executable.absolute_slot.as_ref().map(|s| s.as_str()).unwrap_or_default().to_owned())).collect();
        let values = EthCall::get_values_by_slots(&slots, "mainnet", "0x1234567890123456789012345678901234567890", "0x1234567890123456789012345678901234567890").await?;

        // Flush the queue for the current step
        registry.flush_queue(step);

        // [reload]
        // Process each executed executable
        let mut primitives: HashMap<usize, Executable> = HashMap::new();
        let executeds = &mut registry.executed_per_step[step];
        for executed in executeds {
            // Set the value for the executable based on the performance configuration item
            if let Some(value) = values.get(&executed.id) {
                executed.set_value(value.clone());
            } else {
                // Handle the case where the value is not found in the `values` map
                // You may want to log an error or handle it in some other way
            }

            /******************
                Enqueue next activatable executables
            *******************/
            if executed.type_kind == TypeKind::Primitive {
                // If the executable is a primitive, push it to the output
                primitives[&executed.id] = executed;
            } else if executed.is_iterish() {
                if 
                    executed.fill_iter_unless_empty_index(&context)
                    ||
                    !executed.is_iterish()
                {
                    executed.enqueue_children(&context);
                } else {
                    // Note: Un-filled iterish only goes here. Skip.
                }
            }
        }
        registry.set_primitives(primitives);

        // Flush the executed executables for the current step
        registry.flush_executed(step);
        Ok(context)
    }
}