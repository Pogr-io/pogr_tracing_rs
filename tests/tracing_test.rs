// Import necessary modules and structs from the pogr_tracing_rs crate and the tracing ecosystem.
use pogr_tracing_rs::{PogrAppender, PogrLayer};
use tracing::{info, subscriber::set_global_default};
use tracing_subscriber::Registry;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tokio::sync::Mutex;
use std::sync::Arc;

// Define an asynchronous test function using the tokio runtime.
#[tokio::test]
async fn test_pogr_tracing_appender() {
    // Set mock environment variables required for the PogrAppender authentication process.
    std::env::set_var("POGR_ACCESS", "test_access_key");
    std::env::set_var("POGR_SECRET", "test_secret_key");
    
    // Initialize a mock server to simulate the POGR service's initialization endpoint.
    let mut mock_server = mockito::Server::new();
    let base_url = mock_server.url(); // Retrieve the base URL of the mock server.
    // Construct the full URL for the initialization endpoint.
    let init_endpoint = format!("{}/v1/intake/init", base_url.trim_end_matches('/'));

    // Define a mock response for the initialization request indicating a successful session creation.
    let init_response = serde_json::json!({
        "success": true,
        "payload": {
            "session_id": "test_session_id" // Mock session ID to be returned by the service.
        }
    });

    // Configure the mock server to respond to POST requests at the initialization endpoint.
    let _m = mock_server.mock("POST", "/v1/intake/init")
        .match_header("POGR_ACCESS", "test_access_key") // Expect specific request headers.
        .match_header("POGR_SECRET", "test_secret_key")
        .match_header("Content-Type", "application/json")
        .with_status(200) // HTTP success status code.
        .with_header("content-type", "application/json") // Response content type.
        .with_body(init_response.to_string()) // JSON body of the response.
        .create(); // Activate the mock.

    // Define a mock response for successful log submissions.
    let log_response_success = serde_json::json!({
        "success": true,
        "payload": {
            "log_id": "test_log_id" // Mock log ID indicating successful log submission.
        }
    });
    
    // Configure the mock server to respond to POST requests at the logs submission endpoint.
    let _m_success = mock_server.mock("POST", "/v1/intake/logs")
        .match_header("INTAKE_SESSION_ID", "test_session_id") // Expect the session ID header.
        .match_header("Content-Type", "application/json")
        .with_status(200) // HTTP success status code.
        .with_header("content-type", "application/json") // Response content type.
        .with_body(log_response_success.to_string()) // JSON body of the response.
        .create(); // Activate the mock.

    // Initialize the PogrAppender with the mocked initialization endpoint.
    // The logs endpoint is not specified in this example, assuming it's either not needed for the test or
    // set internally by the PogrAppender based on the initialization response or default configuration.
    let appender = PogrAppender::new(Some(init_endpoint.clone()), None).await;

    // Verify the PogrAppender has been initialized with the correct session ID and init endpoint.
    assert_eq!(appender.session_id, "test_session_id");
    assert_eq!(appender.init_endpoint, init_endpoint);

    // Wrap the appender in a PogrLayer and make it shareable across threads and asynchronous tasks.
    let layer = PogrLayer {
        appender: Arc::new(Mutex::new(appender)),
    };

    // Create a subscriber that combines the default registry with the PogrLayer for capturing log events.
    let subscriber = Registry::default().with(layer);

    // Set the constructed subscriber as the global default to capture log events throughout the application.
    set_global_default(subscriber).expect("Failed to set subscriber");

    // Emit a log event using the `info` macro from the `tracing` crate.
    // The PogrLayer should capture this event and forward it to the mocked POGR service.
    info!(service = "TestService", "This is a test log message");
}
