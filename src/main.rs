mod ethcall;

use std::collections::HashMap;
use std::process::Command;
use serde_json::Value;
use ethcall::{EthCall};


enum TypeKind {
    Mapping,
    Array,
    NaiveStruct,
    Primitive,
}
impl TypeKind {
    fn is_iterish(&self) -> bool {
        match self {
            TypeKind::Mapping | TypeKind::Array => true,
            TypeKind::NaiveStruct | TypeKind::Primitive => false,
        }
    }    
}

struct Compiler {
    solc_path: String,
    base_slot_ast_cache: Option<String>,
    storage_layout_ast_cache: Option<String>,
}

impl Compiler {
    fn new(solc_path: String) -> Self {
        Self {
            solc_path,
            base_slot_ast_cache: None,
            storage_layout_ast_cache: None,
        }
    }

    fn prepare_base_slots(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(ref cache) = self.base_slot_ast_cache {
            return Ok(serde_json::from_str(cache)?);
        }

        let solc_opts = "./solcBaseSlotsOpts.json";
        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(solc_opts)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        self.base_slot_ast_cache = Some(stdout);

        Ok(parsed)
    }

    fn prepare_storage_layout(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(ref cache) = self.storage_layout_ast_cache {
            return Ok(serde_json::from_str(cache)?);
        }

        let solc_opts = "./solcLayoutOpts.json";
        let output = Command::new(&self.solc_path)
            .arg("--standard-json")
            .arg(solc_opts)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let parsed: Value = serde_json::from_str(&stdout)?;

        self.storage_layout_ast_cache = Some(stdout);

        Ok(parsed)
    }
}


struct Extractor {
    compiler: Compiler,
    registry: Registry,
    initial_members: Vec<Box<dyn Executable>>,
    executor: Executor,
    perf_expression_evaluator: PerfExpressionEvaluator,
}

impl Extractor {
    fn new() -> Self {
        let compiler = Compiler::new("solc".to_string());
        let storage_layout = compiler.prepare_storage_layout().unwrap();
        let registry = Registry::new(HashMap::new(), storage_layout);
        let executor = Executor::new(registry.clone());

        Self {
            compiler,
            registry,
            initial_members: Vec::new(),
            executor
        }
    }

    fn init_members_from_compiler(&mut self) {
        let base_slots = self.compiler.prepare_base_slots().unwrap();

        // Create Member objects from base_slots and storage_layout
        for (name, slot_info) in base_slots.as_object().unwrap() {
            let type_kind = match slot_info["type"].as_str().unwrap() {
                "t_mapping" => TypeKind::Mapping,
                "t_array" => TypeKind::Array,
                "t_struct" => TypeKind::NaiveStruct,
                _ => TypeKind::Primitive,
            };

            let value_type = slot_info["valueType"].as_str().unwrap().to_string();
            let relative_slot = slot_info["slot"].as_str().unwrap().to_string();

            let member = Box::new(Member::new(
                name.to_string(),
                type_kind,
                value_type,
                relative_slot,
                self.clone(),
                None, // Initialize iter as None, it will be populated later if needed
            ));

            self.initial_members.push(member);
        }
    }

    fn listen(&mut self) {
        // Listen for events and process them
        // ...

        self.scan_contract();
    }

    fn scan_contract(&mut self) {
        let mut step = 0;
        loop {
            self.executor.bulk_exec_and_enqueue_and_set_primitive_to_output(step, &mut self.registry, &mut self.perf_expression_evaluator);
            step += 1;

            if step > 15 {
                break;
            }
        }
    }
}

struct PerfConfigItem {
    edfs: String,
    from: Option<String>,
    to: Option<String>,
}
struct Member {
    name: String,
    type_kind: TypeKind,
    value_type: String,
    offset: usize,
    relative_slot: String,
    absolute_slot: Option<String>,
    belongs_to: Option<Box<dyn Executable>>,
    iter: Option<IteratorMeta>,
    step: usize,
    value: Option<String>,
    ast_node: Option<&'static Node>,
}

impl Member {
    fn new(
        name: String,
        type_kind: TypeKind,
        value_type: String,
        relative_slot: String,
        belongs_to: Extractor,
        iter: Option<IteratorMeta>,
    ) -> Box<Self> {
        Box::new(Self {
            name,
            type_kind,
            value_type,
            relative_slot,
            belongs_to: Some(Box::new(belongs_to)),
            absolute_slot: None,
            iter,
            step: 0,
            value: None,
            ast_node: None,
        })
    }
}

struct IteratorMeta {
    key_type: Option<String>,
    perf_config: Option<PerfConfigItem>,
    items: Vec<IteratorItem>,
    from: usize,
    to: usize,
}

impl IteratorMeta {
    fn new(
        key_type: Option<String>,
        perf_config: Option<PerfConfigItem>,
        items: Vec<IteratorItem>,
        from: usize,
        to: usize,
    ) -> Self {
        IteratorMeta {
            key_type,
            perf_config,
            items,
            from,
            to,
        }
    }


    fn set_from(&mut self, from: usize) {
        self.from = from;
    }

    fn set_to(&mut self, to: usize) {
        self.to = to;
    }
}

struct IteratorItem {
    name: String,
    type_kind: TypeKind,
    value_type: String,
    relative_slot: String,
    belongs_to: Option<Box<dyn Executable>>,
    absolute_slot: Option<String>,
    iter: Option<IteratorMeta>,
    mapping_key: Option<String>,
    step: usize,
    value: Option<String>,
}

impl IteratorItem {
    fn new(
        name: String,
        type_kind: TypeKind,
        value_type: String,
        relative_slot: String,
        belongs_to: Box<dyn Executable>,
        mapping_key: Option<String>,
        iter: Option<IteratorMeta>,
    ) -> Box<Self> {
        Box::new(Self {
            name,
            type_kind,
            value_type,
            relative_slot,
            belongs_to: Some(belongs_to),
            mapping_key,
            absolute_slot: None,
            iter,
            step: 0,
            value: None,
        })
    }
}


struct Executor {
    queue_per_step: Vec<Vec<Box<dyn Executable>>>,
    executed_per_step: Vec<Vec<Box<dyn Executable>>>,
    registry: Registry,
}

impl Executor {
    fn new(registry: Registry) -> Self {
        Self {
            queue_per_step: Vec::new(),
            executed_per_step: Vec::new(),
            registry,
        }
    }

    fn bulk_exec_and_enqueue_and_set_primitive_to_output(&mut self, step: usize, registry: &mut Registry, perf_expression_evaluator: &mut PerfExpressionEvaluator) {

        // Get values by slots from EthCall
        let slots: HashMap<String, String> = self.queue_per_step[step].iter().map(|executable| (executable.get_edfs(), executable.get_abs_slot().unwrap_or_default())).collect();
        let values = EthCall::get_values_by_slots(&slots, "mainnet", "0x1234567890123456789012345678901234567890", "0x1234567890123456789012345678901234567890");

        // Flush the queue for the current step
        self.flush_queue(step);

        // Process each executed executable
        for executable in &mut self.executed_per_step[step] {
            // Get the performance configuration item for the executable
            let perf_config_item = registry.get_perf_config_item(executable.get_edfs());

            // Set the value for the executable based on the performance configuration item
            executable.set_value(values[executable.get_edfs()]);

            if executable.get_type_kind() == TypeKind::Primitive {
                // If the executable is a primitive, push it to the output
                registry.set_output(executable.get_edfs(), executable.clone());
            } else {
                // If the executable is a non-primitive (NaiveStruct, Array, Mapping)

                // Check if the executable has an iterator
                if let Some(iter) = &mut executable.get_iter_mut() {
                    if !iter.to.is_empty() {
                        // If the iterator has a non-empty `to` field
                        for i in 0..iter.to {
                            // Create new executables for each item in the iterator
                            let new_executable = executable.clone_box();
                            new_executable.enqueue_execution(registry);
                        }
                    } else {
                        // If the iterator's `to` field is empty (likely a mapping)
                        let from_expression = perf_config_item.as_ref().and_then(|item| item.from.clone());
                        let to_expression = perf_config_item.as_ref().and_then(|item| item.to.clone());

                        if let Some(from_expr) = from_expression {
                            let parsed_from = perf_expression_evaluator.eval(from_expr);
                            iter.set_from(parsed_from);
                        }

                        if let Some(to_expr) = to_expression {
                            let parsed_to = perf_expression_evaluator.eval(to_expr);
                            iter.set_to(parsed_to);
                        }

                        // Skipping algorithm for a mapping's unloaded bin_index
                        if iter.to == 0 {
                            executable.increment_step();
                            executable.enqueue_execution(registry);
                        } else {
                            let children = executable.get_children();
                            for child in children {
                                child.enqueue_execution(registry);
                            }
                        }

                    }
                } else {
                    // If the executable doesn't have an iterator
                    if executable.get_abs_slot().is_some() && executable.get_value().is_none() {
                        // If the executable has an absolute slot but no value, enqueue it for execution
                        executable.enqueue_execution(registry);
                    }
                }
            }
        }

        // Flush the executed executables for the current step
        self.flush_executed(step);
    }

    fn flush_queue(&mut self, step: usize) {
        self.queue_per_step[step].clear();
    }

    fn flush_executed(&mut self, step: usize) {
        self.executed_per_step[step].clear();
    }
}

struct Registry {
    perf_config_items: HashMap<String, PerfConfigItem>,
    output_flatten: HashMap<String, Box<dyn Executable>>,
    ast_node: ASTNode,
}

impl Registry {
    fn new(perf_config_items: HashMap<String, PerfConfigItem>, storage_layout: Value) -> Self {
        let ast_node = ASTNode::from_storage_layout(storage_layout);
        Self {
            perf_config_items,
            output_flatten: HashMap::new(),
            ast_node,
        }
    }

    fn set_output(&mut self, edfs: String, e: Box<dyn Executable>) {
        self.output_flatten.insert(edfs, e);
    }

    fn get_output(&self, edfs: &str) -> Option<&Box<dyn Executable>> {
        self.output_flatten.get(edfs)
    }

    fn get_output_flatten(&self) -> &HashMap<String, Box<dyn Executable>> {
        &self.output_flatten
    }

    fn get_perf_config_item(&self, edfs: String) -> Option<&PerfConfigItem> {
        self.perf_config_items.get(&edfs)
    }

    fn get_node(&self, edfs: &str) -> Option<&Node> {
        self.ast_node.get_node(edfs)
    }
}

struct Node {
    label: String,
    members: Vec<Node>,
    offset: usize,
    slot: String,
    node_type: String,
}
struct ASTNode {
    nodes: HashMap<String, Node>,
}

impl ASTNode {
    fn from_storage_layout(storage_layout: Value) -> Self {
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

    fn parse_members(types: &Value, node_type: &str) -> Vec<Node> {
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

    fn get_node(&self, edfs: &str) -> Option<&Node> {
        self.nodes.get(edfs)
    }
}

struct PerfExpressionEvaluator;

impl PerfExpressionEvaluator {
    fn eval(&self, expression: String) -> usize {
        // TODO: A very good parser 
    }
}

trait Executable {
    fn is_iterish(&self) -> bool;
    fn calculate_abs_slot(&mut self) -> ();
    fn get_edfs(&self) -> String;
    fn get_type_and_name(&self) -> String;
    fn get_type_kind(&self) -> TypeKind;
    fn enqueue_execution(&self, registry: &mut Registry);
    fn increment_step(&mut self);
    fn get_children(&self) -> Option<Vec<Box<dyn Executable>>>;
    fn get_iter(&self) -> Option<&IteratorMeta>;
    fn get_iter_mut(&mut self) -> Option<&mut IteratorMeta>;
    fn get_abs_slot(&self) -> Option<String>;
    fn get_value(&self) -> Option<String>;
    fn set_value(&mut self, value: Option<&PerfConfigItem>);
    fn get_belongs_to(&self) -> Option<&dyn Executable>;
}

impl Executable for Member {
    fn is_iterish(&self) -> bool {
        self.type_kind.is_iterish();
    }


    fn get_edfs(&self) -> String {
    }

    fn get_type_and_name(&self) -> String {
        // Implement the logic to get the type and name for a Member
        // ...
    }

    fn get_type_kind(&self) -> TypeKind {
        self.type_kind.clone()
    }

    fn get_iter(&self) -> Option<&IteratorMeta> {
        self.iter.as_ref()
    }

    fn get_iter_mut(&mut self) -> Option<&mut IteratorMeta> {
        self.iter.as_mut()
    }

    fn get_abs_slot(&self) -> Option<String> {
        self.absolute_slot.clone()
    }

    fn get_value(&self) -> Option<String> {
        self.value.clone()
    }

    fn increment_step(&mut self) {
        self.step += 1;
    }

    fn get_belongs_to(&self) -> Option<&dyn Executable> {
        self.belongs_to.as_ref()
    }

    fn enqueue_execution(&self) {
        self.registry.queue_per_step.insert(self.step, self.clone_box());
    }
    fn calculate_abs_slot(&mut self) -> () {
        // iter must have belongs_to and so logic can be shorter
        if let Some(belongs_to) = &self.belongs_to {
            if let Some(abs_slot) = belongs_to.get_abs_slot() {
                let abs_slot_num = abs_slot.parse::<usize>().unwrap();
                let relative_slot_num = self.relative_slot.parse::<usize>().unwrap();
                let combined_slot = abs_slot_num + relative_slot_num;
                self.absolute_slot = Some(format!("{:X}", combined_slot));
            } else {
                // Do nothing, as the error message suggests the function should return `()`
            }
        } else {
            // Do nothing, as the error message suggests the function should return `()`
        }
    }
    fn get_children(&self) -> Option<Vec<Box<dyn Executable>>> {
        if self.is_iterish() && self.iter.as_ref().map(|i| i.to).is_some() {
            let mut children = Vec::new();
            if let Some(iter) = &self.iter {
                for i in iter.from..iter.to.unwrap() {
                    // Find the corresponding struct and its members in the registry.ast_node
                    let ast_node = self.registry.ast_node.find_struct_by_name(format!("{}.{}", self.name, i));
                    if let Some(ast_node) = ast_node {
                        for member in ast_node.members.iter() {
                            let member_name = format!("{}.{}", self.name, member.name);
                            let member_type_kind = match member.type_kind.as_str() {
                                "t_mapping" => TypeKind::Mapping,
                                "t_array" => TypeKind::Array,
                                "t_struct" => TypeKind::NaiveStruct,
                                _ => TypeKind::Primitive,
                            };
                            let member_value_type = member.value_type.clone();
                            let member_relative_slot = member.relative_slot.clone();

                            let item = IteratorItem::new(
                                member_name,
                                member_type_kind,
                                member_value_type,
                                member_relative_slot,
                                self.clone_box(),
                                if member_type_kind.is_iterish() {
                                    Some(IteratorMeta::new(
                                        None, // key_type
                                        None, // perf_config
                                        Vec::new(), // items
                                        0, // from
                                        0, // to
                                    ))
                                } else {
                                    None
                                },
                            );
                            item.increment_step();
                            item.calculate_abs_slot();
                            self.children.push(item);
                        }
                    }

                    self.increment_step();
                    self.calculate_abs_slot();
                    self.calculate_abs_slot();
                    children.push(Box::new(self));
                }
            }
            Some(children)
        } else if self.type_kind == TypeKind::NaiveStruct {
            if let Some(ast_node) = self.ast_node.as_ref() {
                for member in ast_node.members.iter() {
                    let member_name = format!("{}.{}", self.name, member.name);
                    let member_type_kind = match member.type_kind.as_str() {
                        "t_mapping" => TypeKind::Mapping,
                        "t_array" => TypeKind::Array,
                        "t_struct" => TypeKind::NaiveStruct,
                        _ => TypeKind::Primitive,
                    };
                    let member_value_type = member.value_type.clone();
                    let member_relative_slot = member.relative_slot.clone();

                    let member = Member::new(
                        member_name,
                        member_type_kind,
                        member_value_type,
                        member_relative_slot,
                        self.clone_box(),
                        if member_type_kind.is_iterish() {
                            Some(IteratorMeta::new(
                                None, // key_type
                                None, // perf_config
                                Vec::new(), // items
                                0, // from
                                0, // to
                            ))
                        } else {
                            None
                        },

                    );
                    member.increment_step();
                    member.calculate_abs_slot();
                    children.push(Box::new(member));
                }
            }
            Some(children)
        } else {
            None
        }
    }
    fn set_value(&mut self, value: Option<&PerfConfigItem>) {
        match self.type_kind {
            TypeKind::Primitive => {
                if let Some(value) = value {
                    self.value = Some(value.to_string().as_str().to_string());
                } else {
                    self.value = None;
                }
            },
            TypeKind::Array => {
                // Set the value for an array
                if let Some(value) = value {
                    if let Some(iter) = &mut self.iter {
                        iter.to = value.to.as_ref().map(|s| s.parse().unwrap_or(0)).unwrap_or(0);
                    }
                }
            },
            TypeKind::Mapping | TypeKind::NaiveStruct => {
                // Skip setting the value for mappings and naive structs
            }
        }
    }
}

impl Executable for IteratorItem {
    fn get_edfs(&self) -> String {
        // Implement the logic to get the EDFS for a Member
        // ...
    }

    fn get_type_and_name(&self) -> String {
        // Implement the logic to get the type and name for a Member
        // ...
    }

    fn get_type_kind(&self) -> TypeKind {
        self.type_kind.clone()
    }

    fn get_iter(&self) -> Option<&IteratorMeta> {
        self.iter.as_ref()
    }

    fn get_iter_mut(&mut self) -> Option<&mut IteratorMeta> {
        self.iter.as_mut()
    }

    fn get_abs_slot(&self) -> Option<String> {
        self.absolute_slot.clone()
    }

    fn get_value(&self) -> Option<String> {
        self.value.clone()
    }

    fn increment_step(&mut self) {
        self.step += 1;
    }

    fn get_belongs_to(&self) -> Option<&dyn Executable> {
        self.belongs_to.as_ref()
    }

    fn enqueue_execution(&self) {
        self.registry.queue_per_step.insert(self.step, self.clone_box());
    }

    fn get_children(&self) -> Option<Vec<Box<dyn Executable>>> {
        if let Some(iter) = &self.iter {
            let mut children = Vec::new();
            for i in iter.from..iter.to {
                if let Some(ast_node) = &self.ast_node {
                    let item: Box<dyn Executable> = Box::new(IteratorItem::new(
                        format!("{}.{}", self.name, i),
                        iter.items[i].type_kind.clone(),
                        iter.items[i].value_type.clone(),
                        iter.items[i].relative_slot.clone(),
                        self.belongs_to.clone().unwrap(),
                        iter.items[i].mapping_key.clone(),
                        Some(ast_node.get_children(i).unwrap_or_default()),
                    ));
                    item.increment_step();
                    item.calculate_abs_slot();
                    children.push(item);
                } else {
                    // Handle the case where self.ast_node is None
                    let item: Box<dyn Executable> = Box::new(IteratorItem::new(
                        format!("{}.{}", self.name, i),
                        iter.items[i].type_kind.clone(),
                        iter.items[i].value_type.clone(),
                        iter.items[i].relative_slot.clone(),
                        self.belongs_to.clone().unwrap(),
                        iter.items[i].mapping_key.clone(),
                        None,
                    ));
                    item.increment_step();
                    item.calculate_abs_slot();
                    children.push(item);
                }
            }
            Some(children)
        } else {
            None
        }
    }

    fn set_value(&mut self, value: Option<&PerfConfigItem>) {
        match self.type_kind {
            TypeKind::Primitive => {
                if let Some(value) = value {
                    self.value = Some(value.edfs.clone());
                } else {
                    self.value = None;
                }
            },
            TypeKind::Array => {
                if let Some(value) = value {
                    if let Some(iter) = &mut self.iter {
                        iter.to = value.to.as_ref().map(|s| s.parse().unwrap_or(0)).unwrap_or(0);
                    }
                }
            },
            TypeKind::Mapping | TypeKind::NaiveStruct => {
                // Skip setting the value for mappings and naive structs
            }
        }
    }

    fn increment_step(&mut self) {
        self.step += 1;
    }

    fn calculate_abs_slot(&mut self) -> () {
        // iter must have belongs_to and so logic can be shorter
        if let Some(belongs_to) = &self.belongs_to {
            if let Some(abs_slot) = belongs_to.get_abs_slot() {
                let abs_slot_num = abs_slot.parse::<usize>().unwrap();
                let relative_slot_num = self.relative_slot.parse::<usize>().unwrap();
                let combined_slot = abs_slot_num + relative_slot_num;
                self.absolute_slot = Some(format!("{:X}", combined_slot));
            } else {
                // Do nothing, as the error message suggests the function should return `()`
            }
        } else {
            // Do nothing, as the error message suggests the function should return `()`
        }
    }

    fn clone_box(&self) -> Box<dyn Executable> {
        Box::new(self.clone())
    }
}

fn main() {
    let mut extractor = Extractor::new();
    extractor.init_members_from_compiler();
    extractor.listen();
}
