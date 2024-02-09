//! `pogr_tracing_rs` is a Rust crate designed to facilitate structured logging and tracing
//! integration with the POGR analytics platform. This crate provides a set of tools that enable
//! Rust applications to send their log data to the POGR service in a structured and efficient manner.
//! 
//! The main components of `pogr_tracing_rs` include:
//! 
//! - `JsonVisitor`: A utility for collecting and structuring log event fields into JSON format. It
//!   allows for the serialization of log data into a format that is compatible with many logging
//!   and analytics services, including POGR.
//! 
//! - `PogrAppender`: Responsible for sending log messages to the POGR platform. It handles the
//!   construction and submission of log data, including session management and authentication
//!   with the POGR API.
//! 
//! - `PogrLayer`: A tracing layer that integrates with the `tracing` ecosystem, capturing log
//!   events, processing them with `JsonVisitor`, and forwarding them to the POGR service via
//!   `PogrAppender`. This layer enables asynchronous log data submission, minimizing the impact
//!   on application performance.
//! 
//! - Utility functions and structures for session initialization and log submission, ensuring
//!   that log data is accurately represented and securely transmitted to the POGR service.
//! 
//! This crate is designed to be easy to integrate into existing Rust applications, requiring minimal
//! configuration to connect to the POGR platform. It leverages the `tracing` crate for flexible and
//! powerful instrumentation, making it suitable for applications ranging from simple CLI tools to
//! complex web services.
//! 
//! # Getting Started
//! 
//! To use `pogr_tracing_rs`, add it as a dependency in your `Cargo.toml`:
//! 
//! ```toml
//! [dependencies]
//! pogr_tracing_rs = "0.1.0"
//! ```
//! 
//! Then, in your application, set up the `PogrAppender` and `PogrLayer` with the `tracing` subscriber:
//! 
//! ```rust,no_run
//! use pogr_tracing_rs::{PogrLayer, PogrAppender};
//! use tracing_subscriber::{Registry, layer::SubscriberExt};
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//! 
//! #[tokio::main]
//! async fn main() {
//! 
//!     let appender = PogrAppender::new(None, None).await;
//!     let layer = PogrLayer {
//!         appender: Arc::new(Mutex::new(appender)),
//!     };
//! 
//!     let subscriber = Registry::default().with(layer);
//!     tracing::subscriber::set_global_default(subscriber)
//!         .expect("Failed to set global subscriber");
//! }
//! ```
//! 
//! This will enable your application to automatically capture and send log data to the POGR
//! analytics platform, leveraging Rust's async capabilities for efficient logging.
//! 
//! # Features
//! 
//! - Easy integration with the `tracing` ecosystem for Rust applications.
//! - Structured logging with JSON serialization for compatibility with POGR and other logging services.
//! - Asynchronous log data submission for performance optimization.
//! - Flexible configuration to adapt to different environments and application requirements.
//! 
//! `pogr_tracing_rs` is an essential tool for Rust developers looking to enhance their application's
//! logging capabilities with minimal overhead and maximum compatibility with modern logging platforms.


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

/// A `JsonVisitor` is responsible for visiting fields of a log event and collecting
/// their values into a structured format. This structure is particularly useful
/// for converting log event fields into JSON format, which can then be easily
/// serialized and sent to external logging services or stored for analysis.
///
/// The `JsonVisitor` holds a `HashMap` where each entry corresponds to a field
/// in the log event. The key is the field name as a `String`, and the value is
/// a `serde_json::Value`, allowing for representation of structured data in JSON.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use tracing::field::{Field, Visit};
/// use serde_json::{json, Value};
/// use std::collections::HashMap;
///
/// struct JsonVisitor {
///     fields: HashMap<String, Value>,
/// }
///
/// impl JsonVisitor {
///     fn new() -> Self {
///         JsonVisitor {
///             fields: HashMap::new(),
///         }
///     }
/// }
///
/// let mut visitor = JsonVisitor::new();
///
/// // Simulate visiting fields - in practice, this would be done by the tracing framework
/// visitor.fields.insert("level".to_string(), json!("INFO"));
/// visitor.fields.insert("message".to_string(), json!("Application started"));
///
/// assert_eq!(visitor.fields.get("level").unwrap(), &json!("INFO"));
/// assert_eq!(visitor.fields.get("message").unwrap(), &json!("Application started"));
/// ```
///
/// This example demonstrates the creation of a `JsonVisitor` instance and manually
/// inserting log fields into it. In a real-world application, the `tracing` framework
/// would invoke the visitor's methods to record fields dynamically during log events.
pub struct JsonVisitor {
    /// Holds the mapping of field names to their values for a single log event.
    /// Each field name is a unique identifier (a `String`), and its value is
    /// represented as a `serde_json::Value`, which can encompass various JSON
    /// data types (e.g., strings, numbers, arrays, objects).
    pub fields: HashMap<String, Value>,
}

impl JsonVisitor {
    /// Creates a new instance of `JsonVisitor` with an empty `fields` HashMap.
    /// This constructor is typically called at the beginning of a log event
    /// processing sequence, ready to collect field data.
    ///
    /// # Returns
    ///
    /// A new `JsonVisitor` instance with no fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use pogr_tracing_rs::JsonVisitor;
    ///
    /// let visitor = JsonVisitor::new();
    ///
    /// assert!(visitor.fields.is_empty());
    /// ```
    ///
    /// This example illustrates how to create a new `JsonVisitor` and verify
    /// that it initializes with no fields.
    pub fn new() -> Self {
        JsonVisitor {
            fields: HashMap::new(),
        }
    }
}

/// Implementation of the `Visit` trait for `JsonVisitor`.
///
/// This implementation enables `JsonVisitor` to visit fields in a log event
/// and record their values in a structured JSON format. Each method corresponds
/// to a different data type that can be encountered in the fields of a log event,
/// ensuring that a wide range of values can be accurately and efficiently serialized
/// into JSON for logging purposes.
impl Visit for JsonVisitor {
    /// Records a field with an `i64` value.
    ///
    /// # Arguments
    ///
    /// * `field` - The metadata of the field being recorded, including its name.
    /// * `value` - The `i64` value of the field to record.
    ///
    /// Inserts the field and its value into the `JsonVisitor`'s internal `fields` HashMap,
    /// converting the value into a JSON representation.
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    /// Records a field with a `u64` value.
    ///
    /// Similar to `record_i64`, but for unsigned 64-bit integers.
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    /// Records a field with a `f64` value.
    ///
    /// Similar to `record_i64`, but for 64-bit floating-point numbers.
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    /// Records a field with a `bool` value.
    ///
    /// Similar to `record_i64`, but for boolean values.
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    /// Records a field with a `&str` value.
    ///
    /// Similar to `record_i64`, but for string slices.
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    /// Records a field that contains an error.
    ///
    /// # Arguments
    ///
    /// * `field` - The metadata of the field being recorded.
    /// * `value` - The error to record, which implements `std::error::Error`.
    ///
    /// This method converts the error into a string representation before storing it,
    /// ensuring that error information is preserved in the log data.
    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.fields.insert(field.name().to_string(), json!(value.to_string()));
    }

    /// Records a field with a value that implements `fmt::Debug`.
    ///
    /// # Arguments
    ///
    /// * `field` - The metadata of the field being recorded.
    /// * `value` - The value to record, which implements `fmt::Debug`.
    ///
    /// Uses the debug formatting of the value for its representation in the log data,
    /// allowing for complex types to be logged in an easily readable format.
    fn record_debug(&mut self, field: &Field, value: &(dyn fmt::Debug)) {
        self.fields.insert(field.name().to_string(), json!(format!("{:?}", value)));
    }
}

/// Represents an appender for logging to the POGR platform.
///
/// This struct encapsulates the necessary details and client for sending log messages
/// to a specific logging service. It includes configuration like service name, environment,
/// and session management for authenticated requests.
pub struct PogrAppender {
    /// HTTP client used to make requests to the POGR service.
    pub client: Client,
    /// Name of the service generating the logs.
    pub service_name: String,
    /// Deployment environment of the service (e.g., production, development).
    pub environment: String,
    /// Type of the service (e.g., web, database).
    pub service_type: String,
    /// Session ID for authenticating with the POGR service.
    pub session_id: String,
    /// Endpoint URL to which logs are sent.
    pub logs_endpoint: String,
    /// Endpoint URL for session initialization with the POGR service.
    pub init_endpoint: String,
}

/// Represents an empty request structure for initializing a session with the POGR service.
///
/// This struct is serialized and sent as part of the session initialization process.
/// Currently, it does not carry any data, but it is designed to be extensible for future needs.
#[derive(Serialize)]
struct InitRequest {}

/// Represents the response from the POGR service upon session initialization.
///
/// This structure encapsulates the result of the initialization request,
/// indicating success and containing the session payload.
#[derive(Serialize, Deserialize)]
struct InitResponse {
    /// Indicates whether the initialization was successful.
    success: bool,
    /// The payload of the response, containing session details.
    payload: InitPayload,
}

/// Contains the session ID in the initialization response payload.
///
/// This payload is part of the initialization response, providing essential session details.
#[derive(Serialize, Deserialize)]
struct InitPayload {
    /// The session ID assigned by the POGR service for the current session.
    session_id: String,
}

/// Represents a structured log request to be sent to the POGR service.
///
/// This struct contains all necessary details for a log message, including
/// metadata about the service and the log message itself.
#[derive(Serialize, Debug)]
pub struct LogRequest {
    /// Name of the service generating the log.
    pub service: String,
    /// Deployment environment of the service.
    pub environment: String,
    /// Severity level of the log message.
    pub severity: String,
    /// Type of log message (aligns with the service type).
    pub r#type: String,
    /// Text of the log message.
    pub log: String,
    /// Additional structured data associated with the log message.
    pub data: serde_json::Value,
    /// Tags for categorizing and filtering log messages.
    pub tags: serde_json::Value,
}

/// Represents the response from the POGR service upon submitting a log message.
///
/// This structure indicates whether the log submission was successful and includes a payload.
#[derive(Serialize, Deserialize, Debug)]
struct LogResponse {
    /// Indicates whether the log submission was successful.
    success: bool,
    /// The payload of the response, containing details of the log submission.
    payload: LogPayload,
}

/// Contains details of the submitted log message in the log submission response payload.
///
/// This payload provides feedback on the log submission, primarily through the assigned log ID.
#[derive(Serialize, Deserialize, Debug)]
struct LogPayload {
    /// Unique identifier assigned to the submitted log message by the POGR service.
    log_id: String,
}

/// Represents a logging layer that integrates with the POGR analytics platform.
///
/// This layer uses a `PogrAppender` to send log data to the POGR service. It is designed
/// to be added to a `tracing` subscriber to intercept and process log messages.
///
/// The `PogrLayer` holds an `Arc<Mutex<PogrAppender>>`, allowing it to be safely shared
/// across asynchronous tasks and threads. This ensures that log data can be sent concurrently
/// from different parts of an application without data races or other concurrency issues.
pub struct PogrLayer {
    /// Shared state allowing concurrent access to the `PogrAppender` instance.
    /// This appender is responsible for sending log data to the configured POGR endpoints.
    pub appender: Arc<Mutex<PogrAppender>>,
}

/// Serializes metadata from a `tracing` event into a JSON value.
///
/// This function takes metadata from a log event, such as the log level, target,
/// file, and line number, and converts it into a structured JSON representation.
/// This serialized form is suitable for inclusion in log messages sent to external
/// services, providing rich contextual information about each log event.
///
/// # Arguments
///
/// * `metadata` - Metadata from a `tracing` event.
///
/// # Returns
///
/// A `serde_json::Value` representing the serialized metadata.
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
    /// Constructs a new `PogrAppender` with optional custom endpoints.
    ///
    /// Initializes a session with the POGR service using provided or default endpoints.
    /// Requires `POGR_ACCESS` and `POGR_SECRET` environment variables for authentication.
    ///
    /// # Arguments
    ///
    /// * `init_endpoint` - Optional custom URL for the session initialization endpoint.
    /// * `logs_endpoint` - Optional custom URL for the log submission endpoint.
    ///
    /// # Returns
    ///
    /// A new instance of `PogrAppender` configured and initialized for logging.
    ///
    /// # Panics
    ///
    /// Panics if session initialization fails or required environment variables are missing.
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

    /// Asynchronously sends a log message to the POGR service.
    ///
    /// Constructs and sends a log request to the configured POGR endpoint. This method
    /// handles serialization of the log message and metadata, and sends the data using
    /// the internal HTTP client. It ensures that each log message is associated with
    /// the current session via the `INTAKE_SESSION_ID` header.
    ///
    /// # Arguments
    ///
    /// * `log_request` - The log message and associated data to send.
    ///
    /// # Panics
    ///
    /// Panics if the log request fails to send or if the response cannot be deserialized.
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

/// Implements the `Layer` trait from the `tracing` crate for `PogrLayer`.
///
/// This implementation allows `PogrLayer` to interact with the `tracing` ecosystem,
/// capturing log events, processing them, and then forwarding them to an external
/// logging service defined by `PogrAppender`. It leverages asynchronous execution
/// to ensure that the logging process does not block the main application flow.
///
/// The `PogrLayer` captures metadata and structured fields from log events,
/// serializes them into a JSON format, and then sends them to the configured
/// POGR endpoint via the `PogrAppender`. This process is done asynchronously
/// to optimize performance and reduce the impact on application throughput.
///
/// # Type Parameters
///
/// * `S` - The subscriber type. This layer can be added to any subscriber that
///         implements `Subscriber` and `for<'a> LookupSpan<'a>`, allowing it to
///         interact with the span data.
///
/// # Examples
///
/// To use `PogrLayer` with a subscriber:
///
/// ```rust,no_run
/// use tracing_subscriber::{Registry, layer::SubscriberExt};
/// use pogr_tracing_rs::{PogrLayer, PogrAppender};
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// #[tokio::main]
/// async fn main() {
///     // Initialize your PogrAppender here...
///     let appender = PogrAppender::new(None, None).await;
///
///     let layer = PogrLayer {
///         appender: Arc::new(Mutex::new(appender)),
///     };
///
///     let subscriber = Registry::default().with(layer);
///
///     tracing::subscriber::set_global_default(subscriber)
///         .expect("Failed to set global subscriber");
/// }
/// ```
///
/// This example sets up `PogrLayer` with a default `tracing` subscriber, allowing
/// log events to be processed and forwarded to the POGR platform.
impl<S> Layer<S> for PogrLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    /// Responds to log events captured by the `tracing` framework.
    ///
    /// This method is called automatically by the `tracing` framework for each log event.
    /// It extracts event metadata and fields, serializes them into a structured log format,
    /// and forwards them to the POGR platform using the `PogrAppender`. The actual submission
    /// of log data is performed asynchronously to avoid blocking the execution of the event's
    /// source code.
    ///
    /// # Arguments
    ///
    /// * `event` - The log event being processed.
    /// * `_ctx` - The context provided by the `tracing` framework, allowing for interaction
    ///            with the rest of the tracing system, such as querying for active spans.
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
