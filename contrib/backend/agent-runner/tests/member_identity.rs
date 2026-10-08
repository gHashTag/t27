use agent::AgentConfig;

// Test FR-003: Every member response MUST prefix the agent's name and trade before the answer content

#[test]
fn test_member_identity_with_name_and_trade() {
    let mut agent_config = AgentConfig::default();
    agent_config.agent_name = "Alice".to_string();
    agent_config.agent_trade = "Blacksmith".to_string();
    
    let original_response = "I can help you forge that sword.";
    let expected_response = "Alice the Blacksmith:\n\nI can help you forge that sword.";
    
    let result = agent::add_member_identity_prefix(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_member_identity_with_unknown_name_and_trade() {
    let agent_config = AgentConfig::default();
    // agent_name and agent_trade are "Unknown" by default
    
    let original_response = "I can help you with that task.";
    let expected_response = "I can help you with that task."; // No prefix for unknown
    
    let result = agent::add_member_identity_prefix(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_member_identity_with_partial_info() {
    let mut agent_config = AgentConfig::default();
    agent_config.agent_name = "Bob".to_string();
    // agent_trade remains "Unknown"
    
    let original_response = "I can help you with that task.";
    let expected_response = "I can help you with that task."; // No prefix if trade is unknown
    
    let result = agent::add_member_identity_prefix(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_member_identity_empty_response() {
    let mut agent_config = AgentConfig::default();
    agent_config.agent_name = "Charlie".to_string();
    agent_config.agent_trade = "Carpenter".to_string();
    
    let original_response = "";
    let expected_response = "Charlie the Carpenter:\n\n";
    
    let result = agent::add_member_identity_prefix(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_member_identity_multiline_response() {
    let mut agent_config = AgentConfig::default();
    agent_config.agent_name = "Diana".to_string();
    agent_config.agent_trade = "Doctor".to_string();
    
    let original_response = "I can help you diagnose the issue.\nThe treatment should be straightforward.";
    let expected_response = "Diana the Doctor:\n\nI can help you diagnose the issue.\nThe treatment should be straightforward.";
    
    let result = agent::add_member_identity_prefix(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_member_identity_special_characters() {
    let mut agent_config = AgentConfig::default();
    agent_config.agent_name = "Dr. Émile".to_string();
    agent_config.agent_trade = "Alchemist".to_string();
    
    let original_response = "I can transform lead into gold.";
    let expected_response = "Dr. Émile the Alchemist:\n\nI can transform lead into gold.";
    
    let result = agent::add_member_identity_prefix(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}