use super::compiler::Compiler;
// use super::extractor::Extractor;
use super::executor::Executor;
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

pub struct Extractor<'extractor_lifetime> {
    compiler: Compiler,
    registry: Registry<'extractor_lifetime>,
    initial_members: Vec<&'extractor_lifetime Executable<'extractor_lifetime>>,
    executor: Executor,
}

impl Extractor<'_> {
    pub fn new() -> Self {
        let compiler = Compiler::new("solc".to_string());
        let storage_layout = compiler.prepare_storage_layout().unwrap();
        let registry = Registry::new(HashMap::new(), storage_layout);
        let executor = Executor::new(registry.clone());

        Self {
            compiler,
            registry,
            initial_members: Vec::new(),
            executor
        }
    }

    pub fn init_members_from_compiler(&mut self) {
        let base_slots = self.compiler.prepare_base_slots().unwrap();

        // Create Member objects from base_slots and storage_layout
        for (name, slot_info) in base_slots.as_object().unwrap() {
            let type_kind = match slot_info["type"].as_str().unwrap() {
                "t_mapping" => TypeKind::Mapping,
                "t_array" => TypeKind::Array,
                "t_struct" => TypeKind::NaiveStruct,
                _ => TypeKind::Primitive,
            };

            let value_type = slot_info["valueType"].as_str().unwrap().to_string();
            let relative_slot = slot_info["slot"].as_str().unwrap().to_string();

            let member = Executable::new(
                name.to_string(),
                0, // Add the step parameter for base slot
                None, // Pass self as the belongs_to parameter
                type_kind,
                value_type,
                0, // Add the offset parameter
                relative_slot,
                None, // Add the ast_node parameter
                None, // Add the mapping_key parameter
                None, // Initialize iter as None, it will be populated later if needed
            );
            self.initial_members.push(&member);
        }
    }

    pub fn listen(&mut self) {
        // Listen for events and process them
        // ...

        self.scan_contract();
    }

    pub fn scan_contract(&mut self) {
        let mut step = 0;
        loop {
            self.executor.bulk_exec_and_enqueue_and_set_primitive_to_output(step);
            step += 1;

            if step > 15 {
                break;
            }
        }
    }
}
