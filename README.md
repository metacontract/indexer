# metacontract/indexer
- A mc-companion embedded indexer.
- Solidity-esque low-config portable binary program written in Rust.
- Your Schema.sol will automatically be GraphQL API (backend: Seaography-SQLite)
- You can run it in your frontend server or Cloudflare D1 instance (edge computing for low-latency querying).
- Scale until "Your RAM Size > Your Dapp Indexer Size" with in-memory SQLite setting.

# Diagrams
```:mermaid
classDiagram
    class Compiler {
        +solc_path: String
        +base_slot_ast_cache: Option<String>
        +storage_layout_ast_cache: Option<String>
        +new(solc_path: String) -> Self
        +prepare_base_slots() -> Result<Value, Box<dyn std::error::Error>>
        +prepare_slorage_layout() -> Result<Value, Box<dyn std::error::Error>>
    }

    class Extractor {
        +compiler: Compiler
        +file: File
        +evm: EVM
        +eth_call: EthCall
        +registry: Registry
        +initial_members: Executable[]
        +new() -> Self
        +init_members_from_compiler()
        +listen()
        +scan_contract()
    }


    class TypeKind {
        <<enumeration>>
        Primitive
        NaiveStruct
        Array
        Mapping
    }
    class Executable {
        +name:String
        +step: usize
        +type_kind: TypeKind
        +value_type: String
        +belongs_to: Option<Executable>
        +offset: usize
        +relative_slot: String
        +absolute_slot: Option<String>
        +executor: Executor
        +calculat_abs_slot(): String
        +get_edfs():String
        +get_type_and_name() -> String
        +enqueue_execution()
        +increment_step()
        +get_children(ast_node:String): Option<Vec<Executable>>
    }

    class IteratorItem {
        +mapping_key: String
        +new(name: String, type_kind: TypeKind, value_type: String, struct_index: usize, relative_slot: String, belongs_to: Executable, mapping_key: String) -> Self
    }
    class Member {
        +iter: Option<IteratorMeta>
    }
    class IteratorMeta {
        +keyType: String
        +from: Option<usize>
        +to: Option<usize>
        +set_from(parsed_from:usize)
        +set_to(parsed_to:usize)
    }
    class Executor {
        +queue_per_step: Vec<Vec<Executable>>
        +executed_per_step: Vec<Vec<Executable>>
        +perf_expression_evaluator: PerfExpressionEvaluator
        +bulk_exec_and_enqueue_and_set_primitive_to_output(step:usize)
        +flush_queue(step:usize)
        +flush_executed(step:usize)
    }

    class Registry {
        +perf_config_items: HashMap<String, PerfConfigItem>
        +output_flatten: HashMap<String, Executable>
        +set_output(edfs:String, e: Executable)
        +get_output(edfs: &str) -> Option<&Executable>
        +get_output_flatten() -> &Vec<Executable>
        +new(perf_config_items: HashMap<String, PerfConfigItem>) -> Self
        +get_perf_config_item(edfs:String): PerfConfigItem
    }
    class PerfConfigItem {
        +edfs: String,
        +from: Option<String>,
        +to: Option<String>,
        +from_executed: Option<usize>,
        +to_executed: Option<usize>
    }
    class PerfExpressionEvaluator {
        +eval(expression:String) -> usize
    }

    Main --> Extractor
    Extractor --> Compiler
    Extractor --> Registry
    Extractor --> Member
    Member <--> IteratorItem
    Registry <-- PerfConfigItem
    Member<--IteratorMeta
    Executor-->PerfExpressionEvaluator
    Executable-->TypeKind
    Executor --> Registry
    Executor --> Member
    Executor --> IteratorItem
    Member <|-- Executable
    IteratorItem <|-- Executable


```

```:mermaid
sequenceDiagram
    participant Main
    participant Extractor
    participant Compiler
    participant Registry
    participant Executable as Executable (Member/IteratorItem)
    participant Executor

    Main->>Extractor: new()
    activate Extractor

    Extractor->>Compiler: new(solc_path)
    activate Compiler
    Compiler-->>Extractor: compiler
    deactivate Compiler

    Extractor->>Extractor: init_members_from_compiler()
    activate Extractor
    Extractor->>Compiler: prepare_base_slots()
    activate Compiler
    Compiler-->>Extractor: base_slots
    deactivate Compiler
    Extractor->>Compiler: prepare_slorage_layout()
    activate Compiler
    Compiler-->>Extractor: storage_layout
    deactivate Compiler
    Extractor->>Extractor: create Member objects from base_slots and storage_layout
    Extractor->>Registry: new(perf_config_items)
    activate Registry
    Registry-->>Extractor: registry
    deactivate Registry
    Extractor-->>Extractor: initial_members
    deactivate Extractor

    Main->>Extractor: listen()
    activate Extractor
    Extractor->>Extractor: scan_contract()
    activate Extractor

    %% step = 0
    loop for each step
        Extractor->>Executor: bulk_exec_and_enqueue_and_set_primitive_to_output(step)
        activate Executor

        loop for each queued executables
            Executor->>Executable: calculate_abs_slot()
            activate Executable
            Executable-->>Executor: abs_slot
            deactivate Executable
        end

        Executor-->>EthCall: get values by slots
        EthCall-->>Executor: values
        Executor-->>Executor: flush_queue(step)

        loop for each executed
            Executor->>Registry: get_perf_config_item(edfs: String)
            activate Registry
            Registry-->>Executor: perf_config_item 
            deactivate Registry

            %% For Mapping, skip. Because it doesn't have value in the slot. iter.to will be filled a logic below.
            %% For NaiveStruct, skip. Becuase 1st child is overlapping the slot.
            %% In the logic of set_value()
            %%   For Primitive, self.value = value
            %%   For Array, self.iter.to = value
            Executor-->>Executable: set_value
            activate Executable
            Executable-->>Executor: 
            deactivate Executable

            critical executable.typeKind == TypeKind.Primitive
                option true
                    Executor->>Registry: push_output(executable)
                    activate Registry
                    Registry-->>Executor: 
                    deactivate Registry
                    %% Executed primitive must be recorded.
                option false
                    %% Executed NaiveStruct, Array, Mapping
                    Executor->>Executable: get_children(ast_node)
                    activate Executable
                    Executable-->>Executor: Executables
                    deactivate Executable

                    %% enqueueability check for one-step below executables
                    Executor->>Registry: get_perf_config_item(edfs: String)
                    activate Registry
                    Registry-->>Executor: perf_config_item 
                    deactivate Registry

                    critical !executable.iter
                        option false
                            critical !!executable.iter.to
                                option true
                                    loop i in executable.iter.to
                                        Executor->>Executor: Executable.new()
                                        Executor->>Executable: enqueue_execution()
                                        activate Executable
                                        Executable-->>Executor: 
                                        deactivate Executable
                                    end
                                option false
                                    %% trying to fill iter.{from,to}
                                    %% Array: Just have been set executed value
                                    %% Mapping: with eval-ing conf
                                    Executor->>PerfExpressionEvaluator: eval(from_expression:String)
                                    activate PerfExpressionEvaluator
                                    PerfExpressionEvaluator-->>Executor: parsed_from
                                    deactivate PerfExpressionEvaluator
                                    Executor->>PerfExpressionEvaluator: eval(to_expression:String)
                                    activate PerfExpressionEvaluator
                                    PerfExpressionEvaluator-->>Executor: parsed_to
                                    Executor-->>IteratorMeta: set_from(parsed_from)
                                    Executor-->>IteratorMeta: set_to(parsed_to)
                                    deactivate PerfExpressionEvaluator

                                    %% Skipping algo for a mapping's unloaded bin_index
                                    critical executable.iter.to > 0
                                        option false
                                            Executor->>Executable: increment_step()
                                            Executor->>Executable: enqueue_execution()
                                            activate Executable
                                            Executable-->>Executor: 
                                            deactivate Executable
                                            %% if perf_config_item is not found, enqueue the executable for the next step
                                    end
                            end
                        option true
                            critical !!executable.abs_slot && !executable.value
                                option true
                                    Executor->>Executable: enqueue_execution()
                                    activate Executable
                                    Executable-->>Executor: 
                                    deactivate Executable
                                %% "option false" case means this executable should simply be value-filled on next bulk exec.                        
                            end
                    end

            end


                Executor->>Registry: check_primitive_output(edfs_from_perf_config_item: String)
                activate Registry
                Registry-->>Executor: perf_conf_specified_executable: Option<Executable>
                deactivate Registry

                critical perf_conf_specified_executable
                    option Some(perf_conf_specified_executable)
                        Executor-->>Executable: set_value
                        activate Executable
                        Executable-->>Executor: 
                        deactivate Executable

                        Executor->>Executable: increment_step()
                        Executable->>Executable: enqueue_execution()
                        Executable->>Executor: 
                end


            Executor-->>Extractor: 
            deactivate Executor
        end

        Executor-->>Executor: flush_executed(step)
        %% step += 1;

        critical step > 15
            option true
                Executor-->>Executor: break;
                %% 2^4 nest is the limit for safety and efficiency
                %% malformed perf_config causes infinite loop due to enqueue skipping algo 
        end
    end

    Extractor->>Registry: get_output_flatten()
    activate Registry
    Registry-->>Extractor: output_flatten
    deactivate Registry
    Extractor-->>Main: output_flatten
    deactivate Extractor

    deactivate Extractor
```
