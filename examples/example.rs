#![allow(dead_code)]

use pogr_tracing_rs::{PogrAppender, PogrLayer};
use serde_json::{json, to_string};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{event, field, Level};
use tracing_subscriber::{layer::SubscriberExt, Registry};

#[tokio::main]
async fn main() {
    // Setup POGR Appender
    let appender = PogrAppender::new(None, None).await;

    let layer = PogrLayer {
        appender: Arc::new(Mutex::new(appender)),
    };

    let subscriber = Registry::default().with(layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    // Correct way to log structured data with tracing
    let json_data = json!({"example_field": "example_value"});
    event!(Level::INFO, json_data = field::debug(&json_data), "Example event with JSON data");

    // Alternatively, if you want to include JSON as a part of the message
    let json_string = to_string(&json_data).unwrap_or_default();
    event!(Level::INFO, %json_string, "Example event with JSON data as string");

    // Simulate application logic to allow for asynchronous log processing
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
