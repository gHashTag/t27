// ─────────────────────────────────────────────────────────────
// Service mutations
// ─────────────────────────────────────────────────────────────

export const serviceCreateMutation = `
  mutation serviceCreate($input: ServiceCreateInput!) {
    serviceCreate(input: $input) {
      id
      name
    }
  }
`;

export const serviceDeleteMutation = `
  mutation serviceDelete($id: String!) {
    serviceDelete(id: $id)
  }
`;

// ─────────────────────────────────────────────────────────────
// Environment variable mutations
// ─────────────────────────────────────────────────────────────

/**
 * Upsert a collection of environment variables on a service.
 * Variables is an object mapping name → value.
 */
export const variableCollectionUpsertMutation = `
  mutation variableCollectionUpsert($input: VariableCollectionUpsertInput!) {
    variableCollectionUpsert(input: $input)
  }
`;

// ─────────────────────────────────────────────────────────────
// Deployment mutations
// ─────────────────────────────────────────────────────────────

export const serviceInstanceRedeployMutation = `
  mutation serviceInstanceRedeploy($environmentId: String!, $serviceId: String!) {
    serviceInstanceRedeploy(environmentId: $environmentId, serviceId: $serviceId)
  }
`;

// ─────────────────────────────────────────────────────────────
// Volume mutations
// ─────────────────────────────────────────────────────────────

export const volumeCreateMutation = `
  mutation volumeCreate($input: VolumeCreateInput!) {
    volumeCreate(input: $input) {
      id
      name
    }
  }
`;

export const volumeDeleteMutation = `
  mutation volumeDelete($id: String!) {
    volumeDelete(id: $id)
  }
`;
