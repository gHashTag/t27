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

/// Is the right-hand side of `- tag: rhs` a TYPE, or a value?
///
/// The distinction decides whether a variant becomes a `union(enum)` payload
/// or stays a plain field. `0` and `"OK"` are values; `ParsedCommand`,
/// `[]const u8` and `i64` are types. Getting this wrong in the permissive
/// direction emits `success : 0`, which does not compile.
fn is_payload_type(rhs: &str) -> bool {
    let r = rhs.trim();
    // A QUOTED right-hand side is a value, not a type. Stripping the quotes
    // first made `- status: "OK"` read as the type `OK` and emit
    // `status : OK` inside a union(enum) -- a payload whose type does not
    // exist. Across all 366 ancestors not one variant bullet quotes its type
    // (`- sacred_score: SacredScoreData`, `- string: []const u8`), so this
    // costs nothing on real input and closes the case that does not.
    if r.starts_with('"') || r.starts_with('\'') {
        return false;
    }
    if r.is_empty() || r.parse::<i64>().is_ok() || r.parse::<f64>().is_ok() {
        return false;
    }
    r.starts_with("[]")
        || r.starts_with('*')
        || r.starts_with('?')
        || r.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        || matches!(
            r,
            "bool" | "void" | "usize" | "isize" | "f32" | "f64"
                | "i8" | "i16" | "i32" | "i64"
                | "u8" | "u16" | "u32" | "u64"
        )
}

#[derive(Debug, Clone)]
struct TriType {
    name: String,
    description: String,
    fields: Vec<TriField>,
    is_enum: bool,
    enum_values: Vec<String>,
    /// Variants that carry a payload: `- namespaced: ParsedCommand`.
    /// Bare variants live in `enum_values`; a type with any of these emits as
    /// `union(enum)` rather than as a variant list.
    enum_payloads: Vec<(String, String)>,
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
                enum_payloads: Vec::new(),
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
                } else if trimmed == "enum:"
                    || trimmed == "variants:"
                    || trimmed == "union:"
                    || trimmed == "cases:"
                {
                    // A VARIANT LIST, not a field.
                    //
                    // The ancestor writes
                    //
                    //     LogLevel:
                    //       enum:
                    //         - debug
                    //         - info
                    //
                    // and until now `enum:` fell through to the direct-field
                    // branch below: it contains a colon, so split_once gave the
                    // field name `enum` and an EMPTY type. That is literally
                    // the `enum : ,` seen in 31 converted specs -- and the
                    // bullets beneath it match no branch at all, having no
                    // colon of their own, so every variant name was dropped in
                    // silence.
                    //
                    // specs/tri/utils/exit_codes.t27 is why this went unnoticed
                    // for four months: ITS bullets are written `- success: 0`,
                    // with a value, so they parsed as fields and survived. The
                    // bug only fires on a bare bullet.
                    //
                    // `variants:` is the SAME shape under a different key, and
                    // is the commoner of the two upstream: 41 blocks against
                    // enum's 31. Only 3 reached this corpus, because most of
                    // their ancestors have no route (#2723) -- so the count
                    // that matters for a future conversion is 41, not 3.
                    //
                    // All four keys the ancestors use are handled here, and
                    // the earlier claim that `union:`/`cases:` "carry nested
                    // maps this grammar cannot express" was simply wrong --
                    // measured across all 366 ancestors, there is not one
                    // nested map under any of them:
                    //
                    //     key        blocks  bare  payload
                    //     enum:          31   162        0
                    //     variants:      43   204       12
                    //     union:          2     0       15
                    //     cases:          2    10        0
                    //
                    // `union:` needs one guard that is not visible here: the
                    // same word is also a FUNCTION name in tri_bitset.tri and
                    // tri_disjoint_set.tri, where `union:` at indent 2 is a
                    // method with `params:`/`returns:`. Counting by the text
                    // `union:` alone reports 6 blocks and 27 pairs; 4 of those
                    // blocks are the homonym and 12 of the pairs are its
                    // parameters. This arm is reachable only under
                    // `section == "types"` at indent 4 or 6, so the method
                    // cannot arrive here -- but any future count of these
                    // shapes must ask WHERE the key sits, not just what it
                    // says.
                    t.is_enum = true;
                } else if t.is_enum && trimmed.starts_with("- ") {
                    // Strip a trailing `# comment`. Without this,
                    // `- warning      # Minor violation, logged but not
                    // blocking` becomes a variant carrying the whole comment,
                    // and the comma inside it splits off a second bogus
                    // variant. Caught by cross-checking this converter's
                    // output against a hand recovery of the same ancestors:
                    // 16 of 18 declarations matched, and the one that differed
                    // was mine, not theirs.
                    let value = trimmed[2..]
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if value.is_empty() {
                        // nothing but a comment
                    } else if let Some((tag, rhs)) = value.split_once(':') {
                        // A PAYLOAD, or a value assignment wearing the same
                        // punctuation. `- namespaced: ParsedCommand` is the
                        // first; `- success: 0` in exit_codes.tri is the
                        // second, and it must keep falling through to the
                        // field branch below -- emitting `success : 0` inside
                        // a union(enum) would be invalid Zig, and that spec's
                        // declaration is correct today precisely because this
                        // arm does not claim it.
                        let (tag, rhs) = (tag.trim(), rhs.trim().trim_matches('"'));
                        if is_payload_type(rhs) {
                            t.enum_payloads.push((tag.to_string(), rhs.to_string()));
                        } else {
                            t.fields.push(TriField {
                                name: tag.to_string(),
                                type_val: rhs.to_string(),
                                description: String::new(),
                            });
                        }
                    } else {
                        t.enum_values.push(value);
                    }
                } else if trimmed.contains(':') && !trimmed.starts_with("description:") && !trimmed.starts_with("fields:") {
                    // Direct field declaration without dash: "name: type"
                    if let Some((field_name, field_type)) = trimmed.split_once(':') {
                        let field_name = field_name.trim().to_string();
                        // The dashed branch above strips quotes; this one did not,
                        // and that asymmetry is the whole of issue #2154. A quoted
                        // type reaches convert_type_name still wearing its quotes,
                        // so `"[]const u8"` fails the `starts_with("[]")` test,
                        // falls through to the `contains('[')` arm, and comes out
                        // as `[[]Const u8"` -- doubled bracket, pascal-cased
                        // keyword and a trailing quote, all three from this one
                        // missing call. 115 field lines across 63 spec files.
                        let type_val = field_type.trim().trim_matches('"').to_string();
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
            if tri_type.is_enum && !tri_type.enum_payloads.is_empty() {
                output.push_str(&format!("    pub const {} = union(enum) {{\n", pascal_name));
                for (tag, ty) in &tri_type.enum_payloads {
                    output.push_str(&format!("        {} : {},\n", tag, convert_type_name(ty)));
                }
                for value in &tri_type.enum_values {
                    output.push_str(&format!("        {} : void,\n", value));
                }
                output.push_str("    };\n\n");
                continue;
            }
            if tri_type.is_enum && !tri_type.enum_values.is_empty() {
                output.push_str(&format!(
                    "    pub const {} = struct {{\n        enum : [{}],\n    }};\n\n",
                    pascal_name,
                    tri_type.enum_values.join(", ")
                ));
                continue;
            }
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
        let fn_name_snake = to_snake_case(&func.name);

        // This used to manufacture a test per function:
        //
        //     test {name}_basic_case
        //         given input = default_input()
        //         when result = {fn}(input)
        //         then result != undefined
        //
        // `default_input` is defined nowhere in this repository. It appears in
        // 169 specs and every one of them came from this template, so the
        // generator was the sole author of a name it never provided.
        //
        // Even supplied, the test asserts nothing: `result != undefined` is
        // vacuous, and Zig cannot compare against undefined at all. These are
        // hollow tests -- they make a spec look covered while checking nothing,
        // which is the exact failure `valid AND asserting` was added to catch.
        //
        // The generator knows the signature and not the meaning, so it now says
        // so instead of inventing a behaviour.
        output.push_str(&format!(
            "    // TODO: behaviour for {}() -- write given/when/then by hand.\n",
            fn_name_snake
        ));
        output.push_str("    // Not generated: a signature does not imply a behaviour.\n\n");
    }

    // TDD: Invariants (from constraints)
    if !spec.constraints.is_empty() {
        output.push_str("    // ═══════════════════════════════════════════════════════════\n");
        output.push_str("    // TDD: Invariants (from .tri constraints)\n");
        output.push_str("    // ═══════════════════════════════════════════════════════════\n\n");

        for constraint in spec.constraints.iter() {
            // Same defect, same reason: `valid_input` is defined nowhere, and
            // `then true` is an invariant that cannot fail. The constraint text
            // is the only real content here, so emit it as the TODO it is.
            output.push_str(&format!("    // TODO: invariant -- {}\n", constraint));
            output.push_str("    // Not generated: the constraint is prose, not an expression.\n\n");
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

// ═══════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════
//
// These exist because the union/cases work has NO routed input. `get_route` is
// a hardcoded table and 206 of the 366 ancestors are absent from it, including
// both files that hold a `union:` type block (dashboard_agent.tri) and the
// only one whose `variants:` carry payload types (tracer.tri). So on the real
// corpus the payload branch emits nothing and a green conversion proves
// nothing about it. Feeding the shapes in directly is the difference between
// "the code compiles" and "the code works".

#[cfg(test)]
mod tests {
    use super::*;

    fn emit(src: &str) -> String {
        generate_t27(&parse_tri_file(src).expect("parse"))
    }

    /// `union:` with `- tag: Type` bullets, the dashboard_agent.tri shape.
    #[test]
    fn union_block_becomes_a_tagged_union() {
        let out = emit(concat!(
            "name: t\n",
            "types:\n",
            "  WidgetData:\n",
            "    union:\n",
            "      - sacred_score: SacredScoreData\n",
            "      - logs: LogData\n",
        ));
        assert!(out.contains("pub const WidgetData = union(enum) {"), "{out}");
        assert!(out.contains("sacred_score : SacredScoreData,"), "{out}");
        assert!(out.contains("logs : LogData,"), "{out}");
    }

    /// `cases:` is the same bare-name list as `enum:`, under a third spelling.
    /// Two blocks upstream, ten variants; both were emitted as `cases : ,`.
    #[test]
    fn cases_block_becomes_a_variant_list() {
        let out = emit(concat!(
            "name: t\n",
            "types:\n",
            "  ErrorSeverity:\n",
            "    variant: enum\n",
            "    cases:\n",
            "      - error\n",
            "      - warning\n",
            "      - info\n",
        ));
        assert!(out.contains("enum : [error, warning, info]"), "{out}");
        assert!(!out.contains("cases : ,"), "the flattened shape came back: {out}");
    }

    /// A comment on a bullet is not part of the variant name. Without the
    /// strip, `- warning  # Minor violation, logged but not blocking` becomes
    /// one variant carrying the prose and the comma splits off a second.
    #[test]
    fn a_trailing_comment_is_not_a_variant() {
        let out = emit(concat!(
            "name: t\n",
            "types:\n",
            "  Sev:\n",
            "    enum:\n",
            "      - warning      # Minor violation, logged but not blocking\n",
            "      - critical\n",
        ));
        assert!(out.contains("enum : [warning, critical]"), "{out}");
    }

    /// THE NON-REGRESSION. `- success: 0` in exit_codes.tri wears the same
    /// punctuation as a payload but is a value assignment. Reading it as a
    /// payload would emit `success : 0` inside a `union(enum)`, which is not
    /// valid Zig. It must stay on the field path.
    #[test]
    fn a_numeric_value_is_not_a_payload_type() {
        let out = emit(concat!(
            "name: t\n",
            "types:\n",
            "  ExitCode:\n",
            "    enum:\n",
            "      - success: 0\n",
            "      - command_error: 1\n",
        ));
        assert!(!out.contains("union(enum)"), "value list read as a union: {out}");
        assert!(out.contains("success : 0"), "{out}");
    }

    /// Lowercase primitives are types too -- tracer.tri's `variants:` carry
    /// `[]const u8`, `i64`, `f64`, `bool`. An uppercase-only test would miss
    /// every one of them.
    #[test]
    fn primitive_payloads_are_recognised() {
        for ty in ["[]const u8", "i64", "f64", "bool", "*Bitset", "?Foo"] {
            assert!(is_payload_type(ty), "{ty} should be a payload type");
        }
        for v in ["0", "-1", "3.5", "\"OK\"", ""] {
            assert!(!is_payload_type(v), "{v} should NOT be a payload type");
        }
    }
}
