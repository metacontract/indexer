use std::collections::HashMap;
use std::fs::File;
use web3::futures::StreamExt;
use web3::types::{Address, FilterBuilder};
use std::process::Command;
use serde_json::Value;
use ethabi::{encode, Token};

enum TypeKind {
    Mapping,
    Array,
    NaiveStruct,
    Primitive,
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
    initial_members: Vec<Executable>,
    executor: Executor,
    perf_expression_evaluator: PerfExpressionEvaluator,
}

impl Extractor {
    fn new() -> Self {
        let compiler = Compiler::new("solc".to_string());
        let registry = Registry::new(HashMap::new());
        let executor = Executor::new();
        let perf_expression_evaluator = PerfExpressionEvaluator::new();

        Self {
            compiler,
            registry,
            initial_members: Vec::new(),
            executor,
            perf_expression_evaluator,
        }
    }

    fn init_members_from_compiler(&mut self) {
        let base_slots = self.compiler.prepare_base_slots().unwrap();
        let storage_layout = self.compiler.prepare_storage_layout().unwrap();

        // Create Member objects from base_slots and storage_layout
        // ...

        // Store the created Member objects in self.initial_members
        // ...
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

struct IteratorItem {
    name: String,
    type_kind: TypeKind,
    value_type: String,
    struct_index: usize,
    relative_slot: String,
    belongs_to: Option<Executable>,
    mapping_key: Option<String>,
    absolute_slot: Option<String>,
}

impl IteratorItem {
    fn new(
        name: String,
        type_kind: TypeKind,
        value_type: String,
        struct_index: usize,
        relative_slot: String,
        belongs_to: Executable,
        mapping_key: Option<String>,
    ) -> Self {
        Self {
            name,
            type_kind,
            value_type,
            struct_index,
            relative_slot,
            belongs_to: Some(belongs_to),
            mapping_key,
            absolute_slot: None,
        }
    }

    // Implement other methods for IteratorItem
    // ...
}

struct IteratorMeta {
    key_type: Option<String>,
    perf_config: Option<PerfConfigItem>,
    items: Vec<IteratorItem>,
    from: usize,
    to: usize,
}

impl IteratorMeta {
    fn set_from(&mut self, from: usize) {
        self.from = from;
    }

    fn set_to(&mut self, to: usize) {
        self.to = to;
    }

    // Implement other methods for IteratorMeta
    // ...
}

struct Member {
    name: String,
    type_kind: TypeKind,
    value_type: String,
    struct_index: usize,
    offset: usize,
    relative_slot: String,
    absolute_slot: Option<String>,
    belongs_to: Option<Executable>,
    iter: Option<IteratorMeta>,
}

impl Member {
    // Implement methods for Member
    // ...
}

struct Executor {
    queue_per_step: Vec<Vec<Executable>>,
    executed_per_step: Vec<Vec<Executable>>,
}

impl Executor {
    fn new() -> Self {
        Self {
            queue_per_step: Vec::new(),
            executed_per_step: Vec::new(),
        }
    }

    fn bulk_exec_and_enqueue_and_set_primitive_to_output(&mut self, step: usize, registry: &mut Registry, perf_expression_evaluator: &mut PerfExpressionEvaluator) {
        // Calculate absolute slot for each queued executable
        for executable in &mut self.queue_per_step[step] {
            executable.calculate_abs_slot();
        }

        // Get values by slots from EthCall
        let values = EthCall::get_values_by_slots(&self.queue_per_step[step]);

        // Flush the queue for the current step
        self.flush_queue(step);

        // Process each executed executable
        for executable in &mut self.executed_per_step[step] {
            // Get the performance configuration item for the executable
            let perf_config_item = registry.get_perf_config_item(executable.get_edfs());

            // Set the value for the executable based on the performance configuration item
            executable.set_value(perf_config_item);

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
                            let new_executable = Executable::new();
                            new_executable.enqueue_execution();
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
                            executable.enqueue_execution();
                        } else {
                            let children = executable.get_children();
                            for child in children {
                                child.enqueue_execution();
                            }
                        }

                    }
                } else {
                    // If the executable doesn't have an iterator
                    if executable.get_abs_slot().is_some() && executable.get_value().is_none() {
                        // If the executable has an absolute slot but no value, enqueue it for execution
                        executable.enqueue_execution();
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
    output_flatten: HashMap<String, Executable>,
}

impl Registry {
    fn new(perf_config_items: HashMap<String, PerfConfigItem>) -> Self {
        Self {
            perf_config_items,
            output_flatten: HashMap::new(),
        }
    }

    fn set_output(&mut self, edfs: String, e: Executable) {
        self.output_flatten.insert(edfs, e);
    }

    fn get_output(&self, edfs: &str) -> Option<&Executable> {
        self.output_flatten.get(edfs)
    }

    fn get_output_flatten(&self) -> &HashMap<String, Executable> {
        &self.output_flatten
    }

    fn get_perf_config_item(&self, edfs: String) -> Option<&PerfConfigItem> {
        self.perf_config_items.get(&edfs)
    }
}

struct PerfExpressionEvaluator;

impl PerfExpressionEvaluator {
    fn new() -> Self {
        Self
    }

    fn eval(&self, expression: String) -> usize {
        // Evaluate the performance expression and return the result
        // ...
        0
    }
}

trait BelongsTo {}

impl BelongsTo for Member {}

trait Executable {
    fn calculate_abs_slot(&mut self) -> String;
    fn get_edfs(&self) -> String;
    fn get_type_and_name(&self) -> String;
    fn get_type_kind(&self) -> TypeKind;
    fn enqueue_execution(&self);
    fn increment_step(&mut self);
    fn get_children(&self) -> Option<Vec<Box<dyn Executable>>>;
    fn get_iter(&self) -> Option<&IteratorMeta>;
    fn get_iter_mut(&mut self) -> Option<&mut IteratorMeta>;
    fn get_abs_slot(&self) -> Option<String>;
    fn get_value(&self) -> Option<String>;
    fn set_value(&mut self, value: Option<&PerfConfigItem>);
}

impl Executable for Member {
    fn increment_step(&mut self) {
        self.step += 1;
    }
    fn enqueue_execution(&self) {
        self.registry.queue_per_step.insert(self.step, self.clone_box());
    }
    fn calculate_abs_slot(&mut self) -> String {
        if let Some(belongs_to) = &self.belongs_to {
            if let Some(abs_slot) = belongs_to.get_abs_slot() {
                let abs_slot_num = abs_slot.parse::<usize>().unwrap();
                let relative_slot_num = self.relative_slot.parse::<usize>().unwrap();
                let combined_slot = abs_slot_num + relative_slot_num;
                self.absolute_slot = Some(format!("{:X}", combined_slot));
            } else {
                self.absolute_slot.as_ref().unwrap().to_string()
            }
        } else {
            self.absolute_slot.as_ref().unwrap().to_string()
        }
        self.absolute_slot.as_ref().unwrap().to_string()
    }
    fn get_children(&self) -> Option<Vec<Box<dyn Executable>>> {
        if let Some(iter) = &self.iter {
            let mut children = Vec::new();
            for i in iter.from..iter.to {
                let item = IteratorItem::new(
                    format!("{}.{}", self.name, i),
                    iter.items[i].type_kind.clone(),
                    iter.items[i].value_type.clone(),
                    iter.items[i].struct_index,
                    iter.items[i].relative_slot.clone(),
                    self.clone_box(),
                    iter.items[i].mapping_key.clone(),
                );
                item.increment_step();
                item.calculate_abs_slot();
                children.push(Box::new(item));
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
    // Note: iter can have iter as a child
    fn execute(&self, _registry: &mut Registry, _perf_expression_evaluator: &mut PerfExpressionEvaluator) -> Option<PerfConfigItem> {
        match self.type_kind {
            TypeKind::Primitive => {
                // Handle primitive types
                if let Some(value) = &self.value {
                    Some(PerfConfigItem {
                        edfs: value.to_string(),
                        from: None,
                        to: None,
                    })
                } else {
                    None
                }
            },
            TypeKind::Array => {
                // Handle arrays
                if let Some(iter) = &self.iter {
                    Some(PerfConfigItem {
                        edfs: format!("{}.{}", self.name, iter.from..iter.to),
                        from: Some(iter.from.to_string()),
                        to: Some(iter.to.to_string()),
                    })
                } else {
                    None
                }
            },
            TypeKind::Mapping | TypeKind::NaiveStruct => {
                // Handle mappings and naive structs
                None
            }
        }
    }

    fn get_children(&self) -> Option<Vec<Box<dyn Executable>>> {
        if let Some(iter) = &self.iter {
            let mut children = Vec::new();
            for i in iter.from..iter.to {
                let item = IteratorItem::new(
                    format!("{}.{}", self.name, i),
                    iter.items[i].type_kind.clone(),
                    iter.items[i].value_type.clone(),
                    iter.items[i].struct_index,
                    iter.items[i].relative_slot.clone(),
                    self.belongs_to.clone().unwrap(),
                    iter.items[i].mapping_key.clone(),
                );
                item.increment_step();
                item.calculate_abs_slot();
                children.push(Box::new(item));
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

    fn calculate_abs_slot(&mut self) -> String {
        // iter must have belongs_to and so logic can be shorter
        if let Some(belongs_to) = &self.belongs_to {
            if let Some(abs_slot) = belongs_to.get_abs_slot() {
                let abs_slot_num = abs_slot.parse::<usize>().unwrap();
                let relative_slot_num = self.relative_slot.parse::<usize>().unwrap();
                let combined_slot = abs_slot_num + relative_slot_num;
                self.absolute_slot = Some(format!("{:X}", combined_slot));
            } else {
                self.absolute_slot.as_ref().unwrap().to_string()
            }
        } else {
            self.absolute_slot.as_ref().unwrap().to_string()
        }
        self.absolute_slot.as_ref().unwrap().to_string()
    }
}

fn main() {
    let mut extractor = Extractor::new();
    extractor.init_members_from_compiler();
    extractor.listen();
}