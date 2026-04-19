//! Integration tests for trios-kg
//!
//! Tests internal module interactions and type consistency.

use trios_kg::{Edge, Entity, KgClient, KgError, QueryParams, SearchResult};

#[test]
fn kg_client_config() {
    let client = KgClient::new("http://localhost:8080");
    assert_eq!(client.base_url, "http://localhost:8080");
}

#[test]
fn entity_new() {
    let entity = Entity {
        id: "test-id".into(),
        label: "Test Concept".into(),
        properties: vec![],
    };
    assert_eq!(entity.id, "test-id");
    assert_eq!(entity.label, "Test Concept");
    assert!(entity.properties.is_empty());
}

#[test]
fn edge_new() {
    let edge = Edge {
        id: "test-edge".into(),
        subject: "test-id".into(),
        predicate: "has-property".into(),
        object: "test-value".into(),
    };
    assert_eq!(edge.id, "test-edge");
    assert_eq!(edge.subject, "test-id");
    assert_eq!(edge.predicate, "has-property");
    assert_eq!(edge.object, "test-value");
}
