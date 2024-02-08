# README for `pogr_tracing_rs`

`pogr_tracing_rs` is a Rust crate designed to facilitate easy integration of the POGR Analytics Platform with applications written in Rust. This crate leverages the `tracing` ecosystem to collect and send structured log data to the POGR platform, offering a robust solution for monitoring and debugging Rust applications.

## License

This project is licensed under the MIT License.

## Features

- Easy to integrate with existing Rust applications using the `tracing` ecosystem.
- Asynchronous log data submission to the POGR platform, ensuring minimal performance impact on the application.
- Structured logging with support for various data types, including integers, floating-point numbers, booleans, strings, and errors.
- Customizable log levels and metadata, allowing for detailed and contextualized logging information.
- Secure session management with the POGR platform, including automatic session initialization.

## Prerequisites

Before you begin, ensure you have the following installed:

- Rust programming language (utilizing 1.70.0 or higher).
- `tokio` runtime for asynchronous support.
- Access to the POGR platform with valid `POGR_ACCESS` and `POGR_SECRET` environment variables set.

## Environment Variables

### Required Variables

- **`POGR_ACCESS`**: This is a required environment variable that specifies the access key used to authenticate with the POGR platform. It's essential for establishing a secure connection between your application and the POGR services, ensuring that your logs are transmitted securely and are only accessible by authorized users.

- **`POGR_SECRET`**: This required variable is the secret key corresponding to your `POGR_ACCESS` key. It is used in conjunction with the access key to authenticate requests to the POGR platform. The secret key should be kept confidential to prevent unauthorized access to your logging data.

### Optional Variables

- **`SERVICE_NAME`**: This optional variable allows you to specify the name of the service that is sending logs to the POGR platform. If not set, the crate will attempt to use the name of the current executable as the service name. Specifying a service name is useful for identifying and filtering logs from different services within the same project or infrastructure.

- **`ENVIRONMENT`**: The `ENVIRONMENT` variable lets you specify the deployment environment of your application, such as `development`, `testing`, `staging`, or `production`. This information is included in the logs and can be used to differentiate logs from the same service running in different environments.

- **`SERVICE_TYPE`**: With this variable, you can define the type of service that's generating the logs, such as `web`, `database`, `cache`, etc. This categorization helps in organizing and filtering logs based on the service type, providing clearer insights into the behavior and issues of different components of your system.

- **`POGR_INIT_ENDPOINT`** and **`POGR_LOGS_ENDPOINT`**: These optional variables allow for customization of the endpoints to which initialization and log data are sent, respectively. By default, the crate uses the POGR platform's standard endpoints, but you can override them with these variables if you need to direct requests to a different address (e.g., a proxy or a testing environment).

## Installation

Add `pogr_tracing_rs` to your `Cargo.toml` file:

```toml
[dependencies]
pogr_tracing_rs = "0.1.0"
tracing = "0.1"
tokio = { version = "1", features = ["full"] }
reqwest = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Usage

### Basic Setup

To start using `pogr_tracing_rs` in your project, first, set up the tracing subscriber and add the `PogrLayer` to your application. Here is a basic example:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use pogr_tracing_rs::PogrLayer;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // Initialize the PogrAppender with optional endpoints
    let appender = pogr_tracing_rs::PogrAppender::new(None, None).await;
    let pogr_layer = PogrLayer {
        appender: Arc::new(Mutex::new(appender)),
    };

    // Set up the tracing subscriber
    tracing_subscriber::registry()
        .with(pogr_layer)
        .init();

    // Your application logic here
}
```

### Logging Events

To log events, use the `tracing` macros. The `PogrLayer` automatically captures these events and forwards them to the POGR platform:

```rust
use tracing::{info, warn, error};

fn perform_operation() {
    info!("This is an informational message");
    warn!("This is a warning message");
    error!("This is an error message");
}
```

### Custom Logging

You can also create custom log events with structured data:

```rust
use tracing::info;

fn process_data(data: &str) {
    info!(data = data, "Processing data");
}
```

## Customization

You can customize the POGR session initialization by providing custom `init_endpoint` and `logs_endpoint` URLs when creating the `PogrAppender`. Additionally, you may want to adjust the `LogRequest` structure and the serialization logic to fit your specific logging requirements.

## Contributing

Contributions to `pogr_tracing_rs` are welcome. Please submit your pull requests or issues to the project repository.

## Author

- Randolph William Aarseth II