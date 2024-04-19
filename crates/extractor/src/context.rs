use super::executor::Executor;
use super::registry::Registry;
use super::ast_node::ASTNode;
use std::collections::HashMap;
use serde_json::Value;

#[derive(Clone)]
pub struct Context<'a> {
    pub registry: Registry<'a>,
}

impl Context<'_> {
    pub fn dummy() -> Self {
    // Create and return a dummy instance of Context
    // You may need to adjust this based on your Context struct definition
        Context {
            registry: Registry::new(serde_json::Value::default(), HashMap::new()),
        }
    }
}