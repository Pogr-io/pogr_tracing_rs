use pogr_tracing_rs::PogrAppender;
use pogr_tracing_rs::LogRequest;

#[tokio::test]
async fn test_pogr_appender_log() {

    // Mock log endpoint
    let mut mock_server = mockito::Server::new();
    let base_url = mock_server.url();
    let logs_endpoint = format!("{}/v1/intake/logs", base_url.trim_end_matches('/'));
    let log_response_success = serde_json::json!({
        "success": true,
        "payload": {
            "log_id": "test_log_id"
        }
    });
    
    let _m_success = mock_server.mock("POST", "/v1/intake/logs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(log_response_success.to_string())
        .create();

    // Initialize PogrAppender with mocked endpoints
    let appender = PogrAppender {
        client: reqwest::Client::new(),
        session_id: "test_session_id".to_string(),
        logs_endpoint: logs_endpoint.clone(),
        init_endpoint: "".to_string(),
    };

    // Create a log request
    let log_request = LogRequest {
        service: "TestService".to_string(),
        environment: "test".to_string(),
        severity: "INFO".to_string(),
        r#type: "TestType".to_string(),
        log: "This is a test log".to_string(),
        data: serde_json::json!({}),
        tags: serde_json::json!({}),
    };

    // Test PogrAppender::log with a successful response
    appender.log(log_request).await;

    // Additional assertions can be added here, for example, checking that no error was logged
}
