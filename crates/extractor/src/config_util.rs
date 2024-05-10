use std::path::Ancestors;
use regex::Regex;
use bnf::{Grammar, ParseTree, Term, ParseTreeNode};
// use bnf::ParseTreeNode::Terminal;
// use bnf::ParseTreeNode::Nonterminal;

use super::executable::Executable;

#[allow(dead_code)]
#[derive(Clone)]
pub struct ConfigUtil;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum ParseMode {
    Expr,
    Clause,
    Operand,
    Funcs,
    Vars,
    Index,
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

    pub fn to_class_paths(name:String) -> Vec<String> {
        let input =
            "<expr> ::= <clause> | <clause> <math_operand> <expr> | <clause> ' ' <math_operand> <expr> | <clause> <math_operand> ' ' <expr> | <clause> ' ' <math_operand> ' ' <expr>
            <math_operand> ::= '+' | '-' | '/' | '*'
            <clause> ::= <fullname> | <funcs> <l_paren> <fullname> <r_paren> | <vars>
            <l_paren> ::= '('
            <r_paren> ::= ')'
            <funcs> ::= 'head' | 'tail' | 'updatedAt'
            <vars> ::= 'block.timestamp'
            <fullname> ::= <path> | <path> <index> | <path> <path_delimiter> <fullname> | <path> <index> <path_delimiter> <fullname>
            <path_delimiter> ::= '.'
            <path> ::= <character> | <character> <path>
            <index> ::= '[i]'
            <character> ::= <letter> | <digit> | <symbol>
            <symbol> ::= '$' | '_'
            <letter> ::= 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z'
            <digit> ::= '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
            ";
        let grammar: Grammar = input.parse().unwrap();
        let sentence = name.clone();
        let parses: Vec<ParseTree> = grammar
            .parse_input(&sentence)
            .collect();

        let mode_memo = ParseMode::Expr;
        let mut funcs_memo = Vec::new();
        let mut vars_memo = Vec::new();
        let mut paths_memo = Vec::new();
        let mut expr_memo = Vec::new();

        for parse in parses {
            Self::analyze_parse_tree(&parse, mode_memo.clone(), &mut funcs_memo, &mut vars_memo, &mut paths_memo, &mut expr_memo);
        }

        // Print the memoized information
        println!("Functions: {:?}", funcs_memo);
        println!("Variables: {:?}", vars_memo);
        println!("Paths: {:?}", paths_memo);
        println!("Expressions: {:?}", expr_memo);

        name.clone()
            .split(".")
            .map(|part| part.replace("[i]", ""))
            .collect::<Vec<_>>()      
    }

    fn analyze_parse_tree(
        parse: &ParseTree,
        mut mode_memo: ParseMode,
        funcs_memo: &mut Vec<String>,
        vars_memo: &mut Vec<String>,
        paths_memo: &mut Vec<String>,
        expr_memo: &mut Vec<(String, String, String)>,
    ) {
        match parse.lhs {
            Term::Terminal(_) => {

            },
            Term::Nonterminal(lhs) => {
                /*
                Clause
                Operand,
                Funcs,
                Vars,
                Index,
                Fullname,
                Path,
                Char,
                Letter,
                Digit,
                Symbol
                */
                if lhs == "expr" {
                } else if lhs == "clause" {
                    mode_memo = ParseMode::Clause;   
                } else if lhs == "math_operand" {
                    mode_memo = ParseMode::Operand;   
                } else if lhs == "funcs" {
                    mode_memo = ParseMode::Funcs;   
                } else if lhs == "vars" {
                    mode_memo = ParseMode::Vars;   
                } else if lhs == "index" {
                    mode_memo = ParseMode::Index;   
                } else if lhs == "fullname" {
                    mode_memo = ParseMode::Fullname;   
                } else if lhs == "path" {
                    mode_memo = ParseMode::Path;   
                } else if lhs == "charactor" {
                    mode_memo = ParseMode::Char;   
                } 
            }
        }
        for rhs in parse.rhs_iter() {
            match rhs {
                ParseTreeNode::Terminal(_) => {
                }
                ParseTreeNode::Nonterminal(rhs) => {
                    // Note: mode_memo here is parse's mode. So rhs (child) mode is gonna be one-layer digged one. (e.g., Expr-Clause)
                    match mode_memo {
                        ParseMode::Expr => {
                            // store clause operand clause
                            // println!("clause:{:?}", rhs);
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            Self::analyze_parse_tree(rhs, mode_memo.clone(), funcs_memo, vars_memo, paths_memo, expr_memo);
                        },
                        ParseMode::Clause => {
                            Self::analyze_parse_tree(rhs, mode_memo.clone(), funcs_memo, vars_memo, paths_memo, expr_memo);
                            // println!("fullname:{:?}", rhs);
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                        },
                        ParseMode::Fullname => {
                            // store funcs and paths of a clause, or vars
                            paths_memo.push(Self::reconstruct_path(rhs));
                            // println!("path:{:?}", rhs);
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                        },
                        ParseMode::Path => {
                            // println!("char:{:?}", rhs);
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                        },
                        ParseMode::Index => {
                            // println!("char:{:?}", rhs);
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                        },
                        ParseMode::Vars => {
                            // println!("char:{:?}", rhs);
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                        },
                        ParseMode::Funcs=> {
                            // println!("char:{:?}", rhs);
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                        },
                        ParseMode::Operand=> {
                            // println!("char:{:?}", rhs);
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                            // println!("{:?}", "=============");
                        },
                        _ => {}
                    }
                }
    
            }            
        }
    }

    fn reconstruct_path(path_parse: &ParseTree) -> String {
        let mut _string_array = Vec::new();
        for rhs in path_parse.rhs_iter() {
            _string_array.push(match rhs {
                ParseTreeNode::Terminal(value) => value.to_string(),
                ParseTreeNode::Nonterminal(rhs) => Self::reconstruct_path(rhs),
            })
        };
        _string_array.join("")
    }
    
}
