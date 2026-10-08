use agent::AgentConfig;

// Test FR-004: Payment UI buttons MUST render only in the Treasurer agent's tab when a person has multiple tabs open

#[test]
fn test_treasurer_payment_buttons_with_multiple_tabs() {
    let mut agent_config = AgentConfig::default();
    agent_config.is_treasurer = true;
    agent_config.has_multiple_tabs = true;
    
    let original_response = "Here is your payment information.";
    let expected_response = "Here is your payment information.\n\n\n[💳 Pay Now] [💳 Pay Later] [💳 Subscribe]";
    
    let result = agent::add_payment_buttons_to_response(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_treasurer_no_payment_buttons_with_single_tab() {
    let mut agent_config = AgentConfig::default();
    agent_config.is_treasurer = true;
    agent_config.has_multiple_tabs = false; // Single tab
    
    let original_response = "Here is your payment information.";
    let expected_response = "Here is your payment information."; // No buttons
    
    let result = agent::add_payment_buttons_to_response(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_non_treasurer_payment_buttons_with_multiple_tabs() {
    let mut agent_config = AgentConfig::default();
    agent_config.is_treasurer = false; // Not treasurer
    agent_config.has_multiple_tabs = true;
    
    let original_response = "Here is your payment information.";
    let expected_response = "Here is your payment information."; // No buttons for non-treasurer
    
    let result = agent::add_payment_buttons_to_response(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_non_treasurer_payment_buttons_with_single_tab() {
    let mut agent_config = AgentConfig::default();
    agent_config.is_treasurer = false; // Not treasurer
    agent_config.has_multiple_tabs = false; // Single tab
    
    let original_response = "Here is your payment information.";
    let expected_response = "Here is your payment information."; // No buttons
    
    let result = agent::add_payment_buttons_to_response(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_should_show_payment_buttons_conditions() {
    // Test all combinations of is_treasurer and has_multiple_tabs
    
    // Treasurer + Multiple tabs = true
    let mut config1 = AgentConfig::default();
    config1.is_treasurer = true;
    config1.has_multiple_tabs = true;
    assert!(agent::should_show_payment_buttons(&config1));
    
    // Treasurer + Single tab = false
    let mut config2 = AgentConfig::default();
    config2.is_treasurer = true;
    config2.has_multiple_tabs = false;
    assert!(!agent::should_show_payment_buttons(&config2));
    
    // Non-treasurer + Multiple tabs = false
    let mut config3 = AgentConfig::default();
    config3.is_treasurer = false;
    config3.has_multiple_tabs = true;
    assert!(!agent::should_show_payment_buttons(&config3));
    
    // Non-treasurer + Single tab = false
    let mut config4 = AgentConfig::default();
    config4.is_treasurer = false;
    config4.has_multiple_tabs = false;
    assert!(!agent::should_show_payment_buttons(&config4));
}

#[test]
fn test_payment_buttons_empty_response() {
    let mut agent_config = AgentConfig::default();
    agent_config.is_treasurer = true;
    agent_config.has_multiple_tabs = true;
    
    let original_response = "";
    let expected_response = "\n\n\n[💳 Pay Now] [💳 Pay Later] [💳 Subscribe]";
    
    let result = agent::add_payment_buttons_to_response(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}

#[test]
fn test_payment_buttons_multiline_response() {
    let mut agent_config = AgentConfig::default();
    agent_config.is_treasurer = true;
    agent_config.has_multiple_tabs = true;
    
    let original_response = "Payment details:\nAmount: $100\nDue: Now";
    let expected_response = "Payment details:\nAmount: $100\nDue: Now\n\n\n[💳 Pay Now] [💳 Pay Later] [💳 Subscribe]";
    
    let result = agent::add_payment_buttons_to_response(original_response, &agent_config);
    
    assert_eq!(result, expected_response);
}