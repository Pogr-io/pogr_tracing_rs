use pogr_tracing_rs::PogrAppender;


#[tokio::test]
async fn test_pogr_appender_new() {

    // Setup mock environment variables
    std::env::set_var("POGR_CLIENT", "test_client");
    std::env::set_var("POGR_BUILD", "test_build");
    

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
