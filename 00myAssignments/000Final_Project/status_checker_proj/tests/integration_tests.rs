use status_checker_proj;

use mockito::{mock, server_url};
use std::time::Duration;
use website_monitor::{Config, WebsiteMonitor};

#[test]
fn test_successful_http_request_with_ureq() {
    let _m = mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("Hello World")
        .create();

    let config = Config {
        worker_threads: 1,
        request_timeout: Duration::from_secs(5),
        max_retries: 0,
    };
    
    let monitor = WebsiteMonitor::new(config);
    let mock_url = server_url();
    let results = monitor.monitor_websites(vec![mock_url.clone()]);
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url, mock_url);
    
    // With ureq, should get successful response
    match results.status {
        Ok(status) => assert_eq!(status, 200),
        Err(ref error) => panic!("Expected successful request, got error: {}", error),
    }
}

#[test]
fn test_http_404_status_with_ureq() {
    let _m = mock("GET", "/notfound")
        .with_status(404)
        .with_body("Not Found")
        .create();

    let config = Config::default();
    let monitor = WebsiteMonitor::new(config);
    let test_url = format!("{}/notfound", server_url());
    let results = monitor.monitor_websites(vec![test_url]);
    
    assert_eq!(results.len(), 1);
    
    // ureq treats 404 as a successful HTTP response with status code 404
    match results[0].status {
        Ok(status) => assert_eq!(status, 404),
        Err(ref error) => panic!("Expected HTTP 404, got error: {}", error),
    }
}

#[test]
fn test_multiple_concurrent_requests_ureq() {
    let _m1 = mock("GET", "/endpoint1")
        .with_status(200)
        .create();
    
    let _m2 = mock("GET", "/endpoint2")
        .with_status(301)
        .create();
    
    let _m3 = mock("GET", "/endpoint3")
        .with_status(500)
        .create();

    let config = Config {
        worker_threads: 3,
        request_timeout: Duration::from_secs(5),
        max_retries: 0,
    };
    
    let monitor = WebsiteMonitor::new(config);
    let base_url = server_url();
    let urls = vec![
        format!("{}/endpoint1", base_url),
        format!("{}/endpoint2", base_url),
        format!("{}/endpoint3", base_url),
    ];
    
    let results = monitor.monitor_websites(urls);
    assert_eq!(results.len(), 3);
    
    // All should complete successfully (HTTP status codes are not errors)
    let mut status_codes: Vec<u16> = Vec::new();
    for result in &results {
        match result.status {
            Ok(status) => status_codes.push(status),
            Err(ref error) => panic!("Unexpected error: {}", error),
        }
    }
    
    status_codes.sort();
    assert_eq!(status_codes, vec![200, 301, 500]);
}

#[test]
fn test_connection_timeout_with_ureq() {
    // Test with a URL that will definitely fail to connect
    let config = Config {
        worker_threads: 1,
        request_timeout: Duration::from_millis(100),
        max_retries: 0,
    };
    
    let monitor = WebsiteMonitor::new(config);
    // Use a non-routable IP address to guarantee timeout
    let results = monitor.monitor_websites(vec!["http://10.255.255.1:80".to_string()]);
    
    assert_eq!(results.len(), 1);
    // Should timeout and return an error
    assert!(results[0].status.is_err());
    
    if let Err(ref error) = results.status {
        assert!(error.contains("Transport error") || error.contains("timeout"));
    }
}

#[test]
fn test_invalid_url_with_ureq() {
    let config = Config::default();
    let monitor = WebsiteMonitor::new(config);
    
    let results = monitor.monitor_websites(vec![
        "not-a-valid-url".to_string(),
        "http://".to_string(),
    ]);
    
    assert_eq!(results.len(), 2);
    
    // Both should fail with transport errors
    for result in &results {
        assert!(result.status.is_err());
    }
}
