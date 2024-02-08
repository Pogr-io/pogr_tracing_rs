#![allow(dead_code)]


use tracing::{Event, Subscriber, error};
use tracing_subscriber::{layer::Context, Layer, registry::LookupSpan};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::{env, fmt};
use tracing::field::{Field, Visit};
use std::collections::HashMap;
use tracing::Metadata;
use serde_json::{json, to_value, Value};

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
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.fields.insert(field.name().to_string(), json!(value.to_string()));
    }
    

    fn record_debug(&mut self, field: &Field, value: &(dyn fmt::Debug)) {
        self.fields.insert(field.name().to_string(), json!(format!("{:?}", value)));
    }

}

pub struct PogrAppender {
    pub client: Client,
    pub service_name: String,
    pub environment: String,
    pub service_type: String,
    pub session_id: String,
    pub logs_endpoint: String,
    pub init_endpoint: String,
}

#[derive(Serialize)]
struct InitRequest {}

#[derive(Serialize, Deserialize)]
struct InitResponse {
    success: bool,
    payload: InitPayload,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug)]
struct LogResponse {
    success: bool,
    payload: LogPayload,
}

#[derive(Serialize, Deserialize, Debug)]
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

        let service_name = env::var("SERVICE_NAME").unwrap_or_else(|_| env::current_exe().unwrap().file_name().unwrap().to_str().unwrap().to_owned());
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_owned());
        let service_type = env::var("SERVICE_TYPE").unwrap_or_else(|_| "service".to_owned());

        let init_endpoint_url = init_endpoint
            .or_else(|| env::var("POGR_INIT_ENDPOINT").ok())
            .unwrap_or_else(|| "https://api.pogr.io/v1/intake/init".to_string());
        let logs_endpoint_url = logs_endpoint
            .or_else(|| env::var("POGR_LOGS_ENDPOINT").ok())
            .unwrap_or_else(|| "https://api.pogr.io/v1/intake/logs".to_string());

        let pogr_client = env::var("POGR_ACCESS").expect("POGR_ACCESS must be set");
        let pogr_build = env::var("POGR_SECRET").expect("POGR_SECRET must be set");

        let init_response: InitResponse = client.post(&init_endpoint_url)
            .header("POGR_ACCESS", pogr_client)
            .header("POGR_SECRET", pogr_build)
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
                service_name,
                environment,
                service_type,
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

        tokio::spawn(async move {
            let appender = appender.lock().await;
            

        let log_request = LogRequest {
            service: appender.service_name.clone(),
            environment: appender.environment.clone(),
            severity: metadata.level().to_string(),
            r#type: appender.service_type.clone(),
            log: "rust tracing log captured".to_string(),
            data: serialize_metadata(metadata),
            tags: serde_json::to_value(visitor.fields).unwrap_or_else(|_| serde_json::json!({})),
        };
            appender.log(log_request).await;
        });
    }
}
