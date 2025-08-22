use std::time::Duration;
use website_monitor::{Config, WebsiteMonitor};

#[test]
fn test_connection_refused_ureq() {
    let config = Config {
        worker_threads: 1,
        request_timeout: Duration::from_secs(2),
        max_retries: 0,
    };
    
    let monitor = WebsiteMonitor::new(config);
    // Use localhost port that should be closed
    let results = monitor.monitor_websites(vec![
        "http://localhost:9999".to_string()
    ]);
    
    assert_eq!(results.len(), 1);
    assert!(results[0].status.is_err());
    
    if let Err(ref error) = results.status {
        // ureq will give transport errors for connection issues
        assert!(error.contains("Transport error"));
    }
}

#[test]
fn test_dns_resolution_failure_ureq() {
    let config = Config {
        worker_threads: 1,
        request_timeout: Duration::from_secs(5),
        max_retries: 0,
    };
    
    let monitor = WebsiteMonitor::new(config);
    let results = monitor.monitor_websites(vec![
        "https://this-domain-absolutely-does-not-exist-12345.invalid".to_string()
    ]);
    
    assert_eq!(results.len(), 1);
    assert!(results[0].status.is_err());
    
    if let Err(ref error) = results.status {
        assert!(error.contains("Transport error"));
    }
}

#[test]
fn test_malformed_urls_ureq() {
    let config = Config::default();
    let monitor = WebsiteMonitor::new(config);
    
    let invalid_urls = vec![
        "not-a-url".to_string(),
        "ftp://example.com".to_string(), // ureq supports this, but might fail
        "http://".to_string(),
        "://invalid".to_string(),
    ];
    
    let results = monitor.monitor_websites(invalid_urls);
    assert_eq!(results.len(), 4);
    
    // Most should return errors (ureq is more forgiving than our manual implementation)
    let error_count = results.iter().filter(|r| r.status.is_err()).count();
    assert!(error_count >= 2); // At least some should fail
}

#[test]
fn test_mixed_success_and_failure_ureq() {
    let config = Config {
        worker_threads: 3,
        request_timeout: Duration::from_secs(5),
        max_retries: 0,
    };
    let monitor = WebsiteMonitor::new(config);
    
    let urls = vec![
        "https://httpbin.org/status/200".to_string(),  // Should succeed
        "https://nonexistent.invalid".to_string(),     // Should fail (DNS)
        "https://httpbin.org/status/500".to_string(),  // Should succeed with 500
        "not-a-url-at-all".to_string(),               // Should fail (invalid URL)
    ];
    
    let results = monitor.monitor_websites(urls);
    assert_eq!(results.len(), 4);
    
    // Check that we have both successes and failures
    let successes = results.iter().filter(|r| r.status.is_ok()).count();
    let failures = results.iter().filter(|r| r.status.is_err()).count();
    
    assert!(successes >= 1); // At least the httpbin URLs should work
    assert!(failures >= 1);  // At least the invalid ones should fail
    
    // All should have valid response times and URLs
    for result in &results {
        assert!(!result.url.is_empty());
        assert!(result.response_time >= Duration::from_nanos(0));
    }
}

#[test]
fn test_retry_mechanism_ureq() {
    let config = Config {
        worker_threads: 1,
        request_timeout: Duration::from_millis(100), // Very short timeout
        max_retries: 2,
    };
    
    let monitor = WebsiteMonitor::new(config);
    let results = monitor.monitor_websites(vec![
        "https://httpbin.org/delay/5".to_string() // This should timeout
    ]);
    
    assert_eq!(results.len(), 1);
    // Should eventually timeout even with retries
    assert!(results[0].status.is_err());
    
    // Response time should be longer due to retries
    // (though this is hard to test deterministically)
    assert!(results.response_time >= Duration::from_millis(100));
}

#[test]
fn test_concurrent_error_handling_ureq() {
    let config = Config {
        worker_threads: 4,
        request_timeout: Duration::from_secs(2),
        max_retries: 0,
    };
    
    let monitor = WebsiteMonitor::new(config);
    
    // Mix of valid and invalid URLs
    let urls = vec![
        "https://example.com".to_string(),
        "https://invalid1.nonexistent".to_string(),
        "https://google.com".to_string(),
        "https://invalid2.nonexistent".to_string(),
        "not-a-url".to_string(),
        "https://github.com".to_string(),
    ];
    
    let results = monitor.monitor_websites(urls);
    assert_eq!(results.len(), 6);
    
    // Should have a mix of successes and failures
    let successes = results.iter().filter(|r| r.status.is_ok()).count();
    let failures = results.iter().filter(|r| r.status.is_err()).count();
    
    println!("Successes: {}, Failures: {}", successes, failures);
    
    // We expect at least some successes (real domains) and some failures (invalid ones)
    assert!(successes >= 1);
    assert!(failures >= 1);
}
