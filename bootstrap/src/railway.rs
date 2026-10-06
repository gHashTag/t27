// bootstrap/src/railway.rs
// Railway GraphQL client for creating sandbox services

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const RAILWAY_GRAPHQL: &str = "https://backboard.railway.com/graphql/v2";

#[derive(Serialize)]
struct ServiceCreateInput {
    name: String,
    service_id: String,
}

#[derive(Serialize)]
struct VariableCollectionUpsertInput {
    id: String,
    variables: Vec<VariableInput>,
}

#[derive(Serialize)]
struct VariableInput {
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
struct ServiceCreateData {
    service_create: ServiceCreateResult,
}

#[derive(Deserialize)]
struct ServiceCreateResult {
    id: String,
}

/// Create a new Railway service by cloning a base service
///
/// # Arguments
/// * `service_name` - Display name for the new service
/// * `session_id` - Session identifier (used for service naming)
/// * `railway_token` - Railway API token for authentication
/// * `base_service_id` - ID of the base service to clone (from env var RAILWAY_SERVICE_ID)
///
/// # Returns
/// The ID of the newly created Railway service
pub async fn create_railway_service(
    service_name: &str,
    _session_id: &str,
    railway_token: &str,
    base_service_id: &str,
) -> Result<String> {
    let client = Client::new();

    // GraphQL mutation for creating service from a template
    // Note: This is a simplified version. Actual Railway API may require
    // project ID, environment ID, and more complex mutations.
    let mutation = format!(
        r#"mutation {{
            serviceCreate(input: {{
                name: "{}",
                serviceId: "{}"
            }}) {{
                id
            }}
        }}"#,
        service_name, base_service_id
    );

    let resp = client.post(RAILWAY_GRAPHQL)
        .header("Authorization", format!("Bearer {}", railway_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({"query": mutation}))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|_| "Unable to read body".to_string());
        return Err(anyhow::anyhow!(
            "Failed to create Railway service: {} - {}",
            status,
            body
        ));
    }

    let json: GraphQLResponse<ServiceCreateData> = resp.json().await?;

    if let Some(errors) = json.errors {
        return Err(anyhow::anyhow!(
            "GraphQL errors: {}",
            errors.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join("; ")
        ));
    }

    match json.data {
        Some(data) => Ok(data.service_create.id),
        None => Err(anyhow::anyhow!("No data returned from Railway API")),
    }
}

/// Set environment variables for a Railway service
///
/// # Arguments
/// * `service_id` - ID of the Railway service
/// * `variables` - Key-value pairs of environment variables to set
/// * `railway_token` - Railway API token for authentication
pub async fn set_service_variables(
    service_id: &str,
    variables: &[(String, String)],
    railway_token: &str,
) -> Result<()> {
    let client = Client::new();

    let variable_inputs: Vec<VariableInput> = variables
        .iter()
        .map(|(k, v)| VariableInput {
            key: k.clone(),
            value: v.clone(),
        })
        .collect();

    let mutation = format!(
        r#"mutation {{
            variableCollectionUpsert(input: {{
                id: "{}",
                variables: {}
            }}) {{
                id
            }}
        }}"#,
        service_id,
        serde_json::to_string(&variable_inputs)?
    );

    let resp = client.post(RAILWAY_GRAPHQL)
        .header("Authorization", format!("Bearer {}", railway_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({"query": mutation}))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|_| "Unable to read body".to_string());
        return Err(anyhow::anyhow!(
            "Failed to set Railway service variables: {} - {}",
            status,
            body
        ));
    }

    Ok(())
}

/// Delete a Railway service
///
/// # Arguments
/// * `service_id` - ID of the Railway service to delete
/// * `railway_token` - Railway API token for authentication
pub async fn delete_railway_service(
    service_id: &str,
    railway_token: &str,
) -> Result<()> {
    let client = Client::new();

    let mutation = format!(
        r#"mutation {{
            serviceDelete(input: {{ id: "{}" }}) {{
                id
            }}
        }}"#,
        service_id
    );

    let resp = client.post(RAILWAY_GRAPHQL)
        .header("Authorization", format!("Bearer {}", railway_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({"query": mutation}))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|_| "Unable to read body".to_string());
        return Err(anyhow::anyhow!(
            "Failed to delete Railway service: {} - {}",
            status,
            body
        ));
    }

    Ok(())
}

/// Get the health status of a Railway service
///
/// # Arguments
/// * `service_id` - ID of the Railway service
/// * `railway_token` - Railway API token for authentication
///
/// # Returns
/// True if the service is healthy and ready, false otherwise
pub async fn check_service_health(
    service_id: &str,
    railway_token: &str,
) -> Result<bool> {
    let client = Client::new();

    // Query service status
    let query = format!(
        r#"{{
            service(id: "{}") {{
                status
                deployments(last: 1) {{
                    id
                    status
                }}
            }}
        }}"#,
        service_id
    );

    let resp = client.post(RAILWAY_GRAPHQL)
        .header("Authorization", format!("Bearer {}", railway_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({"query": query}))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(false);
    }

    let json: serde_json::Value = resp.json().await?;

    // Check if service is active and deployment is successful
    if let Some(service) = json["data"]["service"].as_object() {
        let status = service["status"].as_str().unwrap_or("");
        if status != "active" {
            return Ok(false);
        }

        if let Some(deployments) = service["deployments"].as_array() {
            if let Some(first_deployment) = deployments.first() {
                let deploy_status = first_deployment["status"].as_str().unwrap_or("");
                return Ok(deploy_status == "success");
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::RAILWAY_GRAPHQL;

    /// FR-002: guard the Railway GraphQL endpoint's host and path, so the
    /// retired host cannot come back unnoticed. The previous host stopped
    /// answering entirely (HTTP 000 within 15 s, measured 2026-10-04, see
    /// docs/now/2026-10-04-railway-graphql-host.md); Railway's public API
    /// lives at backboard.railway.com/graphql/v2.
    #[test]
    fn railway_graphql_targets_public_api() {
        assert_eq!(
            RAILWAY_GRAPHQL,
            "https://backboard.railway.com/graphql/v2"
        );

        let rest = RAILWAY_GRAPHQL
            .strip_prefix("https://")
            .expect("RAILWAY_GRAPHQL must use https");
        let (host, path) = rest
            .split_once('/')
            .expect("RAILWAY_GRAPHQL must include a path");
        assert_eq!(host, "backboard.railway.com");
        assert_eq!(path, "graphql/v2");
    }
}
