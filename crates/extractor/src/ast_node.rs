use super::type_kind::TypeKind;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;

pub struct ASTNode {
    blob: Value,
    types: Value,
}
pub struct ParsedASTType {
    envelope: Option<TypeKind>,
    key_type: Option<String>,
    value_type: String,
}

impl ASTNode {
    pub fn new(&mut self, blob:Value) -> Self {
        // ASTNode["contracts"]["src/_utils/Dummy.sol"]["Dummy"]["storageLayout"]["types"][]

        self.blob = blob;
        self.types = blob["contracts"]["src/_utils/Dummy.sol"]["Dummy"]["storageLayout"]["types"];
    }
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

    pub fn fill_iter_unless_empty_index(&self, executable: &Executable) -> bool {
        // If the iterator's `to` field is empty (likely a mapping)

        let from_expression = perf_config_item.as_ref().and_then(|item| item.from.clone());
        let to_expression = perf_config_item.as_ref().and_then(|item| item.to.clone());

        if let Some(from_expr) = from_expression {
            let parsed_from = PerfExpressionEvaluator::eval(from_expr);
            iter.set_from(parsed_from);
        }

        if let Some(to_expr) = to_expression {
            let parsed_to = PerfExpressionEvaluator::eval(to_expr);
            executable.iter.set_to(parsed_to);
        }

        // Skip 1 step to wait until config-specified info is filled
        if parsed_to == 0 {
            executable.increment_step();
            executable.enqueue_execution();
            false
        } else {
            true
        }
    }
    pub fn labels(&self, executable: &Executable) -> Vec<String> {
        if (executable.is_iterish() && iter.to? > 0) {
            // This executable is iterable member
            let iter = executable.iter().unwrap();
            let value_types = (0..iter.to).map(|i| {
                let current_node = executable.visit(executable.get_edfs().concat(vec!["<next>"]));
                current_node.value_type
            }).collect();
            value_types

        } else {
            // Check if the type is a struct
            if executable.type_kind == TypeKind::Struct {
                // Return all labels (type names) of the members
                executable.members.iter().map(|member| member.label.clone()).collect()
            } else {
                // Primitive type, throw error
                panic!("Primitive type, cannot list labels");
            }
        }
    }
    pub fn children(&self, executable: &Executable) -> Vec<Executable> {
        let mut children = Vec::new();
        let labels = self.labels();
        for i in 0..labels.len() {
            let current_node = self.visit(&labels[i]);
            let new_executable = Executable::new(
                current_node.label, // label of the current node
                executed.step + 1, // step of the current node
                Some(&executed), // set the belongs_to to the current executable
                ASTNode::type_kind(current_node.r#type), // type kind of the current node
                ASTNode::parse_type_str(current_node.r#type), // type of the current node
                current_node.offset, // offset of the current node
                current_node.slot, // slot of the current node
                Some(executed), // the AST node
                if executed.is_iterish() { // check iter or not
                    Some(i) // mapping key
                } else {
                    None // depends on is_iterish
                },
                if ASTNode::type_kind(current_node.r#type).is_iterish() {
                    Some(IteratorMeta {
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
    pub fn visit(&mut self, label: &str) -> Value {
        return self.types.get(label).cloned().unwrap_or_default();
    }
    pub fn enqueue_children(&self) {
        let children = self.children();
        for child in children {
            child.enqueue_execution();
        }
    }

}
