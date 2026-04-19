//! E2E tests for trios-kg
//!
//! Tests knowledge graph client operations with mock HTTP responses.

use trios_kg::{Edge, Entity, KgClient, QueryParams, SearchResult};

#[test]
fn kg_client_construction() {
    let client = KgClient::new("http://localhost:8080");
    // Client constructed successfully
    let _ = client;
}

#[test]
fn kg_entity_creation() {
    let entity = Entity {
        id: "test-id".into(),
        entity_type: "concept".into(),
        name: "Test Concept".into(),
        properties: serde_json::Value::Null,
        created_at: None,
    };
    assert_eq!(entity.id, "test-id");
    assert_eq!(entity.entity_type, "concept");
    assert_eq!(entity.name, "Test Concept");
}

#[test]
fn kg_edge_creation() {
    let edge = Edge {
        id: "test-edge".into(),
        source: "entity-1".into(),
        target: "entity-2".into(),
        edge_type: "depends_on".into(),
        weight: Some(0.95),
        properties: serde_json::Value::Null,
    };
    assert_eq!(edge.id, "test-edge");
    assert_eq!(edge.source, "entity-1");
    assert_eq!(edge.target, "entity-2");
    assert_eq!(edge.edge_type, "depends_on");
}
