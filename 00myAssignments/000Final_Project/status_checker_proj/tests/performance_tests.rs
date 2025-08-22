use status_checker_proj;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;
use website_monitor::{Config, WebsiteMonitor};

fn bench_concurrent_requests_ureq(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_requests_ureq");
    
    // Use shorter timeout for benchmarking (these will mostly fail, but that's ok)
    for &num_urls in &[10, 25, 50] {
        group.bench_with_input(
            BenchmarkId::new("monitor_websites", num_urls),
            &num_urls,
            |b, &num_urls| {
                let config = Config {
                    worker_threads: 8,
                    request_timeout: Duration::from_millis(500), // Short for benchmarking
                    max_retries: 0,
                };
                let monitor = WebsiteMonitor::new(config);
                
                // Use non-existent domains that will fail quickly
                let urls: Vec<String> = (0..num_urls)
                    .map(|i| format!("https://nonexistent{}.invalid", i))
                    .collect();
                
                b.iter(|| {
                    let results = monitor.monitor_websites(black_box(urls.clone()));
                    black_box(results);
                });
            },
        );
    }
    group.finish();
}

fn bench_worker_thread_scaling_ureq(c: &mut Criterion) {
    let mut group = c.benchmark_group("worker_threads_ureq");
    
    for &num_threads in &[1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                let config = Config {
                    worker_threads: num_threads,
                    request_timeout: Duration::from_millis(500),
                    max_retries: 0,
                };
                let monitor = WebsiteMonitor::new(config);
                
                // Use a mix of URLs that will fail quickly
                let urls: Vec<String> = (0..20)
                    .map(|i| format!("https://test{}.invalid", i))
                    .collect();
                
                b.iter(|| {
                    let results = monitor.monitor_websites(black_box(urls.clone()));
                    black_box(results);
                });
            },
        );
    }
    group.finish();
}

fn bench_real_requests(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_requests");
    
    // Benchmark with actual working URLs (use sparingly)
    group.bench_function("httpbin_requests", |b| {
        let config = Config {
            worker_threads: 4,
            request_timeout: Duration::from_secs(10),
            max_retries: 0,
        };
        let monitor = WebsiteMonitor::new(config);
        
        let urls = vec![
            "https://httpbin.org/status/200".to_string(),
            "https://httpbin.org/status/404".to_string(),
            "https://example.com".to_string(),
        ];
        
        b.iter(|| {
            let results = monitor.monitor_websites(black_box(urls.clone()));
            black_box(results);
        });
    });
    
    group.finish();
}

criterion_group!(benches, 
    bench_concurrent_requests_ureq, 
    bench_worker_thread_scaling_ureq,
    bench_real_requests
);
criterion_main!(benches);
