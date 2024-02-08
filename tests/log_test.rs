// Import necessary modules from the `pogr_tracing_rs` crate.
use pogr_tracing_rs::PogrAppender;
use pogr_tracing_rs::LogRequest;

// Attribute macro to define an asynchronous test using the tokio runtime.
#[tokio::test]
async fn test_pogr_appender_log() {
    // Initialize a mock server using the `mockito` library to simulate the log endpoint.
    let mut mock_server = mockito::Server::new();
    // Retrieve the base URL of the mock server.
    let base_url = mock_server.url();
    // Construct the logs endpoint URL by appending the path to the base URL.
    let logs_endpoint = format!("{}/v1/intake/logs", base_url.trim_end_matches('/'));

    // Define the expected successful response for a log submission.
    let log_response_success = serde_json::json!({
        "success": true,
        "payload": {
            "log_id": "test_log_id"
        }
    });

    // Setup a mock response for POST requests to the logs endpoint that matches specific headers.
    let _m_success = mock_server.mock("POST", "/v1/intake/logs")
        .match_header("INTAKE_SESSION_ID", "test_session_id")
        .match_header("Content-Type", "application/json")
        .with_status(200) // HTTP status code for success.
        .with_header("content-type", "application/json") // Response content type.
        .with_body(log_response_success.to_string()) // Response body.
        .create(); // Create the mock.

    // Initialize the `PogrAppender` with the mock server's logs endpoint and a predefined session ID.
    let appender = PogrAppender {
        client: reqwest::Client::new(), // HTTP client for making requests.
        service_name: "test_pogr_appender_log".to_string(), // Name of the service for logging context.
        environment: "testing".to_string(), // Environment the service is running in.
        service_type: "test".to_string(), // Type of the service.
        session_id: "test_session_id".to_string(), // Session ID for authentication with the log service.
        logs_endpoint: logs_endpoint.clone(), // URL of the mocked logs endpoint.
        init_endpoint: "".to_string(), // Initialization endpoint is not needed for this test.
    };

    // Construct a log request with predefined values.
    let log_request = LogRequest {
        service: "TestService".to_string(), // Name of the service generating the log.
        environment: "test".to_string(), // Environment of the service.
        severity: "INFO".to_string(), // Severity level of the log.
        r#type: "TestType".to_string(), // Type of the log.
        log: "This is a test log".to_string(), // Log message.
        data: serde_json::json!({}), // Additional data associated with the log.
        tags: serde_json::json!({}), // Tags for categorizing the log.
    };

    // Execute the `log` function of `PogrAppender` with the constructed log request.
    // The function is awaited to ensure the asynchronous operation completes.
    appender.log(log_request).await;
}
