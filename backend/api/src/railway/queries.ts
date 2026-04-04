// ─────────────────────────────────────────────────────────────
// Service queries
// ─────────────────────────────────────────────────────────────

/**
 * Fetch a single service by its ID.
 * Usage: railwayRequest<{ service: RailwayService }>(getServiceQuery, { id })
 */
export const getServiceQuery = `
  query getService($id: String!) {
    service(id: $id) {
      id
      name
      createdAt
      updatedAt
    }
  }
`;

/**
 * List all services in a project.
 * Usage: railwayRequest<{ project: { services: { edges: { node: RailwayService }[] } } }>(listServicesQuery, { projectId })
 */
export const listServicesQuery = `
  query listServices($projectId: String!) {
    project(id: $projectId) {
      services {
        edges {
          node {
            id
            name
            createdAt
            updatedAt
          }
        }
      }
    }
  }
`;

// ─────────────────────────────────────────────────────────────
// Deployment queries
// ─────────────────────────────────────────────────────────────

/**
 * Get the latest deployment status for a service in a given environment.
 * Usage: railwayRequest<DeploymentStatusResponse>(getDeploymentStatusQuery, { serviceId, environmentId })
 */
export const getDeploymentStatusQuery = `
  query getDeploymentStatus($serviceId: String!, $environmentId: String!) {
    deployments(
      first: 1
      input: {
        serviceId: $serviceId
        environmentId: $environmentId
      }
    ) {
      edges {
        node {
          id
          status
          createdAt
          updatedAt
          url
          meta
        }
      }
    }
  }
`;
