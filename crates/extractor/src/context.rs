use super::executor::Executor;
use super::registry::Registry;
use super::ast_node::ASTNode;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Context<'a> {
    pub registry: Registry<'a>,
    pub ast_node: ASTNode,
}

impl Context<'_> {
    pub fn dummy() -> Self {
        // Create and return a dummy instance of Context
        // You may need to adjust this based on your Context struct definition
        Context {
            registry: Registry::new(HashMap::new()),
            ast_node: ASTNode::dummy(),
        }
    }    
}