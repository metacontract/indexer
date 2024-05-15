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


    pub fn parse_config(constraint_name:String) -> Vec<ParseTree> {

        let input = r##"
        <expr> ::= <term> | <term> " " <operator> " " <expr> | <term> <operator> " " <expr> | <term> " " <operator> <expr> | <term> <operator> <expr>
        <term> ::= <factor> | <factor> " " <operator> " " <term> | <factor> <operator> " " <term> | <factor> " " <operator> <term> | <factor> <operator> <term>
        <factor> ::= <base> | <l_paren> <expr> <r_paren>
        <base> ::= <fullname> | <funcs> <l_paren> <expr> <r_paren> | <vars>
        
        <operator> ::= "+" | "-" | "/" | "*"
        
        <funcs> ::= "updatedAt"
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

        parses
    }
    pub fn to_class_paths(name:String) -> Vec<String> {
        name.clone()
            .split(".")
            .map(|part| part.replace("[i]", ""))
            .collect::<Vec<_>>()      
    }

    // TODO: [2] Constraint to ParseTree. Must be used in registry.rs:L90
    pub fn eval_parse_tree(
        parse: &ParseTree,
        stack: Option<Vec<String>>
    ) -> Vec<String> {
        let mut stack: Vec<String> = if let Some(s) = stack {
            s
        } else {
            Vec::new()
        };

        for rhs in parse.rhs_iter() {
            match rhs {
                ParseTreeNode::Terminal(_) => {
                }
                ParseTreeNode::Nonterminal(new_parse) => {
                    match Self::check_mode(new_parse) {
                        ParseMode::Expr => {
                            stack = Self::eval_parse_tree(new_parse, Some(stack));
                        },
                        ParseMode::Base => {
                            stack = Self::eval_parse_tree(new_parse, Some(stack));
                        },
                        ParseMode::Factor => {
                            stack = Self::eval_parse_tree(new_parse, Some(stack));
                        },
                        ParseMode::Term => {
                            stack = Self::eval_parse_tree(new_parse, Some(stack));
                        },
                        ParseMode::Operator=> {
                            stack.push(Self::parse_concat(new_parse));
                        },
                        ParseMode::Fullname => {
                            stack.push(Self::parse_concat(new_parse));
                        },
                        ParseMode::Vars => {
                            stack.push(Self::parse_concat(new_parse));
                        },
                        ParseMode::Funcs=> {
                            stack.push(Self::parse_concat(new_parse));
                        },
                        ParseMode::Char => {
                            stack.push(Self::parse_concat(new_parse));
                        },
                        _ => {
                        }
                    }
                }
    
            }            
        }
        stack.clone()
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
    
}
