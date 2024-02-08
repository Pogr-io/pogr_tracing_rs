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

- Rust programming language (latest stable version recommended).
- `tokio` runtime for asynchronous support.
- Access to the POGR platform with valid `POGR_CLIENT` and `POGR_BUILD` environment variables set.

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