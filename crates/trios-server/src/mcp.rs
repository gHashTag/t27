use rust_mcp_schema::{
    CallToolRequestParams, CallToolResult, ContentBlock, ListToolsResult, TextContent,
    Tool, ToolInputSchema,
};
use serde_json::{json, Value};
use std::sync::LazyLock;
use tracing::info;

use crate::tools;

static TOOL_DEFINITIONS: LazyLock<Vec<Tool>> = LazyLock::new(build_tool_definitions);

fn prop_type(type_str: &str) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    m.insert("type".into(), Value::String(type_str.into()));
    m
}

fn prop_array_items() -> serde_json::Map<String, Value> {
    json!({"type": "array", "items": {"type": "string"}})
        .as_object()
        .unwrap()
        .clone()
}

fn build_tool_definitions() -> Vec<Tool> {
    fn make_tool(
        name: &str,
        description: &str,
        required: Vec<&str>,
        properties: Vec<(&str, serde_json::Map<String, Value>)>,
    ) -> Tool {
        Tool {
            name: name.into(),
            description: Some(description.into()),
            input_schema: ToolInputSchema::new(
                required.into_iter().map(String::from).collect(),
                Some(
                    properties
                        .into_iter()
                        .map(|(k, v)| (k.into(), v))
                        .collect(),
                ),
                None,
            ),
            annotations: None,
            meta: None,
            icons: vec![],
            execution: None,
            output_schema: None,
            title: None,
        }
    }

    vec![
        // === Git Basic (5) ===
        make_tool(
            "git_status",
            "List all changed files in a repository",
            vec!["repo_path"],
            vec![("repo_path", prop_type("string"))],
        ),
        make_tool(
            "git_stage_files",
            "Stage specific files for commit",
            vec!["repo_path", "paths"],
            vec![
                ("repo_path", prop_type("string")),
                ("paths", prop_array_items()),
            ],
        ),
        make_tool(
            "git_unstage_files",
            "Unstage specific files",
            vec!["repo_path", "paths"],
            vec![
                ("repo_path", prop_type("string")),
                ("paths", prop_array_items()),
            ],
        ),
        make_tool(
            "git_commit",
            "Commit all staged files",
            vec!["repo_path", "message"],
            vec![
                ("repo_path", prop_type("string")),
                ("message", prop_type("string")),
            ],
        ),
        make_tool(
            "git_create_branch",
            "Create a new branch from HEAD",
            vec!["repo_path", "name"],
            vec![
                ("repo_path", prop_type("string")),
                ("name", prop_type("string")),
            ],
        ),
        // === Git Extended (4) ===
        make_tool(
            "git_log",
            "Get commit history",
            vec!["repo_path"],
            vec![
                ("repo_path", prop_type("string")),
                ("limit", json!({"type": "integer", "default": 10}).as_object().unwrap().clone()),
            ],
        ),
        make_tool(
            "git_diff",
            "Show unstaged changes (optionally filtered by file)",
            vec!["repo_path"],
            vec![
                ("repo_path", prop_type("string")),
                ("file", prop_type("string")),
            ],
        ),
        make_tool(
            "git_stash",
            "Stash current working directory changes",
            vec!["repo_path"],
            vec![("repo_path", prop_type("string"))],
        ),
        make_tool(
            "git_checkout",
            "Switch to an existing branch",
            vec!["repo_path", "branch"],
            vec![
                ("repo_path", prop_type("string")),
                ("branch", prop_type("string")),
            ],
        ),
        // === GitButler (3) ===
        make_tool(
            "gb_list_branches",
            "List GitButler virtual branches",
            vec!["repo_path"],
            vec![("repo_path", prop_type("string"))],
        ),
        make_tool(
            "gb_push_stack",
            "Push a GitButler stack/branch",
            vec!["repo_path", "branch_name"],
            vec![
                ("repo_path", prop_type("string")),
                ("branch_name", prop_type("string")),
            ],
        ),
        make_tool(
            "gb_workspace_status",
            "Get full workspace status (git changes + GB branches)",
            vec!["repo_path"],
            vec![("repo_path", prop_type("string"))],
        ),
        // === Filesystem (3) ===
        make_tool(
            "fs_read_file",
            "Read file contents (must be within allowed roots)",
            vec!["path"],
            vec![("path", prop_type("string"))],
        ),
        make_tool(
            "fs_write_file",
            "Write content to a file (must be within allowed roots)",
            vec!["path", "content"],
            vec![
                ("path", prop_type("string")),
                ("content", prop_type("string")),
            ],
        ),
        make_tool(
            "fs_list_dir",
            "List directory contents (must be within allowed roots)",
            vec!["path"],
            vec![("path", prop_type("string"))],
        ),
    ]
}

#[derive(Clone)]
pub struct McpService;

impl McpService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_tools(&self) -> ListToolsResult {
        ListToolsResult {
            tools: TOOL_DEFINITIONS.clone(),
            meta: None,
            next_cursor: None,
        }
    }

    pub async fn call_tool(&self, params: CallToolRequestParams) -> CallToolResult {
        info!("Calling tool: {}", params.name);

        let arguments_value: Value = params
            .arguments
            .map(|map| Value::Object(map.into_iter().collect()))
            .unwrap_or(Value::Null);

        match tools::dispatch(&params.name, arguments_value).await {
            Ok(value) => {
                let text: String = if value.is_object() || value.is_array() {
                    serde_json::to_string_pretty(&value).unwrap_or_default()
                } else if value.is_string() {
                    value.as_str().unwrap().to_string()
                } else {
                    serde_json::to_string(&value).unwrap_or_default()
                };

                CallToolResult {
                    content: vec![ContentBlock::TextContent(TextContent::new(
                        text,
                        None,
                        None,
                    ))],
                    is_error: Some(false),
                    meta: None,
                    structured_content: None,
                }
            }
            Err(e) => CallToolResult {
                content: vec![ContentBlock::TextContent(TextContent::new(
                    format!("Error: {}", e),
                    None,
                    None,
                ))],
                is_error: Some(true),
                meta: None,
                structured_content: None,
            },
        }
    }
}
