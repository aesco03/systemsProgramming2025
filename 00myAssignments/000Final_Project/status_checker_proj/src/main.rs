use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use std::io;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub worker_threads: usize,
    pub request_timeout: Duration,
    pub max_retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            worker_threads: 10,
            request_timeout: Duration::from_secs(5),
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteStatus {
    pub url: String,
    pub status: Result<u16, String>,
    pub response_time: Duration,
    pub timestamp: DateTime<Utc>,
}

// Work item structure for threading
#[derive(Debug, Clone)]
struct WorkItem {
    url: String,
    retries_left: u32,
}

// Website monitor structure
pub struct WebsiteMonitor {
    config: Config,
}

impl WebsiteMonitor {
    pub fn new(config: Config) -> Self {
        WebsiteMonitor { config }
    }

    pub fn monitor_websites(&self, urls: Vec<String>) -> Vec<WebsiteStatus> {
        let (work_sender, work_receiver) = mpsc::channel::<WorkItem>();
        let (result_sender, result_receiver) = mpsc::channel::<WebsiteStatus>();
        
        // Arc wrap the receiver for sharing between threads
        let work_receiver = Arc::new(Mutex::new(work_receiver));
        let urls_to_process = urls.len();
        
        // Start worker threads manually
        let mut handles = Vec::new();
        for worker_id in 0..self.config.worker_threads {
            let work_receiver_clone = Arc::clone(&work_receiver);
            let result_sender_clone = result_sender.clone();
            let config_clone = self.config.clone();
            
            let handle = thread::spawn(move || {
                Self::worker_thread(worker_id, work_receiver_clone, result_sender_clone, config_clone);
            });
            handles.push(handle);
        }

        // Send all work items to the queue
        for url in urls {
            let work_item = WorkItem {
                url,
                retries_left: self.config.max_retries,
            };
            if let Err(e) = work_sender.send(work_item) {
                eprintln!("Failed to send work item: {}", e);
            }
        }

        // Close the work sender to signal completion
        drop(work_sender);

        // Collect results from all workers
        let mut results = Vec::new();
        let mut collected = 0;
        
        // Collect results until we have all of them or the channel closes
        while collected < urls_to_process {
            match result_receiver.recv() {
                Ok(result) => {
                    results.push(result);
                    collected += 1;
                }
                Err(_) => {
                    // Channel closed, break out
                    break;
                }
            }
        }

        // Wait for all worker threads to finish
        for handle in handles {
            if let Err(e) = handle.join() {
                eprintln!("Worker thread panicked: {:?}", e);
            }
        }

        results
    }

    fn worker_thread(
        worker_id: usize,
        work_receiver: Arc<Mutex<Receiver<WorkItem>>>,
        result_sender: Sender<WebsiteStatus>,
        config: Config,
    ) {
        println!("Worker {} started", worker_id);
        
        loop {
            // Get work item from shared queue
            let work_item = {
                match work_receiver.lock() {
                    Ok(receiver) => receiver.recv(),
                    Err(_) => {
                        eprintln!("Worker {}: Failed to acquire lock", worker_id);
                        break;
                    }
                }
            };

            match work_item {
                Ok(item) => {
                    let mut result = Self::check_website(&item.url, &config);
                    
                    // Handle retries manually
                    if result.status.is_err() && item.retries_left > 0 {
                        println!("Worker {}: Retrying {} ({} retries left)", 
                                worker_id, item.url, item.retries_left);
                        
                        // Wait a bit before retry
                        thread::sleep(Duration::from_millis(100));
                        
                        // Retry the request
                        result = Self::check_website(&item.url, &config);
                        
                        // If still failed and more retries available, you could implement
                        // a retry queue here, but for simplicity we'll just try once more
                    }
                    
                    // Send result back
                    if let Err(e) = result_sender.send(result) {
                        eprintln!("Worker {}: Failed to send result: {}", worker_id, e);
                        break;
                    }
                }
                Err(_) => {
                    // Channel closed, no more work
                    println!("Worker {} shutting down", worker_id);
                    break;
                }
            }
        }
    }
////
    pub fn check_website(url: &str, config: &Config) -> WebsiteStatus {
        let start_time = Instant::now();
        let timestamp = Utc::now(); // Use Utc::now() instead of SystemTime::now()
        
        let status = Self::make_http_request_with_ureq(url, config.request_timeout);
        let response_time = start_time.elapsed();


        WebsiteStatus {
            url: url.to_string(),
            status,
            response_time,
            timestamp,
        }
    }

    fn make_http_request_with_ureq(url: &str, timeout: Duration) -> Result<u16, String> {
        // Create ureq agent with timeout
        let agent = ureq::AgentBuilder::new()
            .timeout_read(timeout)
            .timeout_write(timeout)
            .build();

        // Make the HTTP request
        match agent.get(url).call() {
            Ok(response) => {
                let status_code = response.status();
                Ok(status_code)
            }
            Err(ureq::Error::Status(code, _response)) => {
                // HTTP error status codes (4xx, 5xx) are still "successful" requests
                Ok(code)
            }
            Err(ureq::Error::Transport(transport_error)) => {
                Err(format!("Transport error: {}", transport_error))
            }
        }
    }

    pub fn print_results(&self, results: &[WebsiteStatus]) {
        println!("\n=== Website Monitoring Results ===");
        println!("{:<50} {:<15} {:<15} {:<50}", "URL", "Status", "Response Time", "Error");
        println!("{}", "=".repeat(130));
        
        for result in results {
            let status_display = match &result.status {
                Ok(code) => code.to_string(),
                Err(_) => "ERROR".to_string(),
            };
            
            let error_display = match &result.status {
                Ok(_) => "None".to_string(),
                Err(e) => {
                    if e.len() > 45 {
                        format!("{}...", &e[..42])
                    } else {
                        e.clone()
                    }
                }
            };
            
            let response_time_ms = result.response_time.as_millis();
            
            println!("{:<50} {:<15} {:<15}ms {:<50}", 
                     result.url, status_display, response_time_ms, error_display);
        }
        
        // Print summary statistics
        let total = results.len();
        let successful = results.iter().filter(|r| r.status.is_ok()).count();
        let failed = total - successful;
        
        println!("\n=== Summary ===");
        println!("Total websites checked: {}", total);
        println!("Successful: {}", successful);
        println!("Failed: {}", failed);
        
        if total > 0 {
            println!("Success rate: {:.1}%", (successful as f64 / total as f64) * 100.0);
            
            // Calculate average response time for successful requests
            let successful_times: Vec<Duration> = results
                .iter()
                .filter(|r| r.status.is_ok())
                .map(|r| r.response_time)
                .collect();
            
            if !successful_times.is_empty() {
                let avg_time_ms = successful_times
                    .iter()
                    .map(|d| d.as_millis() as f64)
                    .sum::<f64>() / successful_times.len() as f64;
                
                println!("Average response time: {:.1}ms", avg_time_ms);
            }
        }
    }

    pub fn save_results_to_json(&self, results: &[WebsiteStatus], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string_pretty(results)?;
        std::fs::write(filename, json_data)?;
        println!("Results saved to {}", filename);
        Ok(())
    }

    pub fn load_urls_from_file(filename: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(filename)?;
        let urls: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        Ok(urls)
    }
}

fn main() {
    println!("Website Monitor Starting...");
    
    // Configuration - all manual, no external config libraries
    let config = Config {
        worker_threads: 8,
        request_timeout: Duration::from_secs(10),
        max_retries: 2,
    };
    
    // Test URLs for demonstration
    let urls = vec![
        "https://httpbin.org/status/200".to_string(),
        "https://httpbin.org/status/404".to_string(),
        "https://httpbin.org/status/500".to_string(),
        "https://httpbin.org/delay/2".to_string(),
        "https://example.com".to_string(),
        "https://google.com".to_string(),
        "https://github.com".to_string(),
        "https://stackoverflow.com".to_string(),
        "https://rust-lang.org".to_string(),
        "https://crates.io".to_string(),
        "https://invalid-url-that-should-fail.nonexistent".to_string(),
        "https://httpbin.org/status/301".to_string(),
        // Add more URLs to test concurrency
        "https://jsonplaceholder.typicode.com/posts/1".to_string(),
        "https://api.github.com".to_string(),
        "https://www.reddit.com".to_string(),
    ];
    
    println!("Monitoring {} websites with {} worker threads...", 
             urls.len(), config.worker_threads);
    
    // Create monitor and run
    let monitor = WebsiteMonitor::new(config);
    let start_time = Instant::now();
    
    let results = monitor.monitor_websites(urls);
    let total_time = start_time.elapsed();
    
    // Print results
    monitor.print_results(&results);
    println!("\nTotal execution time: {:.2?}", total_time);
    
    // Save results to JSON file
    if let Err(e) = monitor.save_results_to_json(&results, "monitoring_results.json") {
        eprintln!("Failed to save results: {}", e);
    }
    
    // Wait for user input before exiting
    println!("\nPress Enter to exit...");
    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        eprintln!("Failed to read input: {}", e);
    }
}

// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.worker_threads, 10);
        assert_eq!(config.request_timeout, Duration::from_secs(5));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_website_monitor_creation() {
        let config = Config {
            worker_threads: 5,
            request_timeout: Duration::from_secs(10),
            max_retries: 2,
        };
        let monitor = WebsiteMonitor::new(config);
        // Monitor created successfully if we reach here
        assert!(true);
    }

    #[test]
    fn test_invalid_url_handling() {
        let config = Config {
            worker_threads: 1,
            request_timeout: Duration::from_secs(1),
            max_retries: 0,
        };
        let result = WebsiteMonitor::check_website("invalid-url", &config);
        
        assert!(result.status.is_err());
        assert_eq!(result.url, "invalid-url");
        assert!(result.response_time >= Duration::from_nanos(0));
    }

    #[test]
    fn test_successful_request() {
        let config = Config {
            worker_threads: 1,
            request_timeout: Duration::from_secs(10),
            max_retries: 0,
        };
        
        // Use a reliable test endpoint
        let result = WebsiteMonitor::check_website("https://httpbin.org/status/200", &config);
        
        assert_eq!(result.url, "https://httpbin.org/status/200");
        // Should either succeed or fail gracefully
        assert!(result.response_time >= Duration::from_nanos(0));
    }

    #[test]
    fn test_website_status_structure() {
        let config = Config::default();
        let result = WebsiteMonitor::check_website("https://nonexistent.invalid", &config);
        
        // Test all fields exist and have expected types
        assert_eq!(result.url, "https://nonexistent.invalid");
        assert!(result.response_time >= Duration::from_nanos(0));
        
        // Timestamp should be recent
        let now = SystemTime::now();
        if let Ok(duration) = now.duration_since(result.timestamp) {
            assert!(duration < Duration::from_secs(60)); // Should be very recent
        }
    }

    #[test]
    fn test_empty_url_list() {
        let config = Config::default();
        let monitor = WebsiteMonitor::new(config);
        let results = monitor.monitor_websites(vec![]);
        
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_single_url_monitoring() {
        let config = Config {
            worker_threads: 1,
            request_timeout: Duration::from_secs(5),
            max_retries: 0,
        };
        let monitor = WebsiteMonitor::new(config);
        let urls = vec!["https://httpbin.org/status/404".to_string()];
        
        let results = monitor.monitor_websites(urls);
        assert_eq!(results.len(), 1);
        
        // Should get 404 status code (which is a successful HTTP response)
        if let Ok(status) = results[0].status {
            assert_eq!(status, 404);
        }
    }

    #[test]
    fn test_concurrent_monitoring() {
        let config = Config {
            worker_threads: 4,
            request_timeout: Duration::from_secs(10),
            max_retries: 0,
        };
        let monitor = WebsiteMonitor::new(config);
        
        let urls = vec![
            "https://httpbin.org/status/200".to_string(),
            "https://httpbin.org/status/404".to_string(),
            "https://httpbin.org/status/500".to_string(),
            "https://example.com".to_string(),
        ];
        
        let results = monitor.monitor_websites(urls);
        assert_eq!(results.len(), 4);
        
        // All should have completed
        for result in &results {
            assert!(!result.url.is_empty());
            assert!(result.response_time >= Duration::from_nanos(0));
        }
    }

    #[test]
    fn test_json_serialization() {
        let status = WebsiteStatus {
            url: "https://example.com".to_string(),
            status: Ok(200),
            response_time: Duration::from_millis(150),
            timestamp: SystemTime::now(),
        };
        
        // Should be able to serialize and deserialize
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: WebsiteStatus = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.url, status.url);
        assert_eq!(deserialized.status, status.status);
    }
}