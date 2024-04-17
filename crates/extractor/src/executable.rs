use super::compiler::Compiler;
use super::extractor::Extractor;
use super::executor::Executor;
use super::registry::Registry;
// use super::executable::Executable;
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


#[derive(Clone)]
pub struct Executable<'a> {
    pub id: usize,
    pub name: String,
    pub fulltype: String,
    pub step: usize,
    pub belongs_to: Option<&'a Executable<'a>>,
    pub type_kind: TypeKind,
    pub value_type: String,
    offset: usize,
    relative_slot: String,
    pub absolute_slot: Option<String>,
    pub value: Option<String>,
    mapping_key: Option<String>,
    pub iter: Option<IteratorMeta>,
}


impl<'a> Executable<'a> {
    pub fn new(
        id: usize,
        name: String,
        fulltype: String,
        step: usize,
        belongs_to: Option<&Executable>,
        type_kind: TypeKind,
        value_type: String,
        offset: usize,
        relative_slot: String,
        mapping_key: Option<String>,
        iter: Option<IteratorMeta>,
    ) -> Self {
        Self {
            id,
            name,
            fulltype,
            step,
            belongs_to,
            type_kind,
            value_type,
            offset,
            relative_slot,
            absolute_slot: None, // absolute_slot
            value: None, // value
            mapping_key,
            iter,
        }
    }

    pub fn is_iterish(&self) -> bool {
        self.type_kind.is_iterish()
    }

    pub fn increment_step(&mut self) {
        self.step += 1;
    }

    pub fn labels(&self, context: &Context) -> Vec<String> {
        let current_node = context.ast_node.visit(&self.fulltype).unwrap();
        let iter = self.iter.unwrap();
        let to = iter.to.unwrap();

        if self.is_iterish() && to > 0 {
            // This executable is iterable member
            let value_types = (0..to).map(|i| {
                current_node.get("type").unwrap().to_string()
            }).collect();
            value_types
        } else {
            // Check if the type is a struct
            if self.type_kind == TypeKind::NaiveStruct {
                // Return all labels (type names) of the members
                current_node.get("members").unwrap().as_array().unwrap().iter().map(|member| member.as_object().unwrap().get("label").unwrap().as_str().unwrap().to_string()).collect()
            } else {
                // Primitive type, throw error
                panic!("Primitive type, cannot list labels");
            }
        }
    }
    pub fn children(&self, context: &Context) -> Vec<Executable> {
        let mut children = Vec::new();
        let labels = self.labels(&context);
        for i in 0..labels.len() {
            let current_node = context.ast_node.visit(&labels[i]).unwrap();
            let fulltype = current_node.get("type").unwrap().to_string();
            let parsed_type = ASTNode::parse_type_str(&fulltype);
            let new_executable = Executable::new(
                current_node.get("astId").unwrap().as_u64().unwrap() as usize, // astId
                current_node.get("label").unwrap().to_string(), // label of the current node
                fulltype, // fulltype
                self.step + 1, // step of the current node
                Some(&self), // set the belongs_to to the current executable
                ASTNode::type_kind(&fulltype), // type kind of the current node
                parsed_type.value_type, // type of the current node
                current_node.get("offset").unwrap().as_u64().unwrap() as usize, // offset of the current node
                current_node.get("slot").unwrap().to_string(), // slot of the current node
                if self.is_iterish() { // check iter or not
                    Some(i.to_string()) // mapping key
                } else {
                    None // depends on is_iterish
                },
                if ASTNode::type_kind(&fulltype).is_iterish() {
                    Some(IteratorMeta {
                        key_type: parsed_type.key_type,
                        from: None,
                        to: None,
                    })
                } else {
                    None
                },
            );
            children.push(new_executable);
        }
        children
    }

    pub fn enqueue_execution(&self, context: &Context) {
        context.registry.queue_per_step.insert(self.step, vec![self]);
    }
    pub fn enqueue_children(&self, context: &Context) -> () {
        let children = self.children(&context);
        for child in children {
            child.enqueue_execution(&context);
        }
    }

    pub fn fill_iter_unless_empty_index(&self, context: &Context) -> bool {
        // If the iterator's `to` field is empty (likely a mapping)
        let perf_config_item = context.registry.get_perf_config_item(self.id);

        let from_expression = perf_config_item.as_ref().and_then(|item| item.from.clone());
        let to_expression = perf_config_item.as_ref().and_then(|item| item.to.clone());

        let parsed_from = if let Some(from_expr) = from_expression {
            PerfExpressionEvaluator::eval(from_expr, &mut context)
        } else {
            0
        };
        self.iter.unwrap().set_from(parsed_from);

        let parsed_to = if let Some(to_expr) = to_expression {
            PerfExpressionEvaluator::eval(to_expr, &mut context)
        } else {
            0
        };
        self.iter.unwrap().set_to(parsed_to);

        if parsed_to == 0 {
            self.increment_step();
            self.enqueue_execution(&context);
            false
        } else {
            true
        }
    }

    pub fn set_value(&mut self, value: String) {
        match self.type_kind {
            TypeKind::Primitive => {
                self.value = Some(value);
            },
            TypeKind::Array => {
                // Set the value for an array
                if let Some(iter) = &mut self.iter {
                    if let Ok(value_as_u64) = value.parse::<u64>() {
                        iter.to = Some(value_as_u64 as usize);
                    } else {
                        panic!("Unable to parse value as u64: {}", value);
                    }
                }
            },
            TypeKind::Mapping | TypeKind::NaiveStruct => {
                // Skip setting the value for mappings and naive structs
            }
        }
    }

    pub fn calculate_abs_slot(&mut self) -> () {
        // iter must have belongs_to and so logic can be shorter
        if let Some(belongs_to) = self.belongs_to {
            if let Some(abs_slot) = belongs_to.absolute_slot {
                let abs_slot_num = abs_slot.parse::<usize>().unwrap();
                let relative_slot_num = self.relative_slot.parse::<usize>().unwrap();
                let combined_slot = abs_slot_num + relative_slot_num;
                self.absolute_slot = Some(format!("{:X}", combined_slot));
            } else {
                // Do nothing, as the error message suggests the function should return `()`
            }
        } else {
            // Do nothing, as the error message suggests the function should return `()`
        }
    }

}
