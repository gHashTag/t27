//! MCP tools for trios-kg crate (R0 stub)
//!
//! Exposes Knowledge Graph operations through Model Context Protocol.

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};

/// Dispatch knowledge graph tools.
pub async fn dispatch(name: &str, input: &Value) -> Option<Result<Value>> {
    match name {
        "kg_create_entity" => Some(kg_create_entity(input).await),
        "kg_create_edge" => Some(kg_create_edge(input).await),
        "kg_query" => Some(kg_query(input).await),
        "kg_traverse" => Some(kg_traverse(input).await),
        _ => None,
    }
}

/// Create an entity (node) in knowledge graph.
pub async fn kg_create_entity(input: &Value) -> Result<Value> {
    let entity_type = input
        .get("entity_type")
        .and_then(|v| v.as_str())
        .context("entity_type is required")?;

    let name = input
        .get("name")
        .and_then(|v| v.as_str())
        .context("name is required")?;

    let properties = input
        .get("properties")
        .and_then(|v| match v {
            Value::Object(props) => Some(props.clone()),
            _ => None,
        })
        .unwrap_or_else(|| Map::new());

    // R0: Generate mock ID
    let id = format!("entity-{:x}", md5_compute(&name.to_string()));

    Ok(json!({
        "id": id,
        "entity_type": entity_type,
        "name": name,
        "properties": properties,
    }))
}

/// Create a relationship (edge) between two entities.
pub async fn kg_create_edge(input: &Value) -> Result<Value> {
    let source = input
        .get("source")
        .and_then(|v| v.as_str())
        .context("source is required")?;

    let target = input
        .get("target")
        .and_then(|v| v.as_str())
        .context("target is required")?;

    let edge_type = input
        .get("edge_type")
        .and_then(|v| v.as_str())
        .context("edge_type is required")?;

    let weight = input
        .get("weight")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    let id = format!("edge-{:x}", md5_compute(&format!("{}-{}-{}", source, target, edge_type)));

    Ok(json!({
        "id": id,
        "source": source,
        "target": target,
        "edge_type": edge_type,
        "weight": weight,
    }))
}

/// Query knowledge graph with optional filters.
pub async fn kg_query(input: &Value) -> Result<Value> {
    let _query = input
        .get("query")
        .and_then(|v| v.as_str())
        .context("query is required")?;

    let limit = input
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10);

    // R0: Return mock results
    let entities: Vec<Value> = (0..limit.min(3))
        .map(|i| json!({
            "id": format!("entity-{:x}", md5_compute(&format!("query-{}", i))),
            "entity_type": "concept",
            "name": format!("Concept {}", i),
            "properties": {},
        }))
        .collect();

    Ok(json!({
        "entities": entities,
        "edges": [],
        "total": entities.len(),
    }))
}

/// Traverse graph relationships starting from an entity.
pub async fn kg_traverse(input: &Value) -> Result<Value> {
    let source = input
        .get("source")
        .and_then(|v| v.as_str())
        .context("source is required")?;

    let max_depth = input
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    // R0: Return mock traversal
    let paths: Vec<Value> = (0..max_depth.min(3))
        .map(|i| json!({
            "source": source,
            "target": format!("entity-{:x}", md5_compute(&format!("path-{}", i))),
            "edge_type": "related_to",
            "depth": i + 1,
        }))
        .collect();

    Ok(json!({
        "paths": paths,
        "total": paths.len(),
    }))
}

// Simple hash stub for ID generation
fn md5_compute(data: &str) -> u64 {
    // R0: Simple hash (not real MD5)
    let mut hash: u64 = 5381;
    for b in data.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    hash
}
