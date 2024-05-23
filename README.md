# rust-vector-logger

This is a simple logger that logs to vector. It is useful for testing and debugging. It uses tokio to send logs to the vector.

## Usage

```rust
use rust_vector_logger::Logger;

#[tokio::main]
async fn main() {
    let host = "127.0.0.1"; // The vector host address
    let port = 12345; // The vector port
    let level = "INFO"; // The log level

    let mut logger = Logger::init("AppName", &level, &host, port).await.unwrap();
    logger.info("Hello, world!");
    logger.infof(format_args!("Hello, {}", "world"));

    logger.debug("This is a debug message");
    logger.warnf(format_args!("This is a warning message for {}", "you"));
```
