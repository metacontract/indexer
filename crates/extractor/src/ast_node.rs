// use super::compiler::Compiler;
// use super::extractor::Extractor;
// use super::executor::Executor;
use super::registry::Registry;
use super::executable::Executable;
// use super::perf_config_item::PerfConfigItem;
use super::type_kind::TypeKind;
// use super::eth_call::EthCall;
use super::perf_expression_evaluator::PerfExpressionEvaluator;
// use super::ast_node::ASTNode;


use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;

#[derive(Clone)]
pub struct ASTNode;
pub struct ParsedASTType {
    pub envelope: Option<TypeKind>,
    pub key_type: Option<String>,
    pub value_type: String,
}

impl ASTNode {
    pub fn type_kind(type_str: &str) -> TypeKind {
        let parsed_type = ASTNode::parse_type_str(type_str);
        if let Some(envelope) = parsed_type.envelope {
            match envelope {
                TypeKind::Array => TypeKind::Array,
                TypeKind::Mapping => TypeKind::Mapping,
                _ => panic!("Unsupported TypeKind: {:?}", envelope),
            }
        } else {
            let value_type = parsed_type.value_type;
            if value_type.starts_with("t_struct") {
                TypeKind::NaiveStruct
            } else {
                TypeKind::Primitive
            }
        }
    }
    pub fn parse_type_str(type_str: &str) -> ParsedASTType {
        let mut parsed_type = ParsedASTType {
            envelope: None,
            key_type: None,
            value_type: String::new(),
        };

        // Check if the type is an array
        if type_str.starts_with("t_array(") {
            parsed_type.envelope = Some(TypeKind::Array);

            // Extract the element type
            let element_type = type_str.split("(").nth(1).unwrap().split(")").next().unwrap();
            parsed_type.value_type = element_type.to_string();
        }
        // Check if the type is a mapping
        else if type_str.starts_with("t_mapping(") {
            parsed_type.envelope = Some(TypeKind::Mapping);

            // Extract the key and value types
            let mut parts = type_str.split(",");
            let key_type = parts.next().unwrap().split("(").nth(1).unwrap();
            let value_type = parts.next().unwrap().split(")").next().unwrap();
            parsed_type.key_type = Some(key_type.to_string());
            parsed_type.value_type = value_type.to_string();
        }
        // Primitive type
        else {
            parsed_type.value_type = type_str.to_string();
        }

        parsed_type
    }
}
