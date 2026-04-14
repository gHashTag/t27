use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════
// Simple line-based parser for .tri format
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
struct TriField {
    name: String,
    type_val: String,
    description: String,
}

#[derive(Debug, Clone)]
struct TriType {
    name: String,
    description: String,
    fields: Vec<TriField>,
    is_enum: bool,
    enum_values: Vec<String>,
}

#[derive(Debug, Clone)]
struct TriConstant {
    name: String,
    type_val: String,
    value: String,
    description: String,
}

#[derive(Debug, Clone)]
struct TriParam {
    name: String,
    type_val: String,
    description: String,
}

#[derive(Debug, Clone)]
struct TriFunction {
    name: String,
    params: Vec<TriParam>,
    returns: String,
    description: String,
}

#[derive(Debug, Clone)]
struct TriSpec {
    name: String,
    description: String,
    types: Vec<TriType>,
    constants: Vec<TriConstant>,
    functions: Vec<TriFunction>,
    constraints: Vec<String>,
}

fn parse_tri_file(content: &str) -> Result<TriSpec> {
    let lines: Vec<&str> = content.lines().collect();
    let mut spec = TriSpec {
        name: String::new(),
        description: String::new(),
        types: Vec::new(),
        constants: Vec::new(),
        functions: Vec::new(),
        constraints: Vec::new(),
    };

    let mut i = 0;
    let mut current_type: Option<TriType> = None;
    let mut current_function: Option<TriFunction> = None;
    let mut section = String::new();

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }

        // Parse header fields
        if trimmed.starts_with("name:") {
            spec.name = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if trimmed.starts_with("description:") {
            spec.description = trimmed.split(':').nth(1).unwrap_or("")
                .trim().trim_matches('"').to_string();
        }
        // Section markers
        else if trimmed == "types:" {
            section = "types".to_string();
            current_type = None;
        } else if trimmed == "constants:" {
            section = "constants".to_string();
        } else if trimmed == "functions:" {
            section = "functions".to_string();
            current_function = None;
        } else if trimmed == "behaviors:" {
            section = "behaviors".to_string();
        } else if trimmed == "constraints:" {
            section = "constraints".to_string();
        }
        // Parse types
        else if section == "types" && indent == 2 && !trimmed.starts_with('-') && trimmed.ends_with(':') {
            if let Some(prev_type) = current_type.take() {
                spec.types.push(prev_type);
            }
            let type_name = trimmed.trim_end_matches(':').trim().to_string();
            current_type = Some(TriType {
                name: type_name.clone(),
                description: String::new(),
                fields: Vec::new(),
                is_enum: false,
                enum_values: Vec::new(),
            });
        } else if section == "types" && (indent == 4 || indent == 6) {
            if let Some(ref mut t) = current_type {
                if trimmed.starts_with("description:") {
                    t.description = trimmed.split(':').nth(1).unwrap_or("")
                        .trim().trim_matches('"').to_string();
                } else if trimmed.starts_with("fields:") {
                    // Just a marker, fields come next
                } else if trimmed.starts_with("- name:") {
                    // YAML-style field with dash
                    let name = trimmed.split("name:").nth(1).unwrap_or("").trim().to_string();
                    let mut field = TriField {
                        name,
                        type_val: "auto".to_string(),
                        description: String::new(),
                    };
                    let j = i + 1;
                    if j < lines.len() {
                        let next_line = lines[j].trim();
                        if next_line.starts_with("type:") {
                            field.type_val = next_line.split("type:").nth(1).unwrap_or("")
                                .trim().trim_matches('"').to_string();
                        }
                        let jj = j + 1;
                        if jj < lines.len() {
                            let next_next = lines[jj].trim();
                            if next_next.starts_with("description:") {
                                field.description = next_next.split("description:").nth(1).unwrap_or("")
                                    .trim().trim_matches('"').to_string();
                            }
                        }
                    }
                    t.fields.push(field);
                } else if trimmed.contains(':') && !trimmed.starts_with("description:") && !trimmed.starts_with("fields:") {
                    // Direct field declaration without dash: "name: type"
                    if let Some((field_name, field_type)) = trimmed.split_once(':') {
                        let field_name = field_name.trim().to_string();
                        let type_val = field_type.trim().to_string();
                        t.fields.push(TriField {
                            name: field_name,
                            type_val,
                            description: String::new(),
                        });
                    }
                }
            }
        }
        // Parse constants
        else if section == "constants" && indent == 2 && trimmed.ends_with(':') {
            let const_name = trimmed.trim_end_matches(':').trim().to_string();
            let mut type_val = "u32".to_string();
            let mut value = "0".to_string();
            let mut description = String::new();

            let j = i + 1;
            if j < lines.len() {
                let next_line = lines[j].trim();
                if next_line.starts_with("type:") {
                    type_val = next_line.split("type:").nth(1).unwrap_or("").trim().to_string();
                }
                let jj = j + 1;
                if jj < lines.len() {
                    let next_next = lines[jj].trim();
                    if next_next.starts_with("value:") {
                        value = next_next.split("value:").nth(1).unwrap_or("").trim().to_string();
                    }
                    let jjj = jj + 1;
                    if jjj < lines.len() {
                        let next_next_next = lines[jjj].trim();
                        if next_next_next.starts_with("description:") {
                            description = next_next_next.split("description:").nth(1).unwrap_or("")
                                .trim().trim_matches('"').to_string();
                        }
                    }
                }
            }

            spec.constants.push(TriConstant {
                name: const_name.to_uppercase(),
                type_val,
                value,
                description,
            });
        }
        // Parse functions
        else if section == "functions" && indent == 2 && trimmed.ends_with(':') {
            if let Some(prev_fn) = current_function.take() {
                spec.functions.push(prev_fn);
            }
            let fn_name = trimmed.trim_end_matches(':').trim().to_string();
            let mut function = TriFunction {
                name: fn_name,
                params: Vec::new(),
                returns: "void".to_string(),
                description: String::new(),
            };

            let j = i + 1;
            while j < lines.len() {
                let next_line = lines[j].trim();
                if next_line.is_empty() {
                    break;
                }
                if next_line.starts_with("params:") {
                    // Collect params
                    let mut jj = j + 1;
                    while jj < lines.len() {
                        let param_line = lines[jj].trim();
                        if param_line.is_empty() || !param_line.starts_with('-') {
                            break;
                        }
                        if let Some(name_part) = param_line.split("name:").nth(1) {
                            let param_name = name_part.split(',').next().unwrap_or("").trim().to_string();
                            let mut param = TriParam {
                                name: param_name,
                                type_val: "auto".to_string(),
                                description: String::new(),
                            };
                            let jjj = jj + 1;
                            if jjj < lines.len() {
                                let type_line = lines[jjj].trim();
                                if type_line.starts_with("type:") {
                                    param.type_val = type_line.split("type:").nth(1).unwrap_or("")
                                        .trim().trim_matches('"').to_string();
                                }
                            }
                            function.params.push(param);
                        }
                        jj += 1;
                    }
                    break;
                } else if next_line.starts_with("returns:") {
                    function.returns = next_line.split("returns:").nth(1).unwrap_or("void")
                        .trim().trim_matches('"').to_string();
                } else if next_line.starts_with("description:") {
                    function.description = next_line.split("description:").nth(1).unwrap_or("")
                        .trim().trim_matches('"').to_string();
                }
                break;
            }

            current_function = Some(function);
        }
        // Parse constraints
        else if section == "constraints" && trimmed.starts_with('-') {
            let constraint = trimmed.trim_start_matches('-').trim().to_string();
            spec.constraints.push(constraint);
        }

        i += 1;
    }

    // Don't forget the last item
    if let Some(prev_type) = current_type {
        spec.types.push(prev_type);
    }
    if let Some(prev_fn) = current_function {
        spec.functions.push(prev_fn);
    }

    Ok(spec)
}

// ═══════════════════════════════════════════════════════════
// Routing table (same as before)
// ═══════════════════════════════════════════════════════════

struct Route {
    target_dir: &'static str,
    target_name: &'static str,
}

fn get_route(source_path: &Path) -> Option<Route> {
    let file_name = source_path.file_stem()?.to_str()?;
    let parent = source_path.parent()?.file_name()?.to_str()?;

    match parent {
        "algo" => get_algo_route(file_name),
        "tri" => get_tri_route(file_name),
        _ => None,
    }
}

fn get_algo_route(name: &str) -> Option<Route> {
    let (target_dir, target_name) = match name {
        "relu" => ("ml/activation", "relu_activation"),
        "sigmoid" => ("ml/activation", "sigmoid_activation"),
        "tanh" => ("ml/activation", "tanh_activation"),
        "gelu" => ("ml/activation", "gelu_activation"),
        "gelu_approx" => ("ml/activation", "gelu_approx_activation"),
        "elu" => ("ml/activation", "elu_activation"),
        "leaky_relu" => ("ml/activation", "leaky_relu_activation"),
        "silu_swish" => ("ml/activation", "silu_swish_activation"),
        "silu_swish_vbt" => ("ml/activation", "silu_swish_vbt_activation"),
        "softmax" => ("ml/activation", "softmax"),
        "dense" => ("ml/layers", "dense_layer"),
        "conv2d" => ("ml/layers", "conv2d_layer"),
        "maxpool2d" => ("ml/layers", "maxpool2d_layer"),
        "avgpool2d" => ("ml/layers", "avgpool2d_layer"),
        "flatten" => ("ml/layers", "flatten_layer"),
        "embedding" => ("ml/layers", "embedding_layer"),
        "batchnorm" => ("ml/layers", "batchnorm_layer"),
        "layernorm" => ("ml/layers", "layernorm_layer"),
        "dropout" => ("ml/layers", "dropout_layer"),
        "residual" => ("ml/layers", "residual_connection"),
        "lstm" => ("ml/recurrent", "lstm_cell"),
        "lstm_cell" => ("ml/recurrent", "lstm_single"),
        "gru" => ("ml/recurrent", "gru_cell"),
        "rnn_cell" => ("ml/recurrent", "rnn_cell"),
        "bilstm" => ("ml/recurrent", "bilstm"),
        "seq2seq" => ("ml/recurrent", "seq2seq"),
        "attention" => ("ml/recurrent", "attention_mechanism"),
        "self_attention" => ("ml/recurrent", "self_attention"),
        "multi_head_attn" => ("ml/transformer", "multi_head_attention"),
        "positional_enc" => ("ml/transformer", "positional_encoding"),
        "feed_forward" => ("ml/transformer", "feed_forward_network"),
        "encoder_block" => ("ml/transformer", "encoder_block"),
        "sgd" => ("ml/optimizer", "sgd"),
        "sgd_momentum" => ("ml/optimizer", "sgd_momentum"),
        "adam" => ("ml/optimizer", "adam"),
        "adamw" => ("ml/optimizer", "adamw"),
        "rmsprop" => ("ml/optimizer", "rmsprop"),
        "adagrad" => ("ml/optimizer", "adagrad"),
        "lamb" => ("ml/optimizer", "lamb"),
        "scheduler" => ("ml/optimizer", "lr_scheduler"),
        "mse_loss" => ("ml/loss", "mse_loss"),
        "cross_entropy" => ("ml/loss", "cross_entropy_loss"),
        "binary_ce" => ("ml/loss", "binary_crossentropy_loss"),
        "huber_loss" => ("ml/loss", "huber_loss"),
        "kl_divergence" => ("ml/loss", "kl_divergence"),
        "contrastive_loss" => ("ml/loss", "contrastive_loss"),
        "dqn" => ("ml/rl", "dqn"),
        "dqn_target" => ("ml/rl", "dqn_target_network"),
        "ppo_actor" => ("ml/rl", "ppo_actor"),
        "ppo_critic" => ("ml/rl", "ppo_critic"),
        "ppo_clip_loss" => ("ml/rl", "ppo_clip_loss"),
        "sac_actor" => ("ml/rl", "sac_actor"),
        "sac_critic" => ("ml/rl", "sac_critic"),
        "advantage" => ("ml/rl", "advantage_estimator"),
        "mlp" => ("ml/pathway", "mlp"),
        _ => return None,
    };
    Some(Route { target_dir, target_name })
}

fn get_tri_route(name: &str) -> Option<Route> {
    let (target_dir, target_name) = match name {
        "tri_list" => ("tri/collections", "list"),
        "tri_map" => ("tri/collections", "map"),
        "tri_set" => ("tri/collections", "set"),
        "tri_queue" => ("tri/collections", "queue"),
        "tri_stack" => ("tri/collections", "stack"),
        "tri_deque" => ("tri/collections", "deque"),
        "tri_linked_list" => ("tri/collections", "linked_list"),
        "tri_array" => ("tri/collections", "array"),
        "tri_bitmap" => ("tri/collections", "bitmap"),
        "tri_bitset" => ("tri/collections", "bitset"),
        "tri_bitvector" => ("tri/collections", "bitvector"),
        "tri_ring" => ("tri/collections", "ring_buffer"),
        "tri_circular_buffer" => ("tri/collections", "circular_buffer"),
        "tri_tuple" => ("tri/collections", "tuple"),
        "tri_option" => ("tri/collections", "option"),
        "tri_result" => ("tri/collections", "result"),
        "tri_either" => ("tri/collections", "either"),
        "tri_maybe" => ("tri/collections", "maybe"),
        "tri_variant" => ("tri/collections", "variant"),
        "tri_btree" => ("tri/collections", "btree"),
        "tri_skip_list" => ("tri/collections", "skip_list"),
        "tri_lru_cache" => ("tri/collections", "lru_cache"),
        "tri_lru" => ("tri/collections", "lru"),
        "tri_lockfree_stack" => ("tri/collections", "lockfree_stack"),
        "tri_vector" => ("tri/collections", "vector"),
        "tri_interval" => ("tri/collections", "interval"),
        "tri_namespace" => ("tri/collections", "namespace"),
        "tri_context" => ("tri/collections", "context"),
        "tri_state" => ("tri/collections", "state"),
        "tri_priority_queue" => ("tri/collections", "priority_queue"),
        "tri_avl_tree" => ("tri/trees", "avl_tree"),
        "tri_b_tree" => ("tri/trees", "b_tree"),
        "tri_rb_tree" => ("tri/trees", "red_black_tree"),
        "tri_splay_tree" => ("tri/trees", "splay_tree"),
        "tri_kd_tree" => ("tri/trees", "kd_tree"),
        "tri_quadtree" => ("tri/trees", "quadtree"),
        "tri_octree" => ("tri/trees", "octree"),
        "tri_rtree" => ("tri/trees", "rtree"),
        "tri_segment_tree" => ("tri/trees", "segment_tree"),
        "tri_fenwick" => ("tri/trees", "fenwick_tree"),
        "tri_suffix_array" => ("tri/trees", "suffix_array"),
        "tri_trie" => ("tri/trees", "trie"),
        "tri_tree" => ("tri/trees", "tree"),
        "tri_quick_sort" => ("tri/sort", "quick_sort"),
        "tri_merge_sort" => ("tri/sort", "merge_sort"),
        "tri_heap_sort" => ("tri/sort", "heap_sort"),
        "tri_insertion_sort" => ("tri/sort", "insertion_sort"),
        "tri_selection_sort" => ("tri/sort", "selection_sort"),
        "tri_shell_sort" => ("tri/sort", "shell_sort"),
        "tri_counting_sort" => ("tri/sort", "counting_sort"),
        "tri_radix_sort" => ("tri/sort", "radix_sort"),
        "tri_tim_sort" => ("tri/sort", "tim_sort"),
        "tri_sort" => ("tri/sort", "sort"),
        "tri_graph" => ("tri/graph", "graph"),
        "tri_graph_bfs" => ("tri/graph", "graph_bfs"),
        "tri_graph_dfs" => ("tri/graph", "graph_dfs"),
        "tri_dijkstra" => ("tri/graph", "dijkstra"),
        "tri_bellman_ford" => ("tri/graph", "bellman_ford"),
        "tri_disjoint_set" => ("tri/graph", "disjoint_set"),
        "tri_topological" => ("tri/graph", "topological_sort"),
        "tri_prims_mst" => ("tri/graph", "prims_mst"),
        "tri_bloom_filter" => ("tri/search", "bloom_filter"),
        "tri_kmp" => ("tri/search", "knuth_morris_pratt"),
        "tri_boyer_moore" => ("tri/search", "boyer_moore"),
        "tri_rabin_karp" => ("tri/search", "rabin_karp"),
        "tri_aho_corasick" => ("tri/search", "aho_corasick"),
        "tri_search" => ("tri/search", "search"),
        "tri_pattern" => ("tri/search", "pattern"),
        "tri_match" => ("tri/search", "match"),
        "tri_regex" => ("tri/search", "regex"),
        "tri_regex_advanced" => ("tri/search", "regex_advanced"),
        "tri_sha256" => ("tri/crypto", "sha256"),
        "tri_hmac" => ("tri/crypto", "hmac"),
        "tri_ecc" => ("tri/crypto", "ecc"),
        "tri_rsa" => ("tri/crypto", "rsa"),
        "tri_base64" => ("tri/crypto", "base64"),
        "tri_base32" => ("tri/crypto", "base32"),
        "tri_hex" => ("tri/crypto", "hex"),
        "tri_crypto" => ("tri/crypto", "crypto"),
        "tri_reed_solomon" => ("tri/crypto", "reed_solomon"),
        "tri_json" => ("tri/encoding", "json"),
        "tri_xml" => ("tri/encoding", "xml"),
        "tri_csv" => ("tri/encoding", "csv"),
        "tri_bson" => ("tri/encoding", "bson"),
        "tri_msgpack" => ("tri/encoding", "msgpack"),
        "tri_html" => ("tri/encoding", "html"),
        "tri_markup" => ("tri/encoding", "markup"),
        "tri_mime" => ("tri/encoding", "mime"),
        "tri_http" => ("tri/net", "http"),
        "tri_net" => ("tri/net", "net"),
        "tri_websocket" => ("tri/net", "websocket"),
        "tri_url" => ("tri/net", "url"),
        "tri_channel" => ("tri/net", "channel"),
        "tri_async" => ("tri/net", "async"),
        "tri_async_stream" => ("tri/net", "async_stream"),
        "tri_cloud" => ("tri/net", "cloud"),
        "tri_fs" => ("tri/io", "fs"),
        "tri_filesystem" => ("tri/io", "filesystem"),
        "tri_io" => ("tri/io", "io"),
        "tri_reader" => ("tri/io", "reader"),
        "tri_writer" => ("tri/io", "writer"),
        "tri_compress" => ("tri/io", "compress"),
        "tri_zipper" => ("tri/io", "zip"),
        "tri_math" => ("tri/math", "math"),
        "tri_statistics" => ("tri/math", "statistics"),
        "tri_matrix" => ("tri/math", "matrix"),
        "tri_polynomial" => ("tri/math", "polynomial"),
        "tri_bezier" => ("tri/math", "bezier"),
        "tri_probability" => ("tri/math", "probability"),
        "tri_measurement" => ("tri/math", "measurement"),
        "tri_constants" => ("tri/math", "constants"),
        "tri_cli" => ("tri/utils", "cli"),
        "tri_config" => ("tri/utils", "config"),
        "tri_logging" => ("tri/utils", "logging"),
        "tri_logger" => ("tri/utils", "logger"),
        "tri_time" => ("tri/utils", "time"),
        "tri_arrow_time" => ("tri/utils", "arrow_time"),
        "tri_terminal" => ("tri/utils", "terminal"),
        "tri_help" => ("tri/utils", "help"),
        "tri_args" => ("tri/utils", "args"),
        "tri_string" => ("tri/utils", "string"),
        "tri_text" => ("tri/utils", "text"),
        "tri_bytes" => ("tri/utils", "bytes"),
        "tri_utf8" => ("tri/utils", "utf8"),
        "tri_color" => ("tri/utils", "color"),
        "tri_colors" => ("tri/utils", "colors"),
        "tri_error" => ("tri/utils", "error"),
        "tri_exit_codes" => ("tri/utils", "exit_codes"),
        "tri_version" => ("tri/utils", "version"),
        "tri_random" => ("tri/utils", "random"),
        "tri_template" => ("tri/utils", "template"),
        "batch_runner" => ("tri/pipeline", "batch_runner"),
        "cloud_orchestrator" => ("tri/pipeline", "cloud_orchestrator"),
        "workflow" => ("tri/pipeline", "workflow"),
        "workflow_executor" => ("tri/pipeline", "workflow_executor"),
        "workflow_parser" => ("tri/pipeline", "workflow_parser"),
        "tri_pipeline" => ("tri/pipeline", "pipeline"),
        "tri_pipeline_parallel" => ("tri/pipeline", "pipeline_parallel"),
        "tri_spec_parser" => ("tri/pipeline", "spec_parser"),
        "tri_spec_writer" => ("tri/pipeline", "spec_writer"),
        "tri_builder" => ("tri/pipeline", "builder"),
        "codegen_engine_full_upgrade" => ("tri/pipeline", "codegen"),
        "agents" => ("tri/agent", "agents"),
        "autonomous_lifecycle" => ("tri/agent", "autonomous_lifecycle"),
        "autonomous_universe" => ("tri/agent", "autonomous_universe"),
        "eternal_monitor" => ("tri/agent", "eternal_monitor"),
        "tri_agent_run" => ("tri/agent", "agent_run"),
        "swarm_agents" => ("tri/agent", "swarm_agents"),
        "faculty_board" => ("tri/agent", "faculty_board"),
        "handoff" => ("tri/agent", "handoff"),
        "memory" => ("tri/agent", "memory"),
        "experience_hooks" => ("tri/agent", "experience_hooks"),
        "governance_agent" => ("tri/agent", "governance_agent"),
        "sacred_constants" => ("sacred", "sacred_constants"),
        "sacred_identity" => ("sacred", "sacred_identity"),
        "sacred_governance" => ("sacred", "sacred_governance"),
        "tri_gravity" => ("sacred", "gravity"),
        "tri_cosmology" => ("sacred", "cosmology"),
        "tri_dark_matter" => ("sacred", "dark_matter"),
        "tri_quantum" => ("sacred", "quantum"),
        "tri_quantum_gravity" => ("sacred", "quantum_gravity"),
        "tri_superconductivity" => ("sacred", "superconductivity"),
        "tri_monopoles" => ("sacred", "monopoles"),
        _ => return None,
    };
    Some(Route { target_dir, target_name })
}

// ═══════════════════════════════════════════════════════════
// .t27 Generator
// ═══════════════════════════════════════════════════════════

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

fn convert_type_name(tri_type: &str) -> String {
    match tri_type {
        "f32" => "f32".to_string(),
        "f64" => "f64".to_string(),
        "i32" => "i32".to_string(),
        "u32" => "u32".to_string(),
        "i64" => "i64".to_string(),
        "u64" => "u64".to_string(),
        "i8" => "i8".to_string(),
        "u8" => "u8".to_string(),
        "u16" => "u16".to_string(),
        "usize" => "usize".to_string(),
        "bool" => "bool".to_string(),
        "void" => "void".to_string(),
        t if t.starts_with("?") => {
            format!("?{}", convert_type_name(&t[1..]))
        }
        t if t.starts_with("[]const ") => {
            format!("[]const {}", convert_type_name(&t[8..]))
        }
        t if t.starts_with("[]") => {
            format!("[]{}", convert_type_name(&t[2..]))
        }
        t if t.contains('[') => {
            // Array syntax like [16]u8
            if let Some(end) = t.split(']').nth(1) {
                format!("[{}]{}", &t[1..t.find(']').unwrap_or(t.len())], convert_type_name(end))
            } else {
                to_pascal_case(tri_type)
            }
        }
        _ => to_pascal_case(tri_type),
    }
}

fn generate_t27(spec: &TriSpec) -> String {
    let module_name = to_pascal_case(&spec.name);
    let mut output = String::new();

    // Header
    output.push_str("// SPDX-License-Identifier: Apache-2.0\n");
    output.push_str("// t27/specs/\n");
    output.push_str(&format!("// {} | φ² + 1/φ² = 3 | TRINITY\n", spec.description));
    output.push_str("\n");

    output.push_str(&format!("module {};\n", module_name));
    output.push_str("    use base::types;\n");
    output.push_str("    use math::constants;\n\n");

    // Constants
    if !spec.constants.is_empty() {
        output.push_str("    // ═══════════════════════════════════════════════════════════\n");
        output.push_str("    // 1. Constants\n");
        output.push_str("    // ═══════════════════════════════════════════════════════════\n\n");

        for constant in &spec.constants {
            let type_name = convert_type_name(&constant.type_val);
            output.push_str(&format!("    const {} : {} = {};\n",
                constant.name, type_name, constant.value));
        }
        output.push('\n');
    }

    // Types
    if !spec.types.is_empty() {
        output.push_str("    // ═══════════════════════════════════════════════════════════\n");
        output.push_str("    // 2. Types\n");
        output.push_str("    // ═══════════════════════════════════════════════════════════\n\n");

        for tri_type in &spec.types {
            let pascal_name = to_pascal_case(&tri_type.name);
            output.push_str(&format!("    pub const {} = struct {{\n", pascal_name));
            for field in &tri_type.fields {
                let field_type = convert_type_name(&field.type_val);
                output.push_str(&format!("        {} : {},\n", field.name, field_type));
            }
            output.push_str("    };\n\n");
        }
    }

    // Functions
    if !spec.functions.is_empty() {
        output.push_str("    // ═══════════════════════════════════════════════════════════\n");
        output.push_str("    // 3. Core Functions\n");
        output.push_str("    // ═══════════════════════════════════════════════════════════\n\n");

        for func in &spec.functions {
            let fn_name_snake = to_snake_case(&func.name);
            let mut params_str = String::new();
            for (i, param) in func.params.iter().enumerate() {
                if i > 0 {
                    params_str.push_str(", ");
                }
                let param_type = convert_type_name(&param.type_val);
                params_str.push_str(&format!("{}: {}", param.name, param_type));
            }

            let return_type = convert_type_name(&func.returns);

            output.push_str(&format!("    // {}({}) → {}\n", fn_name_snake, params_str, return_type));
            output.push_str(&format!("    fn {}({}) -> {} {{\n", fn_name_snake, params_str, return_type));
            output.push_str("        // TODO: Implement from .tri spec\n");
            output.push_str("    }\n\n");
        }
    }

    // TDD: Tests
    output.push_str("    // ═══════════════════════════════════════════════════════════\n");
    output.push_str("    // TDD: Tests (from .tri behaviors)\n");
    output.push_str("    // ═══════════════════════════════════════════════════════════\n\n");

    for func in &spec.functions {
        let test_name = format!("{}_basic_case", to_snake_case(&func.name));
        let fn_name_snake = to_snake_case(&func.name);

        output.push_str(&format!("    test {}\n", test_name));
        output.push_str(&format!("        given input = default_input()\n"));
        output.push_str(&format!("        when result = {}(input)\n", fn_name_snake));
        output.push_str(&format!("        then result != undefined\n\n"));
    }

    // TDD: Invariants (from constraints)
    if !spec.constraints.is_empty() {
        output.push_str("    // ═══════════════════════════════════════════════════════════\n");
        output.push_str("    // TDD: Invariants (from .tri constraints)\n");
        output.push_str("    // ═══════════════════════════════════════════════════════════\n\n");

        for (i, constraint) in spec.constraints.iter().enumerate() {
            let inv_name = format!("{}_constraint_{}", to_snake_case(&spec.name), i);
            output.push_str(&format!("    invariant {}\n", inv_name));
            output.push_str(&format!("        given input = valid_input()\n"));
            output.push_str(&format!("        then {} // {}\n\n", "true", constraint));
        }
    }

    output
}

// ═══════════════════════════════════════════════════════════
// Main entry point
// ═══════════════════════════════════════════════════════════

fn convert_file(source_path: &Path, target_base: &Path) -> Result<()> {
    let route = get_route(source_path)
        .ok_or_else(|| anyhow::anyhow!("No route found for: {:?}", source_path))?;

    let content = fs::read_to_string(source_path)
        .with_context(|| format!("Failed to read: {:?}", source_path))?;

    let spec = parse_tri_file(&content)?;

    let t27_content = generate_t27(&spec);

    let target_dir = target_base.join(route.target_dir);
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("Failed to create dir: {:?}", target_dir))?;

    let target_path = target_dir.join(format!("{}.t27", route.target_name));

    fs::write(&target_path, t27_content)
        .with_context(|| format!("Failed to write: {:?}", target_path))?;

    println!("Converted: {:?} -> {:?}", source_path.file_name(), target_path);

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <source-dir> <target-dir>", args[0]);
        eprintln!("Example: {} /Users/playra/trinity-w1/specs /Users/playra/t27/specs", args[0]);
        std::process::exit(1);
    }

    let source_dir = PathBuf::from(&args[1]);
    let target_dir = PathBuf::from(&args[2]);

    println!("Starting migration: {:?} -> {:?}", source_dir, target_dir);

    let mut converted = 0;
    let mut skipped = 0;

    // Process algo files
    let algo_path = source_dir.join("algo");
    if algo_path.exists() {
        for entry in fs::read_dir(&algo_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("tri") {
                match convert_file(&path, &target_dir) {
                    Ok(_) => converted += 1,
                    Err(e) => {
                        eprintln!("Skipping {:?}: {}", path.file_name(), e);
                        skipped += 1;
                    }
                }
            }
        }
    }

    // Process tri files
    let tri_path = source_dir.join("tri");
    if tri_path.exists() {
        for entry in fs::read_dir(&tri_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("tri") {
                match convert_file(&path, &target_dir) {
                    Ok(_) => converted += 1,
                    Err(e) => {
                        eprintln!("Skipping {:?}: {}", path.file_name(), e);
                        skipped += 1;
                    }
                }
            }
        }
    }

    println!("\nMigration complete:");
    println!("  Converted: {} files", converted);
    println!("  Skipped: {} files", skipped);

    Ok(())
}
