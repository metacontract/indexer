use super::compiler::Compiler;
use super::extractor::Extractor;
use super::executor::Executor;
use super::registry::Registry;
// use super::executable::Executable;
use super::config_util::ConfigUtil;
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
#[derive(Clone,Debug)]
pub struct Executable {
    pub id: usize,
    pub name: String,
    pub fulltype: String,
    pub belongs_to: Option<Box<Executable>>,
    pub type_kind: TypeKind,
    pub value_type: String,
    offset: usize,
    relative_slot: String,
    pub mapping_key: Option<String>,
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
    pub fn children(&self, registry: &Registry, indices: Option<Vec<String>>) -> Result<Vec<Executable>, Box<dyn Error>> {
        let mut children = Vec::new();

        let current_node = &registry.visit_ast(&self.fulltype).unwrap();


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
                if self.is_iterish() && indices.unwrap().len() > 0 {
                    for i in indices.unwrap() {
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
                            Some(i),
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
                return false;
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
                                    let hash_combined = hex::encode(ethers::utils::keccak256(iterable_absolute_slot.as_bytes()));
                                    hash_combined
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
        let value_uint = BigUint::from_bytes_be(&value_array);
        let number_uint = BigUint::from(number_uint);
        let result_uint = value_uint + number_uint;
        let result_bytes = result_uint.to_bytes_be();
        let mut result_array = [0u8; 32];
        result_array[32 - result_bytes.len()..].copy_from_slice(&result_bytes);
    
        let result_hex = hex::encode(result_array);

        if result_hex.len() != 64 {
            return Err(format!("Invalid result length. Expected 64 characters, got {}", result_hex.len()));
        }
    
        Ok(result_hex)
    }

    pub fn ancestors(&self) -> Vec<Executable> {
        let mut _ancestors: Vec<Executable> = Vec::new();

        if let Some(parent) = self.belongs_to.as_ref() {
            _ancestors.insert(0, *parent.clone());
            let mut current = parent;
            while let Some(grandparent) = current.belongs_to.as_ref() {
                _ancestors.insert(0, *grandparent.clone());
                current = grandparent;
            }
        }

        _ancestors
    }
    #[allow(dead_code)]
    pub fn fullname(&self) -> String {
        self.instance_paths().join(".")
    }
    pub fn fullname_in_conf(&self) -> String {
        self.paths_in_conf().join(".")
    }
    pub fn class_paths(&self) -> Vec<String> {
        let _ancestors = self.ancestors();
        let mut _paths = Vec::new();

        for e in _ancestors {
            if e.belongs_to.is_some() {
                if e.mapping_key.is_some() {
                    // skip
                } else {
                    _paths.push(e.name);
                }
            } else {
                let regex = regex::Regex::new(r"t_struct\((\w+)\)\d{3}_storage").unwrap();
                let captures = regex.captures(&e.value_type).unwrap();
                let struct_name = captures.get(1).unwrap().as_str().to_string();
                _paths.push(struct_name.clone());
            }
        }
        _paths.push(self.name.replace("\"", ""));
        _paths
    }
    pub fn instance_paths(&self) -> Vec<String> {
        // TODO: iter.child[i] is like ["iter", "child", "child[i]"] in Executable. But what we want is ["iter", "child", "[i]"]
        let _ancestors = self.ancestors();
        let mut _paths = Vec::new();

        for e in _ancestors {
            if e.belongs_to.is_some() {
                if e.mapping_key.is_some() {
                    _paths.push(e.mapping_key.unwrap());
                } else {
                    _paths.push(e.name);
                }
            } else {
                let regex = regex::Regex::new(r"t_struct\((\w+)\)\d{3}_storage").unwrap();
                let captures = regex.captures(&e.value_type).unwrap();
                let struct_name = captures.get(1).unwrap().as_str().to_string();
                _paths.push(struct_name.clone());
            }
        }
        _paths.push(self.name.replace("\"", ""));
        _paths
    }
    pub fn paths_in_conf(&self) -> Vec<String> {
        let _ancestors = self.ancestors();
        let mut _paths = Vec::new();

        for e in _ancestors {
            if e.belongs_to.is_some() {
                if e.mapping_key.is_some() {
                    _paths.push("[i]".to_string());
                } else {
                    _paths.push(e.name);
                }
            } else {
                let regex = regex::Regex::new(r"t_struct\((\w+)\)\d{3}_storage").unwrap();
                let captures = regex.captures(&e.value_type).unwrap();
                let struct_name = captures.get(1).unwrap().as_str().to_string();
                _paths.push(struct_name.clone());
            }
        }
        _paths.push(self.name.replace("\"", ""));
        _paths
    }
    pub fn cid(&self) -> usize {
        ConfigUtil::calc_id(self.class_paths())
    }

}
