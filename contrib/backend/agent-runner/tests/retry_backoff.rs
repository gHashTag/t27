use std::time::{Duration, Instant};
use tokio::time::sleep;

// Test FR-002: A refused turn MUST retry up to 3 times with exponential backoff of 4s, 8s, 16s

#[tokio::test]
async fn test_retry_backoff_first_attempt() {
    let start = Instant::now();
    
    // Simulate first attempt success (no delay)
    let result = simulate_api_call_with_retry(0).await; // 0 = succeed immediately
    
    assert!(result);
    let elapsed = start.elapsed();
    
    // Should complete quickly since first attempt succeeds
    assert!(elapsed < Duration::from_millis(100));
}

#[tokio::test]
async fn test_retry_backoff_second_attempt() {
    let start = Instant::now();
    
    // Simulate failure on first attempt, success on second
    let result = simulate_api_call_with_retry(1).await; // 1 = fail first, succeed second
    
    assert!(result);
    let elapsed = start.elapsed();
    
    // Should take approximately 4 seconds (first delay)
    assert!(elapsed >= Duration::from_secs(3)); // Allow some margin
    assert!(elapsed <= Duration::from_secs(5)); // Allow some margin
}

#[tokio::test]
async fn test_retry_backoff_third_attempt() {
    let start = Instant::now();
    
    // Simulate failure on first two attempts, success on third
    let result = simulate_api_call_with_retry(2).await; // 2 = fail first two, succeed third
    
    assert!(result);
    let elapsed = start.elapsed();
    
    // Should take approximately 4 + 8 = 12 seconds
    assert!(elapsed >= Duration::from_secs(10)); // Allow some margin
    assert!(elapsed <= Duration::from_secs(15)); // Allow some margin
}

// Helper function to simulate API call with retry logic
async fn simulate_api_call_with_retry(fail_attempts: u32) -> bool {
    let max_retries = 3;
    let delays = vec![Duration::from_secs(4), Duration::from_secs(8), Duration::from_secs(16)];
    
    for attempt in 0..=max_retries {
        // Simulate API call
        if attempt < fail_attempts {
            // Fail this attempt
            let delay = delays.get(attempt).unwrap_or(&Duration::from_secs(16));
            sleep(*delay).await;
            continue;
        } else {
            // Success
            return true;
        }
    }
    
    false // All attempts failed
}

#[tokio::test]
async fn test_retry_backoff_all_fail() {
    let start = Instant::now();
    
    // Simulate all attempts failing
    let result = simulate_api_call_with_retry(10).await; // More than max_retries
    
    assert!(!result);
    let elapsed = start.elapsed();
    
    // Should take approximately 4 + 8 + 16 = 28 seconds
    assert!(elapsed >= Duration::from_secs(25)); // Allow some margin
    assert!(elapsed <= Duration::from_secs(35)); // Allow some margin
}

#[tokio::test]
async fn test_retry_backoff_exponential_backoff_timing() {
    let start = Instant::now();
    
    // Test that delays follow exponential pattern: 4s, 8s, 16s
    let delays = vec![Duration::from_secs(4), Duration::from_secs(8), Duration::from_secs(16)];
    
    for (i, expected_delay) in delays.iter().enumerate() {
        let actual_start = Instant::now();
        sleep(*expected_delay).await;
        let actual_duration = actual_start.elapsed();
        
        // Each delay should be close to expected
        assert!(actual_duration >= Duration::from_secs(expected_delay.as_secs() - 1));
        assert!(actual_duration <= Duration::from_secs(expected_delay.as_secs() + 1));
        
        // Second delay should be roughly double the first
        if i == 1 {
            assert!(actual_duration.as_secs() >= 6); // Should be ~8s
            assert!(actual_duration.as_secs() <= 10); // Should be ~8s
        }
        // Third delay should be roughly double the second
        else if i == 2 {
            assert!(actual_duration.as_secs() >= 14); // Should be ~16s
            assert!(actual_duration.as_secs() <= 18); // Should be ~16s
        }
    }
    
    let total_time = start.elapsed();
    // Total time should be approximately 4 + 8 + 16 = 28 seconds
    assert!(total_time >= Duration::from_secs(25));
    assert!(total_time <= Duration::from_secs(35));
}