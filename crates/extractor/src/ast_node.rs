use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;

pub struct Node {
    label: String,
    members: Vec<Node>,
    offset: usize,
    slot: String,
    node_type: String,
}
pub struct ASTNode {
    nodes: HashMap<String, Node>,
}

impl ASTNode {
    pub fn from_storage_layout(storage_layout: Value) -> Self {
        let mut nodes = HashMap::new();
        let contracts = storage_layout["contracts"].as_object().unwrap();
        for (contract_name, contract) in contracts {
            let contract_object = contract.as_object().unwrap();
            let storage_layout = contract_object["storageLayout"].as_object().unwrap();
            let storage = storage_layout["storage"].as_array().unwrap();
            for item in storage {
                let label = item["label"].as_str().unwrap().to_string();
                let offset = item["offset"].as_u64().unwrap() as usize;
                let slot = item["slot"].as_str().unwrap().to_string();
                let node_type = item["type"].as_str().unwrap().to_string();
                let members = Self::parse_members(&storage_layout["types"], &node_type);
                let node = Node { label, members, offset, slot, node_type };
                let edfs = format!("{}.{}", contract_name, label);
                nodes.insert(edfs, node);
            }
        }
        Self { nodes }
    }

    pub fn parse_members(types: &Value, node_type: &str) -> Vec<Node> {
        let mut members = Vec::new();
        if let Some(node_type_value) = types[node_type].as_object() {
            if let Some(node_members) = node_type_value["members"].as_array() {
                for member in node_members {
                    let label = member["label"].as_str().unwrap().to_string();
                    let offset = member["offset"].as_u64().unwrap() as usize;
                    let slot = member["slot"].as_str().unwrap().to_string();
                    let member_type = member["type"].as_str().unwrap().to_string();
                    let member_node = Node {
                        label,
                        members: Self::parse_members(types, &member_type),
                        offset,
                        slot,
                        node_type: member_type,
                    };
                    members.push(member_node);
                }
            }
        }
        members
    }

    pub fn get_node(&self, edfs: &str) -> Option<&Node> {
        self.nodes.get(edfs)
    }
}