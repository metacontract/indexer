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


#[allow(dead_code)]
#[derive(Clone)]
pub struct Executable {
    pub id: usize,
    pub name: String,
    pub fulltype: String,
    pub belongs_to: Option<Box<Executable>>,
    pub type_kind: TypeKind,
    pub value_type: String,
    offset: usize,
    relative_slot: String,
    mapping_key: Option<String>,
    pub key_type: Option<String>,
}


impl Executable {
    pub fn new(
        id: usize,
        name: String,
        fulltype: String,
        belongs_to: Option<Box<Executable>>,
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

    pub fn member_fulltypes(&self, from_to:Option<&(usize, usize)>, registry: &Registry) -> Vec<String> {
        let to = match from_to {
            Some((_, to)) => to.clone(),
            None => 0
        };

        let current_node = &registry.visit_ast(&self.fulltype).unwrap();

        if self.is_iterish() && to > 0 {
            // This executable is iterable member
            let value_types = (0..to).map(|_| {
                current_node.get("type").unwrap().to_string()
            }).collect();
            value_types
        } else {
            // Check if the type is a struct
            if self.type_kind == TypeKind::NaiveStruct {
                // Return all member_fulltypes (struct fullname) of the members
                current_node.get("members").unwrap().as_array().unwrap().iter().map(|member| member.as_object().unwrap().get("type").unwrap().as_str().unwrap().to_string()).collect()
            } else {
                // Primitive type, throw error
                panic!("Primitive type, cannot list member_fulltypes");
            }
        }
    }
    pub fn children(&self, registry: &Registry, from_to: Option<&(usize, usize)>) -> Vec<Executable> {
        let mut children = Vec::new();
        let member_fulltypes = self.member_fulltypes(from_to, registry);
        for i in 0..member_fulltypes.len() {
            let current_node = registry.visit_ast(&member_fulltypes[i]).unwrap();
            println!("{:?}", current_node.clone());
            println!("{:?}", member_fulltypes[i].clone());
            // let fulltype = current_node.get("type").unwrap().to_string();
            let fulltype = member_fulltypes[i].clone();
            let parsed_type = ASTNode::parse_type_str(&fulltype.clone());
            let new_executable = Executable::new(
                current_node.get("astId").unwrap().as_u64().unwrap() as usize, // astId
                current_node.get("label").unwrap().to_string(), // member_fulltype of the current node
                fulltype.clone(), // fulltype
                self.belongs_to.clone(), // set the belongs_to to the current executable
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
        children
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
        match &self.belongs_to {
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
