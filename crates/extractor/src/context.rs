use super::executor::Executor;
use super::registry::Registry;
use super::ast_node::ASTNode;
use std::collections::HashMap;
use serde_json::Value;

#[derive(Clone)]
pub struct Context {
    pub registry: Registry,
}
