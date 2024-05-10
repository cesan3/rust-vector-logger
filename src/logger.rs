use serde::{Deserialize, Serialize};
use std::fmt::Arguments;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time;

// Message Structure for logging
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub timestamp: String,
    pub application: String,
    pub level: String,
    pub message: String,
}

impl Message {
    pub fn new(timestamp: String, application: &str, level: &str, message: &str) -> Self {
        Message {
            timestamp: timestamp.to_string(),
            application: application.to_string(),
            level: level.to_string(),
            message: message.to_string(),
        }
    }
}

// Logger struct to handle connections
//
pub struct Logger {
    stream: TcpStream,
    application: String,
    level: String,
}

impl Logger {
    // Initialize a new Logger
    pub async fn init(
        application: &str,
        level: &str,
        host: &str,
        port: u16,
    ) -> tokio::io::Result<Logger> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(addr).await?;
        Ok(Logger {
            application: application.to_string(),
            level: level.to_string(),
            stream,
        })
    }
    // Establish a connection to the Vector server
    pub fn time_now() -> String {
        let now = time::Instant::now();
        let now = now.elapsed().as_secs();
        return now.to_string();
    }

    async fn send(&mut self, message: &Message) -> tokio::io::Result<()> {
        let json = serde_json::to_string(&message).unwrap();
        self.stream.write_all(json.as_bytes()).await?;
        Ok(())
    }

    pub async fn info(&mut self, message: &str) {
        if self.level.to_string().to_uppercase() == "ERROR"
            || self.level.to_string().to_uppercase() == "WARN"
        {
            return;
        }
        let message = Message::new(Self::time_now(), &self.application, "INFO", message);
        self.send(&message).await.unwrap();
    }

    pub async fn infof(&mut self, fmt_str: Arguments<'_>) {
        if self.level.to_string().to_uppercase() == "ERROR"
            || self.level.to_string().to_uppercase() == "WARN"
        {
            return;
        }
        self.info(&fmt_str.to_string()).await;
    }

    pub async fn error(&mut self, message: &str) {
        let message = Message::new(Self::time_now(), &self.application, "ERROR", message);
        self.send(&message).await.unwrap();
    }

    pub async fn errorf(&mut self, fmt_str: Arguments<'_>) {
        self.error(&fmt_str.to_string()).await;
    }

    pub async fn warn(&mut self, message: &str) {
        if self.level.to_string().to_uppercase() == "ERROR" {
            return;
        }
        let message = Message::new(Self::time_now(), &self.application, "WARN", message);
        self.send(&message).await.unwrap();
    }

    pub async fn warnf(&mut self, fmt_str: Arguments<'_>) {
        if self.level.to_string().to_uppercase() == "ERROR" {
            return;
        }
        self.warn(&fmt_str.to_string()).await;
    }

    pub async fn debug(&mut self, message: &str) {
        if self.level.to_string().to_uppercase() != "DEBUG" {
            return;
        }
        let message = Message::new(Self::time_now(), &self.application, "DEBUG", message);
        self.send(&message).await.unwrap();
    }

    pub async fn debugf(&mut self, fmt_str: Arguments<'_>) {
        if self.level.to_string().to_uppercase() != "DEBUG" {
            return;
        }
        self.debug(&fmt_str.to_string()).await;
    }

    pub async fn trace(&mut self, message: &str) {
        let message = Message::new(Self::time_now(), &self.application, "TRACE", message);

        self.send(&message).await.unwrap();
    }

    pub async fn tracef(&mut self, fmt_str: Arguments<'_>) {
        self.trace(&fmt_str.to_string()).await;
    }
}

// Tests module
#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::net::SocketAddr;
    use std::sync::Mutex;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;
    use tokio::sync::{mpsc, oneshot};

    static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    async fn start_mock_server(
        port: u16,
    ) -> (SocketAddr, mpsc::Receiver<String>, oneshot::Sender<()>) {
        let (tx, rx) = mpsc::channel(100);
        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        if let Ok((mut socket, _)) = accept_result {
                            let mut buf = vec![0; 1024];
                            match socket.read(&mut buf).await {
                                Ok(n) => {
                                    if n == 0 { continue; }
                                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                    tx.send(msg).await.unwrap();
                                }
                                Err(e) => {
                                    eprintln!("Error reading from socket: {}", e);
                                }
                            }
                        }
                    },
                    _ = &mut stop_rx => {
                        println!("Stopping server");
                        break;
                    },
                }
            }
        });
        (addr, rx, stop_tx)
    }

    #[tokio::test]
    async fn test_message_creation() {
        let msg = Message::new(
            "12345".to_string(),
            "TestApp",
            "INFO",
            "This is a test message",
        );
        assert_eq!(msg.application, "TestApp");
        assert_eq!(msg.level, "INFO");
        assert_eq!(msg.message, "This is a test message");
    }

    #[tokio::test]
    async fn test_logger_initialization() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, _, stop_server) = start_mock_server(12345).await;
        let level = "INFO";

        let logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await;
        assert!(logger.is_ok());
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_send() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "INFO";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        logger
            .send(&Message::new(
                "12345".to_string(),
                "TestApp",
                "INFO",
                "This is a test message",
            ))
            .await
            .unwrap();

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"12345\",\"application\":\"TestApp\",\"level\":\"INFO\",\"message\":\"This is a test message\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_info() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "INFO";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        logger.info("This is a test message").await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"INFO\",\"message\":\"This is a test message\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_infof() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "INFO";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        let test_arg = "dear tester";
        logger
            .infof(format_args!("This is a test message {}", test_arg))
            .await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"INFO\",\"message\":\"This is a test message dear tester\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_error() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "ERROR";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        logger.error("This is a test message").await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"ERROR\",\"message\":\"This is a test message\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_errorf() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "ERROR";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        let test_arg = "dear tester";
        logger
            .errorf(format_args!("This is a test message {}", test_arg))
            .await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"ERROR\",\"message\":\"This is a test message dear tester\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_warn() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "WARN";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        logger.warn("This is a test message").await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"WARN\",\"message\":\"This is a test message\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_warnf() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "WARN";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        let test_arg = "dear tester";
        logger
            .warnf(format_args!("This is a test message {}", test_arg))
            .await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"WARN\",\"message\":\"This is a test message dear tester\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_debug() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "DEBUG";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        logger.debug("This is a test message").await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"DEBUG\",\"message\":\"This is a test message\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_debugf() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "DEBUG";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        let test_arg = "dear tester";
        logger
            .debugf(format_args!("This is a test message {}", test_arg))
            .await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"DEBUG\",\"message\":\"This is a test message dear tester\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_trace() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "TRACE";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        logger.trace("This is a test message").await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"TRACE\",\"message\":\"This is a test message\"}");
        stop_server.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_logger_tracef() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let (local_addr, mut receiver, stop_server) = start_mock_server(12345).await;
        let level = "TRACE";

        let mut logger = Logger::init(
            "TestApp",
            &level,
            &local_addr.ip().to_string(),
            local_addr.port(),
        )
        .await
        .unwrap();
        let test_arg = "dear tester";
        logger
            .tracef(format_args!("This is a test message {}", test_arg))
            .await;

        let rx = receiver.recv().await.unwrap();

        assert_eq!(rx, "{\"timestamp\":\"0\",\"application\":\"TestApp\",\"level\":\"TRACE\",\"message\":\"This is a test message dear tester\"}");
        stop_server.send(()).unwrap();
    }
}
