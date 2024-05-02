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
use std::error::Error;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};


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
    pub fn children(&self, registry: &Registry, from_to: Option<&(usize, usize)>) -> Result<Vec<Executable>, Box<dyn Error>> {
        let mut children = Vec::new();

        let current_node = &registry.visit_ast(&self.fulltype).unwrap();

        let to = match from_to {
            Some((_, to)) => to.clone(),
            None => 0
        };

        match current_node.get("members") {
            Some(_members) => {
                for _member in _members.as_array().unwrap() {
                    let fulltype = _member.get("type").unwrap().to_string();
                    let _ast_id = u64::from_le_bytes(keccak256(&format!("{}{}", fulltype, "").as_bytes())[..8].try_into().unwrap()) as usize;

                    let new_executable = Executable::new(
                        _ast_id,
                        _member.get("label").unwrap().to_string(), // member_fulltype of the current node
                        fulltype.clone(), // fulltype
                        Some(Box::new(self.clone())), // set the belongs_to to the current executable
                        ASTNode::type_kind(&fulltype.clone()), // type kind of the current node
                        fulltype.clone(), // type of the current node
                        _member.get("offset").unwrap().as_u64().unwrap() as usize, // offset of the current node
                        _member.get("slot").unwrap().to_string(), // slot of the current node
                        None,
                        None,
                    );
                    children.push(new_executable);
                }
                Ok(children)
            },
            None => {
                if self.is_iterish() && to > 0 {
                    for i in 0..to {
                        let key_type = current_node.get("key_type").unwrap().to_string();
                        let value_type = current_node.get("value_type").unwrap().to_string();
                        let _ast_id = u64::from_le_bytes(keccak256(&format!("{}{}", value_type, i).as_bytes())[..8].try_into().unwrap()) as usize;
                        let current_node = registry.visit_ast(&value_type).unwrap();

                        let new_executable = Executable::new(
                            _ast_id,
                            current_node.get("label").unwrap().to_string(), // member_fulltype of the current node
                            value_type.clone(), // fulltype
                            Some(Box::new(self.clone())), // set the belongs_to to the current executable
                            ASTNode::type_kind(&value_type.clone()), // type kind of the current node
                            value_type.clone(), // type of the current node
                            current_node.get("offset").unwrap().as_u64().unwrap() as usize, // offset of the current node
                            current_node.get("slot").unwrap().to_string(), // slot of the current node
                            Some(i.to_string()),
                            Some(key_type.clone()),
                        );
                        children.push(new_executable);
                    }
                    Ok(children)
                } else {
                    // primitive doesn't have children
                    Ok(vec!())
                }
            }
        }
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
                            match Executable::add_usize_to_32bytes(&belongs_to_absolute_slot.clone().trim_start_matches("0x"), &self.relative_slot.clone()) {
                                Ok(abs_slot) => abs_slot,
                                Err(err) => panic!("{}",err),
                            }
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

    fn add_usize_to_32bytes(value: &str, number: &str) -> Result<String, String> {
        if value.len() != 64 {
            return Err(format!("Invalid value length. Expected 64 characters, got {}", value.len()));
        }
    
        let value_bytes = hex::decode(value)
            .map_err(|e| format!("Failed to decode value: {}", e))?;
    
        if value_bytes.len() != 32 {
            return Err(format!("Invalid decoded value length. Expected 32 bytes, got {}", value_bytes.len()));
        }
    
        let mut value_array = [0u8; 32];
        value_array.copy_from_slice(&value_bytes);

        let number_uint = number.trim_start_matches('"').trim_end_matches('"').parse::<usize>()
            .map_err(|e| format!("Failed to parse number: {}", e))?;
        let value_uint = BigUint::from_bytes_le(&value_array);
        let number_uint = BigUint::from(number_uint);
        let result_uint = value_uint + number_uint;
    
        let result_bytes = result_uint.to_bytes_le();
        let mut result_array = [0u8; 32];
        result_array[..result_bytes.len()].copy_from_slice(&result_bytes);
    
        let result_hex = hex::encode(result_array);
    
        Ok(result_hex)
    }

}
