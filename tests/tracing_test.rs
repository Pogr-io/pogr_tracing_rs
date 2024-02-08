use pogr_tracing_rs::PogrAppender;
use pogr_tracing_rs::PogrLayer;
use tracing::{info, subscriber::set_global_default};
use tracing_subscriber::Registry;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_pogr_tracing_appender() {

    // Setup mock environment variables
    std::env::set_var("POGR_ACCESS", "test_access_key");
    std::env::set_var("POGR_SECRET", "test_secret_key");
    

    // Mock init endpoint
    let mut mock_server = mockito::Server::new();
    let base_url = mock_server.url();
    let init_endpoint = format!("{}/v1/intake/init", base_url.trim_end_matches('/'));
    let init_response = serde_json::json!({
        "success": true,
        "payload": {
            "session_id": "test_session_id"
        }
    });
    let _m = mock_server.mock("POST", "/v1/intake/init")
        .match_header("POGR_ACCESS", "test_access_key")
        .match_header("POGR_SECRET", "test_secret_key")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(init_response.to_string())
        .create();

    let log_response_success = serde_json::json!({
        "success": true,
        "payload": {
            "log_id": "test_log_id"
        }
    });
    
    let _m_success = mock_server.mock("POST", "/v1/intake/logs")
        .match_header("INTAKE_SESSION_ID", "test_session_id")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(log_response_success.to_string())
        .create();

    // Test PogrAppender::new with the mock endpoint
    let appender = PogrAppender::new(Some(init_endpoint.clone()), None).await;

    assert_eq!(appender.session_id, "test_session_id");
    assert_eq!(appender.init_endpoint, init_endpoint);

    let layer = PogrLayer {
        appender: Arc::new(Mutex::new(appender)),
    };

    let subscriber = Registry::default().with(layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

    // Emit a log event
    info!(service = "TestService", "This is a test log message");

}
