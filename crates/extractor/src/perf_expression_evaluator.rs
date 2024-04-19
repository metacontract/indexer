use super::registry::Registry;
use super::ast_node::ASTNode;
use super::context::Context;
use std::rc::Rc;
use std::cell::RefCell;

pub struct PerfExpressionEvaluator;

#[derive(Debug)]
enum ExpressionNode {
    Variable(String),
    FunctionCall(String, Vec<ExpressionNode>),
    Operator(String, Box<ExpressionNode>, Box<ExpressionNode>),
    Literal(usize),
}

impl PerfExpressionEvaluator {
    pub fn eval(expression: String, registry: &Registry) -> usize {
        let parsed_expression = Self::parse_expression(expression);
        Self::evaluate_expression(parsed_expression, &registry)
    }

    fn parse_expression(expression: String) -> ExpressionNode {
        // TODO: Implement the expression parser
        // The parser should return an ExpressionNode representing the parsed expression tree
        // ExpressionNode can be an enum with variants for different types of nodes (e.g., variable, function call, operator)
        // Use the vars defined in the YAML file to handle variables like $p, $h, $c

        // Pseudo code:
        // - Tokenize the expression
        // - Build the expression tree based on the token stream
        // - Handle variables, function calls, and operators according to the YAML specification
        // - Return the root node of the expression tree

        // For simplicity, let's assume the expression is a single variable for now
        ExpressionNode::Variable(expression)
    }

    fn evaluate_expression(node: ExpressionNode, registry: &Registry) -> usize {
        match node {
            ExpressionNode::Variable(var) => {
                // TODO: Use the ASTNode and Registry to get the corresponding executable accessor
                // Retrieve the value of the corresponding schema slot
                match var.as_str() {
                    "$p" => {
                        // Get the value of $p from the registry
                        // Placeholder value for now
                        42
                    }
                    "$h" => {
                        // Get the value of $h from the registry
                        // Placeholder value for now
                        100
                    }
                    "$c" => {
                        // Get the value of $c from the registry
                        // Placeholder value for now
                        200
                    }
                    _ => panic!("Unknown variable: {}", var),
                }
            }
            ExpressionNode::FunctionCall(func, _) => {
                // TODO: Evaluate the arguments recursively
                // Call the corresponding function with the evaluated arguments
                match func.as_str() {
                    "updatedAt" => {
                        // Placeholder value for now
                        1234567890
                    }
                    "block.timestamp" => {
                        // Placeholder value for now
                        987654321
                    }
                    "head" => {
                        // Placeholder value for now
                        10
                    }
                    "tail" => {
                        // Placeholder value for now
                        20
                    }
                    _ => panic!("Unknown function: {}", func),
                }
            }
            ExpressionNode::Operator(op, left, right) => {
                // TODO: Evaluate the left and right operands recursively
                // Apply the operator to the evaluated operands
                let left_value = Self::evaluate_expression(*left, &registry);
                let right_value = Self::evaluate_expression(*right, &registry);
                match op.as_str() {
                    "/" => left_value / right_value,
                    "*" => left_value * right_value,
                    _ => panic!("Unknown operator: {}", op),
                }
            }
            ExpressionNode::Literal(value) => value,
        }
    }
}