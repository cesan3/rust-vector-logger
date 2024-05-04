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
}

impl Logger {
    // Initialize a new Logger
    pub async fn init(application: &str, host: &str, port: u64) -> tokio::io::Result<Logger> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(addr).await?;
        Ok(Logger {
            application: application.to_string(),
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
        self.stream.write_all(b"\n").await?;
        Ok(())
    }

    pub async fn info(&mut self, message: &str) {
        let message = Message::new(Self::time_now(), &self.application, "INFO", message);

        self.send(&message).await.unwrap();
    }

    pub async fn infof(&mut self, fmt_str: Arguments<'_>) {
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
        let message = Message::new(Self::time_now(), &self.application, "WARN", message);

        self.send(&message).await.unwrap();
    }

    pub async fn warnf(&mut self, fmt_str: Arguments<'_>) {
        self.warn(&fmt_str.to_string()).await;
    }

    pub async fn debug(&mut self, message: &str) {
        let message = Message::new(Self::time_now(), &self.application, "DEBUG", message);

        self.send(&message).await.unwrap();
    }

    pub async fn debugf(&mut self, fmt_str: Arguments<'_>) {
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
