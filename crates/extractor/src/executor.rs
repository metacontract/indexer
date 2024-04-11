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
use super::ast_node::Node;


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;


pub struct Executor {
    queue_per_step: Vec<Vec<&'static Executable>>,
    executed_per_step: Vec<Vec<&'static Executable>>,
    registry: Registry,
}

impl Executor {
    pub fn new(registry: Registry) -> Self {
        Self {
            queue_per_step: Vec::new(),
            executed_per_step: Vec::new(),
            registry,
        }
    }

    pub fn bulk_exec_and_enqueue_and_set_primitive_to_output(&mut self, step: usize) {

        // Get values by slots from EthCall
        let slots: HashMap<String, String> = self.queue_per_step[step].iter().map(|executable| (executable.get_edfs(), executable.get_abs_slot().unwrap_or_default())).collect();
        let values = EthCall::get_values_by_slots(&slots, "mainnet", "0x1234567890123456789012345678901234567890", "0x1234567890123456789012345678901234567890");

        // Flush the queue for the current step
        self.flush_queue(step);

        // Process each executed executable
        for executable in &mut self.executed_per_step[step] {
            // Get the performance configuration item for the executable
            let perf_config_item = self.registry.get_perf_config_item(executable.get_edfs());

            // Set the value for the executable based on the performance configuration item
            executable.set_value(values[executable.get_edfs()]);

            if executable.get_type_kind() == TypeKind::Primitive {
                // If the executable is a primitive, push it to the output
                self.registry.set_output(executable.get_edfs(), executable.clone());
            } else {
                // If the executable is a non-primitive (NaiveStruct, Array, Mapping)

                // Check if the executable has an iterator
                if let Some(iter) = &mut executable.get_iter_mut() {
                    if !iter.to.is_empty() {
                        // If the iterator has a non-empty `to` field
                        for i in 0..iter.to {
                            // Create new executables for each item in the iterator
                            let new_executable = executable.clone_box();
                            new_executable.enqueue_execution();
                        }
                    } else {
                        // If the iterator's `to` field is empty (likely a mapping)
                        let from_expression = perf_config_item.as_ref().and_then(|item| item.from.clone());
                        let to_expression = perf_config_item.as_ref().and_then(|item| item.to.clone());

                        if let Some(from_expr) = from_expression {
                            let parsed_from = PerfExpressionEvaluator::eval(from_expr);
                            iter.set_from(parsed_from);
                        }

                        if let Some(to_expr) = to_expression {
                            let parsed_to = PerfExpressionEvaluator::eval(to_expr);
                            iter.set_to(parsed_to);
                        }

                        // Skipping algorithm for a mapping's unloaded bin_index
                        if iter.to == 0 {
                            executable.increment_step();
                            executable.enqueue_execution();
                        } else {
                            let children = executable.get_children();
                            for child in children {
                                child.enqueue_execution();
                            }
                        }

                    }
                } else {
                    // If the executable doesn't have an iterator
                    if executable.get_abs_slot().is_some() && executable.get_value().is_none() {
                        // If the executable has an absolute slot but no value, enqueue it for execution
                        executable.enqueue_execution();
                    }
                }
            }
        }

        // Flush the executed executables for the current step
        self.flush_executed(step);
    }

    fn flush_queue(&mut self, step: usize) {
        self.queue_per_step[step].clear();
    }

    fn flush_executed(&mut self, step: usize) {
        self.executed_per_step[step].clear();
    }
}