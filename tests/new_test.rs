use pogr_tracing_rs::PogrAppender;


#[tokio::test]
async fn test_pogr_appender_new() {

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

    // Test PogrAppender::new with the mock endpoint
    let appender = PogrAppender::new(Some(init_endpoint.clone()), None).await;

    assert_eq!(appender.session_id, "test_session_id");
    assert_eq!(appender.init_endpoint, init_endpoint);
    // You should also assert the logs_endpoint is correctly set based on your mock or environment variable
}
