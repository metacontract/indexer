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
use super::ast_node::Node;


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;


#[derive(Clone)]
pub struct Executable<'executable_lifetime> {
    name: String,
    step: usize,
    belongs_to: Option<&'executable_lifetime Executable<'executable_lifetime>>,
    type_kind: TypeKind,
    value_type: String,
    offset: usize,
    relative_slot: String,
    absolute_slot: Option<String>,
    value: Option<String>,
    children: Option<Vec<&'executable_lifetime Executable<'executable_lifetime>>>,
    ast_node: Option<&'static Node>,
    mapping_key: Option<String>,
    iter: Option<IteratorMeta>,
}


trait ExecutableT {
    pub fn new(
        name: String,
        step: usize,
        belongs_to: Option<&Executable>,
        type_kind: TypeKind,
        value_type: String,
        offset: usize,
        relative_slot: String,
        ast_node: Option<&'static Node>,
        mapping_key: Option<String>,
        iter: Option<IteratorMeta>,
    ) -> Self {
        Self {
            name,
            step,
            belongs_to: Some(belongs_to),
            type_kind,
            value_type,
            offset: offset,
            relative_slot,
            None,
            None,
            None,
            ast_node,
            mapping_key,
            iter,
        }
    }

    pub fn is_iterish(&self) -> bool {
        self.type_kind.is_iterish();
    }


    pub fn get_edfs(&self) -> String {
    }

    pub fn get_type_and_name(&self) -> String {
        // Implement the logic to get the type and name for a Member
        // ...
    }

    pub fn get_type_kind(&self) -> TypeKind {
        self.type_kind.clone()
    }

    pub fn get_iter(&self) -> Option<&IteratorMeta> {
        self.iter.as_ref()
    }

    pub fn get_iter_mut(&mut self) -> Option<&mut IteratorMeta> {
        self.iter.as_mut()
    }

    pub fn get_abs_slot(&self) -> Option<String> {
        self.absolute_slot.clone()
    }

    pub fn get_value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn increment_step(&mut self) {
        self.step += 1;
    }

    pub fn get_belongs_to(&self) -> Option<&Executable> {
        self.belongs_to.as_ref()
    }

    pub fn enqueue_execution(&self) {
        self.registry.queue_per_step.insert(self.step, self);
    }

    pub fn get_children(&self) -> Option<Vec<&Executable>> {
        if self.is_iterish() && self.iter.as_ref().map(|i| i.to).is_some() {
            let mut children = Vec::new();
            if let Some(iter) = &self.iter {
                for i in iter.from..iter.to.unwrap() {
                    // Find the corresponding struct and its members in the registry.ast_node
                    let ast_node = self.registry.ast_node.find_struct_by_name(format!("{}.{}", self.name, i));
                    if let Some(ast_node) = ast_node {
                        for member in ast_node.members.iter() {
                            let member_name = format!("{}.{}", self.name, member.name);
                            let member_type_kind = match member.type_kind.as_str() {
                                "t_mapping" => TypeKind::Mapping,
                                "t_array" => TypeKind::Array,
                                "t_struct" => TypeKind::NaiveStruct,
                                _ => TypeKind::Primitive,
                            };
                            let member_value_type = member.value_type.clone();
                            let member_relative_slot = member.relative_slot.clone();

                            let item = Executable::new(
                                member_name,
                                member_type_kind,
                                member_value_type,
                                member_relative_slot,
                                self.clone_box(),
                                if member_type_kind.is_iterish() {
                                    Some(IteratorMeta::new(
                                        None, // key_type
                                        None, // perf_config
                                        Vec::new(), // items
                                        0, // from
                                        0, // to
                                    ))
                                } else {
                                    None
                                },
                            );
                            item.increment_step();
                            item.calculate_abs_slot();
                            self.children.push(&item);
                        }
                    }

                    self.increment_step();
                    self.calculate_abs_slot();
                    self.calculate_abs_slot();
                    children.push(self);
                }
            }
            Some(children)
        } else if self.type_kind == TypeKind::NaiveStruct {
            let mut children = Vec::new();
            if let Some(ast_node) = self.ast_node.as_ref() {
                for member in ast_node.members.iter() {
                    let member_name = format!("{}.{}", self.name, member.name);
                    let member_type_kind = match member.type_kind.as_str() {
                        "t_mapping" => TypeKind::Mapping,
                        "t_array" => TypeKind::Array,
                        "t_struct" => TypeKind::NaiveStruct,
                        _ => TypeKind::Primitive,
                    };
                    let member_value_type = member.value_type.clone();
                    let member_relative_slot = member.relative_slot.clone();

                    let member = Executable::new(
                        member_name,
                        member_type_kind,
                        member_value_type,
                        member_relative_slot,
                        self.clone_box(),
                        if member_type_kind.is_iterish() {
                            Some(IteratorMeta::new(
                                None, // key_type
                                None, // perf_config
                                Vec::new(), // items
                                0, // from
                                0, // to
                            ))
                        } else {
                            None
                        },

                    );
                    member.increment_step();
                    member.calculate_abs_slot();
                    children.push(&member);
                }
            }
            Some(children)
        } else {
            None
        }
    }
    pub fn set_value(&mut self, value: Option<&PerfConfigItem>) {
        match self.type_kind {
            TypeKind::Primitive => {
                if let Some(value) = value {
                    self.value = Some(value.to_string().as_str().to_string());
                } else {
                    self.value = None;
                }
            },
            TypeKind::Array => {
                // Set the value for an array
                if let Some(value) = value {
                    if let Some(iter) = &mut self.iter {
                        iter.to = value.to.as_ref().map(|s| s.parse().unwrap_or(0)).unwrap_or(0);
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
        if let Some(belongs_to) = &self.belongs_to {
            if let Some(abs_slot) = belongs_to.get_abs_slot() {
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

impl ExecutableT for Executable {}
