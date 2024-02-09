use criterion::{criterion_group, criterion_main, Criterion};
use pogr_tracing_rs::{PogrAppender, PogrLayer};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::Registry;
use mockito::{Server};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use criterion::black_box;

fn setup_mock_server() -> String {
    let mut server = Server::new();
    let init_response = serde_json::json!({
        "success": true,
        "payload": {"session_id": "test_session_id"}
    }).to_string();

    server.mock("POST", "/v1/intake/init")
          .with_status(200)
          .with_header("content-type", "application/json")
          .with_body(&init_response)
          .create();

    let log_response_success = serde_json::json!({
        "success": true,
        "payload": {"log_id": "test_log_id"}
    }).to_string();

    server.mock("POST", "/v1/intake/logs")
          .with_status(200)
          .with_header("content-type", "application/json")
          .with_body(&log_response_success)
          .create();

    server.url()
}


async fn log_event(base_url: &str) {
    let init_endpoint = format!("{}/v1/intake/init", base_url.trim_end_matches('/'));
    let logs_endpoint = format!("{}/v1/intake/logs", base_url.trim_end_matches('/'));

    let appender = PogrAppender::new(Some(init_endpoint), Some(logs_endpoint)).await;
    let layer = PogrLayer {
        appender: Arc::new(Mutex::new(appender)),
    };
    let subscriber = Registry::default().with(layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
    tracing::info!("This is a benchmark test log");
}

fn criterion_benchmark(c: &mut Criterion) {
    let base_url = setup_mock_server(); // Setup mock server and get URL

    // Create a new Tokio runtime

    // Initialize Tokio runtime
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    c.bench_function("pogr_logging", |b| {
        b.iter(|| {
            // Use the runtime to block on the async function
            runtime.block_on(black_box(log_event(&base_url)))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
