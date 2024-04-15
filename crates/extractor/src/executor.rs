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


use std::result::Result;
use std::result::Result::{Ok, Err};
use std::error::Error;
use std::future::Future;
use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;


pub struct Executor<'executor> {
    pub queue_per_step: Vec<Vec<&'executor Executable<'executor>>>,
    pub executed_per_step: Vec<Vec<&'executor mut Executable<'executor>>>,
    registry: &'executor mut Registry<'executor>,
}

impl Executor<'_> {
    pub fn new(registry: &mut Registry) -> Self {
        Self {
            queue_per_step: Vec::new(),
            executed_per_step: Vec::new(),
            registry,
        }
    }

    pub async fn bulk_exec_and_enqueue_and_set_primitive_to_output(&mut self, step: usize) -> Result<(), Box<dyn Error>> {
        // Get values by slots from EthCall
        let slots: HashMap<usize, String> = self.queue_per_step[step].iter().map(|executable| (executable.id, executable.absolute_slot.as_ref().map(|s| s.as_str()).unwrap_or_default().to_owned())).collect();
        let values = EthCall::get_values_by_slots(&slots, "mainnet", "0x1234567890123456789012345678901234567890", "0x1234567890123456789012345678901234567890").await?;

        // Flush the queue for the current step
        self.flush_queue(step);

        // Process each executed executable
        for executed in &mut self.executed_per_step[step] {
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
                self.registry.set_output(executed.id, &executed);
            } else if executed.is_iterish() {
                if 
                    executed.fill_iter_unless_empty_index()
                    ||
                    !executed.is_iterish()
                {
                    executed.enqueue_children();
                } else {
                    // Note: Un-filled iterish only goes here. Skip.
                }
            }
        }

        // Flush the executed executables for the current step
        self.flush_executed(step);
        Ok(())
    }

    fn flush_queue(&mut self, step: usize) {
        self.queue_per_step[step].clear();
    }

    fn flush_executed(&mut self, step: usize) {
        self.executed_per_step[step].clear();
    }
}