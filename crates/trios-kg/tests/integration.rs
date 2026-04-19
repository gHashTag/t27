//! Integration tests for trios-kg
//!
//! Tests internal module interactions and type consistency.

use trios_kg::{Edge, Entity, KgClient, KgError, QueryParams, SearchResult};

#[test]
fn kg_client_config() {
    let _client = KgClient::new("http://localhost:8080");
}

#[test]
fn entity_construction() {
    let entity = Entity {
        id: "test-id".into(),
        entity_type: "concept".into(),
        name: "Test Concept".into(),
        properties: serde_json::json!({"key": "value"}),
        created_at: Some("2026-01-01".into()),
    };
    assert_eq!(entity.id, "test-id");
    assert_eq!(entity.name, "Test Concept");
    assert!(entity.properties.is_object());
}

#[test]
fn edge_construction() {
    let edge = Edge {
        id: "test-edge".into(),
        source: "entity-1".into(),
        target: "entity-2".into(),
        edge_type: "related_to".into(),
        weight: None,
        properties: serde_json::Value::Null,
    };
    assert_eq!(edge.id, "test-edge");
    assert_eq!(edge.source, "entity-1");
    assert_eq!(edge.target, "entity-2");
    assert!(edge.weight.is_none());
}
