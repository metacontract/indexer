#![allow(unused_imports)]

mod compiler;
mod extractor;
mod executor;
mod registry;
mod executable;
mod config_util;
mod type_kind;
mod eth_call;
mod perf_expression_evaluator;
mod ast_node;
mod context;
mod mc_repo_fetcher;

extern crate dotenv;

use compiler::Compiler;
use extractor::Extractor;
use executor::Executor;
use registry::Registry;
use executable::Executable;
use config_util::ConfigUtil;
use type_kind::TypeKind;
use eth_call::EthCall;
use perf_expression_evaluator::PerfExpressionEvaluator;
use ast_node::ASTNode;
use context::Context;
use mc_repo_fetcher::MCRepoFetcher;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use std::cell::RefCell;
use std::env;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let project_root = env::current_dir().unwrap();
    let identifier = env::var("REPO_IDENTIFIER").unwrap();
    let bundle = env::var("BUNDLE_NAME").unwrap();

    let mc_repo_fetcher = MCRepoFetcher::new(identifier.clone(), bundle.clone(), Some(project_root.clone()));
    // mc_repo_fetcher.clone_repo().unwrap();
    

    let mut compiler = Compiler::new("solc".to_string(), project_root.clone(), identifier.clone(), bundle.clone());
    let base_slots = compiler.prepare_base_slots().unwrap();

    let base_slots_vec: Vec<String> = base_slots.keys().cloned().collect();
    mc_repo_fetcher.gen_dummy_contract(&base_slots_vec).unwrap();
    mc_repo_fetcher.gen_standard_json_input().unwrap();
    let storage_layout_blob = compiler.prepare_storage_layout().unwrap();

    #[allow(unused_mut)]
    let mut context = Context {
        registry: Registry::new(
                    storage_layout_blob,
                    mc_repo_fetcher.load_perf_config().expect("config load failed"),
                    bundle.clone()
                ),
    };

    let mut extractor = Extractor::new(context);
    extractor.init_members_from_compiler(&base_slots);

    extractor.listen().await;
}