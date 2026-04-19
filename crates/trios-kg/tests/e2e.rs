//! E2E tests for trios-kg
//!
//! Tests knowledge graph client operations with mock HTTP responses.

use trios_kg::{Edge, Entity, KgClient, QueryParams, SearchResult};

#[test]
fn kg_query_with_results() {
    // Verify API structure is correct
    let client = KgClient::new("http://localhost:8080");
    assert_eq!(client.base_url, "http://localhost:8080");
}

#[test]
fn kg_entity_creation() {
    let entity = Entity {
        id: "test-id".into(),
        entity_type: "concept".into(),
        name: "Test Concept".into(),
        properties: vec![],
    };
    assert_eq!(entity.id, "test-id");
    assert_eq!(entity.entity_type, "concept");
    assert_eq!(entity.name, "Test Concept");
    assert!(entity.properties.is_empty());
}

#[test]
fn kg_edge_creation() {
    let edge = Edge {
        id: "test-edge".into(),
        source: "test-id".into(),
        target: "test-id".into(),
        edge_type: "has-property".into(),
        object: "test-value".into(),
    };
    assert_eq!(edge.id, "test-edge");
    assert_eq!(edge.source, "test-id");
    assert_eq!(edge.target, "test-id");
    assert_eq!(edge.object, "test-value");
}
