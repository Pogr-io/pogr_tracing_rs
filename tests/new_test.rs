// Import the necessary module from the `pogr_tracing_rs` crate.
use pogr_tracing_rs::PogrAppender;

// Attribute macro to define an asynchronous test using the tokio runtime.
#[tokio::test]
async fn test_pogr_appender_new() {
    // Set mock environment variables that would be used for authentication with the POGR service.
    // These variables mimic what would be expected in a live environment for accessing the POGR API.
    std::env::set_var("POGR_ACCESS", "test_access_key");
    std::env::set_var("POGR_SECRET", "test_secret_key");
    
    // Initialize a mock server using the `mockito` library to simulate the initialization endpoint.
    let mut mock_server = mockito::Server::new();
    // Retrieve the base URL of the mock server.
    let base_url = mock_server.url();
    // Construct the initialization endpoint URL by appending the path to the base URL.
    let init_endpoint = format!("{}/v1/intake/init", base_url.trim_end_matches('/'));

    // Define the expected response for the initialization request.
    // This response includes a success flag and a payload with a test session ID.
    let init_response = serde_json::json!({
        "success": true,
        "payload": {
            "session_id": "test_session_id"
        }
    });

    // Setup a mock response for POST requests to the initialization endpoint.
    // The mock is configured to expect specific headers and respond with a predefined JSON payload.
    let _m = mock_server.mock("POST", "/v1/intake/init")
        .match_header("POGR_ACCESS", "test_access_key")
        .match_header("POGR_SECRET", "test_secret_key")
        .match_header("Content-Type", "application/json")
        .with_status(200) // HTTP status code for success.
        .with_header("content-type", "application/json") // Expected response content type.
        .with_body(init_response.to_string()) // The body of the response, converted to a string.
        .create(); // Creates and registers the mock.

    // Execute the `new` method of `PogrAppender` with the mocked initialization endpoint.
    // The logs endpoint is set to None because it's not the focus of this test.
    let appender = PogrAppender::new(Some(init_endpoint.clone()), None).await;

    // Verify that the `session_id` of the `PogrAppender` matches the one from the mock response.
    assert_eq!(appender.session_id, "test_session_id");
    // Verify that the `init_endpoint` of the `PogrAppender` is correctly set to the mocked endpoint URL.
    assert_eq!(appender.init_endpoint, init_endpoint);
}
