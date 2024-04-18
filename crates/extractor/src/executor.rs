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
        let mut registry = context.registry;
        let registry_clone = registry.clone(); // read only

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
        let mut pending_fillable_iterish: HashMap<usize, Executable> = HashMap::new();
        let mut filled_queueable_iterish: HashMap<usize, Executable> = HashMap::new();
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
                primitives.insert(executed.id, executed.clone());
            } else if executed.is_iterish() {
                if executed.is_iter_readied(&registry_clone) {
                    filled_queueable_iterish.insert(executed.id, executed.clone());
                } else {
                    pending_fillable_iterish.insert(executed.id, executed.clone());
                }
            }
        }
        let ast_node = &context.ast_node;
        registry = registry.set_primitives(&primitives);
        registry = registry.bulk_fill_from_to(&pending_fillable_iterish); // First pending_fillable_iterish usage is mut borrow
        registry = registry.bulk_enqueue_execution(step+1, &pending_fillable_iterish);// Second pending_fillable_iterish usage is move (clone inside)
        registry = registry.bulk_enqueue_children_execution(step+1, &filled_queueable_iterish, &ast_node);

        // Flush the executed executables for the current step
        registry.flush_executed(step);

        // Update the context with the modified registry
        context.registry = registry.clone();

        Ok(context)
    }
}