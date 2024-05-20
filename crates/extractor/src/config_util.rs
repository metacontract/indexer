use std::path::Ancestors;
use regex::Regex;
use bnf::{Grammar, ParseTree, Term, ParseTreeNode};
// use bnf::ParseTreeNode::Terminal;
// use bnf::ParseTreeNode::Nonterminal;

use super::executable::Executable;


pub struct Config {
    cid: usize,
    parses: Vec<Box<ParseTree>>,
}
impl Config {
    pub fn new(cid: usize, parses:Vec<ParseTree>) -> Self {
        Self {
            cid,
            parses
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ConfigUtil;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum ParseMode {
    Expr,
    Term,
    Factor,
    Base,
    Operator,
    Funcs,
    Vars,
    Fullname,
    Path,
    Char,
}


#[allow(dead_code)]
impl ConfigUtil {
    pub fn calc_id(paths: Vec<String>) -> usize {
        let path_string = paths.join("");
        let hash_bytes = ethers::utils::keccak256(path_string.as_bytes());
        let id_bytes: [u8; 4] = hash_bytes[..4].try_into().unwrap();
        let id = u32::from_be_bytes(id_bytes.try_into().unwrap());
        id as usize
    }


    // returns: bytecode = vec!(lines),  line = vec!(expr.to_array)
    pub fn parse_config(constraint_name:String) -> Vec<Vec<String>> {

        let input = r##"
        <expr> ::= <term> | <term> " " <operator> " " <expr> | <term> <operator> " " <expr> | <term> " " <operator> <expr> | <term> <operator> <expr>
        <term> ::= <factor> | <factor> " " <operator> " " <term> | <factor> <operator> " " <term> | <factor> " " <operator> <term> | <factor> <operator> <term>
        <factor> ::= <base> | <l_paren> <expr> <r_paren>
        <base> ::= <fullname> | <funcs> <l_paren> <expr> <r_paren> | <vars>
        
        <operator> ::= "+" | "-" | "*" | "/" | "%"
        
        <funcs> ::= "createdAt" | "updatedAt" | "head" | "tail"
        <vars> ::= "block.timestamp"
        
        <fullname> ::= <path> | <path> <index> | <path> <delimiter> <fullname> | <path> <index> <delimiter> <fullname>
        <l_paren> ::= "("
        <r_paren> ::= ")"
        <delimiter> ::= "."
        <path> ::= <character> | <character> <path>
        <index> ::= "[i]"
        <character> ::= <letter> | <digit> | <symbol>
        <symbol> ::= "$" | "_"
        <letter> ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" | "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
        <digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
        
        "##;
        let grammar: Grammar = input.parse().unwrap();
        let sentence = constraint_name.clone();
        let parses: Vec<ParseTree> = grammar
            .parse_input(&sentence)
            .collect();

        let bytecode = Self::eval_parse_tree(&parses);

        bytecode.clone()
    }
    pub fn to_class_paths(name:String) -> Vec<String> {
        name.clone()
            .split(".")
            .map(|part| part.replace("[i]", ""))
            .collect::<Vec<_>>()      
    }

    // TODO: [2] Constraint to ParseTree. Must be used in registry.rs:L90
    pub fn eval_parse_tree(
        parses: &Vec<ParseTree>,
    ) -> Vec<Vec<String>> {

        let mut stack = Vec::new();
        let tree_stack = Box::new(Vec::new());
        for parse in parses {
            Self::slice_newline(parse, &mut stack, &mut tree_stack);
        }
        stack.clone()
    }

    pub fn eval_bytecode(mut stack: Vec<Vec<String>>, e: Executable) -> usize {
        stack.reverse();
        let mut reversed_stack = stack.clone();
        for (i, line) in reversed_stack.iter().enumerate() {
            // Note: Parsed reversed stack compresses lines from primitive values to complex formula
            // If line didn't meet with each function confition, then nothing would be changed.

            line = Self::expand_filtrated_var_with_value(line); // 1. var to value
            line = Self::apply_precalculated_vars(line, &mut reversed_stack); // ex
            line = Self::apply_func(line);
            (operator_op, operator_args_as_line) = Self::arrange_for_operator(line);
            line = Self::apply_operator(line);

            reversed_stack[i] = line.clone();
        }

        reversed_stack.last().unwrap().get(0).unwrap().parse::<usize>().unwrap()
    }


    fn slice_newline(new_parse: &ParseTree, mut stack: &mut Vec<Vec<String>>, tree_stack: &mut Box<Vec<Vec<ParseTree>>>) -> () {
        let mut current_line = Vec::new();

        // if new_parse has single rhs and it is fullname, vars, or char, then jdo nothing
        let current_stack_depth = stack.len();
        for (i, rhs) in new_parse.rhs_iter().enumerate() {
            match rhs {
                ParseTreeNode::Terminal(_) => {
                }
                ParseTreeNode::Nonterminal(new_parse2) => {
                    tree_stack[current_stack_depth].push(new_parse2.clone());
                }
            }
        }

        let mut copy_tree_stack = tree_stack.clone();
        for (i, parse) in copy_tree_stack[current_stack_depth].clone().iter_mut().enumerate() {
            match Self::check_mode(parse) {
                ParseMode::Expr => {
                    // down to next
                    Self::slice_newline(parse, stack, tree_stack);
                },
                ParseMode::Term => {
                    // down to next
                    Self::slice_newline(parse, stack, tree_stack);
                },
                ParseMode::Factor => {
                    // down to next
                    Self::slice_newline(parse, stack, tree_stack);
                },
                ParseMode::Base => {
                    // down to next
                    Self::slice_newline(parse, stack, tree_stack);
                },
                ParseMode::Operator=> {
                    // Nice! just push.
                    current_line.push(Self::parse_concat(parse));
                },
                ParseMode::Funcs=> {
                    current_line.push(Self::parse_concat(parse)); // push this to current_line
                    copy_tree_stack[current_stack_depth].remove(i); // regard following tree L_N

                    let new_line_depth = current_stack_depth + 1;
                    // TODO: don't push, insert
                    current_line.push(format!("L{}", new_line_depth)); // insert L_N to current_line, current index
                    tree_stack[new_line_depth] = copy_tree_stack[current_stack_depth].clone(); // push following tree to newline
                },
                ParseMode::Fullname=> {
                    if copy_tree_stack[current_stack_depth].len() == 1 {
                        current_line.push(Self::parse_concat(parse)); // push a fullname concat to current line
                    } else {
                        copy_tree_stack[current_stack_depth].remove(i);

                        let new_line_depth = current_stack_depth + 1;
                        // TODO: don't push, insert
                        current_line.push(format!("L{}", new_line_depth));// insert L_N to current_line, current index
                        tree_stack[new_line_depth] = copy_tree_stack[current_stack_depth].clone(); // push fullname to newline
                    }
                },
                ParseMode::Vars=> {
                    if copy_tree_stack[current_stack_depth].len() == 1 {
                        current_line.push(Self::parse_concat(parse)); // push a fullname concat to current line
                    } else {
                        copy_tree_stack[current_stack_depth].remove(i);

                        let new_line_depth = current_stack_depth + 1;
                        // TODO: don't push, insert
                        current_line.push(format!("L{}", new_line_depth));// insert L_N to current_line, current index
                        tree_stack[new_line_depth] = copy_tree_stack[current_stack_depth].clone(); // push var to newline
                    }
                },
                _ => {

                }
            }
        }
    }

    fn check_mode(parse: &ParseTree) -> ParseMode {
        match parse.lhs {
            Term::Terminal(_) => {
                panic!("Termianl lhs is unknown.");
            },
            Term::Nonterminal(lhs) => {
                if lhs == "expr" {
                    ParseMode::Expr
                } else if lhs == "base" {
                    ParseMode::Base
                } else if lhs == "term" {
                    ParseMode::Term
                } else if lhs == "factor" {
                    ParseMode::Factor
                } else if lhs == "operator" {
                    ParseMode::Operator
                } else if lhs == "funcs" {
                    ParseMode::Funcs
                } else if lhs == "vars" {
                    ParseMode::Vars
                } else if lhs == "index" {
                    ParseMode::Path
                } else if lhs == "fullname" {
                    ParseMode::Fullname
                } else if lhs == "path" {
                    ParseMode::Path
                } else if lhs == "charactor" {
                    ParseMode::Char
                } else if lhs == "l_paren" {
                    ParseMode::Char
                } else if lhs == "r_paren" {
                    ParseMode::Char
                } else if lhs == "delimiter" {
                    ParseMode::Char                                        
                } else {
                    panic!("{} didn't match for mode check.", lhs);
                }
            }
        }
    }

    fn parse_concat(path_parse: &ParseTree) -> String {
        let mut _string_array = Vec::new();
        for rhs in path_parse.rhs_iter() {
            _string_array.push(match rhs {
                ParseTreeNode::Terminal(value) => value.to_string(),
                ParseTreeNode::Nonterminal(rhs) => Self::parse_concat(rhs),
            })
        };
        _string_array.join("")
    }
    fn arrange_for_operator(line) -> (Option<String>, Vec<String>) {
        let mut operator:Option<String> = None;
        for (i, opcode) in line.iter().enumerate() {
            if opcode.is_operator() {
                operator = Some(opcode.clone());
                line.remove(i);
            }
        };

        operator match {
            Some(opeartor) => {
                vec!(operator).concat(line)
            },
            None => {
                line
            }
        }
    }
    
    fn apply_precalculated_vars(line: &mut Vec<String>, revstack) -> Vec<String> {
        for (i, opcode) in line.iter().enumerate() {
            if opcode.test("^L(\d+)$") {
                line[i] = revstack[revstack.len() - captured.get(0).unwrap()];
            }
        }
        line
    }
    
    fn expand_filtrated_var_with_value(mut line: Vec<String>) -> Vec<String> {
        for (i, opcode) in line.iter().enumerate() {
            let _var = Self::is_reserved_words(opcode);
            match _var {
                Some(_var) => {
                    line[i] = Self::access_var(_var);
                    // registry.get_iid(ConfigUtil::calc_id(ConfigUtil::to_class_paths(opcode))
                },
                None => {
                    line[i] = Self::access_values_or_db(opcode);
                }
            }
        }
        line
    }
    fn apply_func(line) -> Vec<String> {

    }
    
    fn apply_operator(line) -> Vec<String> {
    
    }
}