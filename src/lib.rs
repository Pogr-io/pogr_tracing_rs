#![allow(dead_code)]


use tracing::{Event, Subscriber, error};
use tracing_subscriber::{layer::Context, Layer, registry::LookupSpan};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;
use std::env;
use tracing::field::{Field, Visit};
use std::fmt;
use std::collections::HashMap;
use serde_json::json;
use tracing::Metadata;
use serde_json::to_value;

struct JsonVisitor {
    fields: HashMap<String, Value>,
}

impl JsonVisitor {
    fn new() -> Self {
        JsonVisitor {
            fields: HashMap::new(),
        }
    }
}

impl Visit for JsonVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        // Note: JSON representation of floating-point numbers can be lossy.
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        // Directly capture the error message without downcasting
        self.fields.insert(field.name().to_string(), json!(value.to_string()));
    }
    

    // Custom method to record debug information
    fn record_debug(&mut self, field: &Field, value: &(dyn fmt::Debug)) {
        // Use Debug trait to convert value to a string representation
        self.fields.insert(field.name().to_string(), json!(format!("{:?}", value)));
    }

}

pub struct PogrAppender {
    pub client: Client,
    pub session_id: String,
    pub logs_endpoint: String,
    pub init_endpoint: String
}

#[derive(Serialize)]
struct InitRequest {}

#[derive(Deserialize)]
struct InitResponse {
    success: bool,
    payload: InitPayload,
}

#[derive(Deserialize)]
struct InitPayload {
    session_id: String,
}

#[derive(Serialize, Debug)]
pub struct LogRequest {
    pub service: String,
    pub environment: String,
    pub severity: String,
    pub r#type: String,
    pub log: String,
    pub data: serde_json::Value,
    pub tags: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct LogResponse {
    success: bool,
    payload: LogPayload,
}

#[derive(Deserialize, Debug)]
struct LogPayload {
    log_id: String,
}

pub struct PogrLayer {
   pub appender: Arc<Mutex<PogrAppender>>, // Using Arc<Mutex<T>> for shared state across threads.
}

fn serialize_metadata(metadata: &Metadata) -> Value {
    let mut map = HashMap::new();

    map.insert("name", Value::from(metadata.name()));
    map.insert("target", Value::from(metadata.target()));
    map.insert("level", Value::from(format!("{:?}", metadata.level())));
    map.insert("file", metadata.file().map(Value::from).unwrap_or(Value::Null));
    map.insert("line", metadata.line().map(|line| Value::from(line as i64)).unwrap_or(Value::Null));

    // Convert the HashMap<&str, Value> to Value directly using to_value
    to_value(map).unwrap_or_else(|_| Value::Null)
}

impl PogrAppender {
    pub async fn new(init_endpoint: Option<String>, logs_endpoint: Option<String>) -> Self {
        let client = Client::new();

        // Use environment variables for POGR_CLIENT and POGR_BUILD
        let pogr_client = env::var("POGR_CLIENT").expect("POGR_CLIENT must be set");
        let pogr_build = env::var("POGR_BUILD").expect("POGR_BUILD must be set");

        // Determine the endpoint URL based on the POGR_ENDPOINT environment variable
        let init_endpoint_url = init_endpoint
        .or_else(|| env::var("POGR_INIT_ENDPOINT").ok())
        .unwrap_or_else(|| "https://api.pogr.io/v1/intake/init".to_string());
        let logs_endpoint_url = logs_endpoint
        .or_else(|| env::var("POGR_LOGS_ENDPOINT").ok())
        .unwrap_or_else(|| "https://api.pogr.io/v1/intake/logs".to_string());


        let init_response: InitResponse = client.post(&init_endpoint_url)
            .header("POGR_CLIENT", pogr_client)
            .header("POGR_BUILD", pogr_build)
            .header("Content-Type", "application/json")
            .send()
            .await
            .expect("Failed to send init request")
            .json()
            .await
            .expect("Failed to deserialize init response");

        if init_response.success {
            PogrAppender {
                client,
                session_id: init_response.payload.session_id,
                logs_endpoint: logs_endpoint_url,
                init_endpoint: init_endpoint_url,
            }
        } else {
            panic!("Failed to initialize POGR session");
        }
    }

    pub async fn log(&self, log_request: LogRequest) {

        let log_endpoint = self.logs_endpoint.clone();


        let response: LogResponse = self.client.post(&log_endpoint)
            .header("INTAKE_SESSION_ID", &self.session_id)
            .header("Content-Type", "application/json")
            .json(&log_request)
            .send()
            .await
            .expect("Failed to send log request")
            .json()
            .await
            .expect("Failed to deserialize log response");

        if !response.success {
            error!("Failed to log to POGR: {:?}", response);
        }
    }
}

impl<S> Layer<S> for PogrLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let appender = Arc::clone(&self.appender);
        let metadata = event.metadata();

        let mut visitor = JsonVisitor::new();
        event.record(&mut visitor);

        // Convert the event to a LogRequest struct. This requires extracting the relevant information from `event`.
        // For simplicity, this example does not show the conversion process. You'll need to implement it based on your LogRequest structure and what information you want to log.
        let log_request = LogRequest {
            // Fill in the fields as necessary. This is an example and likely needs adjustment.
            service: "YourServiceName".to_string(),
            environment: "production".to_string(),
            severity: metadata.level().to_string(),
            r#type: "api".to_string(),
            log: "rust tracing log captured".to_string(),
            data: serialize_metadata(metadata),
            tags: serde_json::to_value(visitor.fields).unwrap_or_else(|_| serde_json::json!({})),
        };

        // Spawn a new task for sending the log message. This is necessary to avoid blocking the current thread.
        tokio::spawn(async move {
            let appender = appender.lock().await;
            appender.log(log_request).await;
        });
    }
}
