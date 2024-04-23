use super::compiler::Compiler;
use super::extractor::Extractor;
use super::executor::Executor;
use super::registry::Registry;
// use super::executable::Executable;
use super::perf_config_item::PerfConfigItem;
use super::type_kind::TypeKind;
use super::eth_call::EthCall;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
use super::ast_node::ASTNode;
use super::context::Context;


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::rc::Rc;
use std::cell::RefCell;
use ethers::utils::keccak256;
use ethers::utils::hex;


#[derive(Clone)]
pub struct Executable<'a> {
    pub id: usize,
    pub name: String,
    pub fulltype: String,
    pub belongs_to: Option<&'a Executable<'a>>, // to avoid recursive type
    pub type_kind: TypeKind,
    pub value_type: String,
    offset: usize,
    relative_slot: String,
    mapping_key: Option<String>,
    pub key_type: Option<String>,
}


impl<'a> Executable<'a> {
    pub fn new(
        id: usize,
        name: String,
        fulltype: String,
        belongs_to: Option<&'a Executable>,
        type_kind: TypeKind,
        value_type: String,
        offset: usize,
        relative_slot: String,
        mapping_key: Option<String>,
        key_type: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            fulltype,
            belongs_to,
            type_kind,
            value_type,
            offset,
            relative_slot,
            mapping_key,
            key_type,
        }
    }

    pub fn is_iterish(&self) -> bool {
        self.type_kind.is_iterish()
    }

    pub fn labels(&self, to:usize, registry: &Registry) -> Vec<String> {
        let current_node = &registry.visitAST(&self.fulltype).unwrap();

        if self.is_iterish() && to > 0 {
            // This executable is iterable member
            let value_types = (0..to).map(|_| {
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
    pub fn children(&'a self, to: usize, registry: &'a mut Registry<'a>) -> (&'a mut Registry<'a>, Vec<Executable<'a>>) {
        let mut children = Vec::new();
        let labels = self.labels(to, registry);
        for i in 0..labels.len() {
            let current_node = registry.visitAST(&labels[i]).unwrap();
            let fulltype = current_node.get("type").unwrap().to_string();
            let parsed_type = ASTNode::parse_type_str(&fulltype.clone());
            let new_executable = Executable::new(
                current_node.get("astId").unwrap().as_u64().unwrap() as usize, // astId
                current_node.get("label").unwrap().to_string(), // label of the current node
                fulltype.clone(), // fulltype
                Some(&self), // set the belongs_to to the current executable
                ASTNode::type_kind(&fulltype.clone()), // type kind of the current node
                parsed_type.value_type, // type of the current node
                current_node.get("offset").unwrap().as_u64().unwrap() as usize, // offset of the current node
                current_node.get("slot").unwrap().to_string(), // slot of the current node
                if self.is_iterish() { // check iter or not
                    Some(i.to_string()) // mapping key
                } else {
                    None // depends on is_iterish
                },
                parsed_type.key_type,
            );
            children.push(new_executable);
        };
        (&mut registry, children)
    }
 

    pub fn is_iter_readied(&self, registry: &Registry) -> bool {
        let (_, to) = match registry.iterish_from_to.get(&self.id) {
            Some((from, to)) => (*from, *to),
            None => {
                panic!("No from/to values found for executable with ID: {}", self.id);
            }
        };

        // If the iterator's `to` field is empty (likely a mapping)
        if to == 0 {
            false
        } else {
            true
        }
    }
    
    pub fn calculate_absolute_slot(&self, registry: &Registry) -> String {
        match self.belongs_to {
            Some(belongs_to) => {
                match registry.absolute_slots.get(&belongs_to.id) {
                    Some(belongs_to_absolute_slot) => {
                        let combined_slot = if belongs_to.is_iterish() {
                            match &self.mapping_key {
                                Some(mapping_key) => {
                                    let iterable_absolute_slot = format!("{}{}", mapping_key, belongs_to_absolute_slot.trim_start_matches("0x"));
                                    hex::encode(ethers::utils::keccak256(iterable_absolute_slot.as_bytes()))
                                },
                                None => {
                                    panic!("No absolute_slot: {}", belongs_to.id);
                                }            
                            }
                        } else {
                            let abs_slot_num = belongs_to_absolute_slot.parse::<usize>().unwrap();
                            let relative_slot_num = self.relative_slot.parse::<usize>().unwrap();
                            format!("0x{:x}", abs_slot_num + relative_slot_num)
                        };
                        combined_slot
                    },
                    None => {
                        panic!("No absolute_slot: {}", belongs_to.id);
                    }            
                }
            },
            None => {
                panic!("No belongs_to: {}", self.id);
            }
        }

    }

}
