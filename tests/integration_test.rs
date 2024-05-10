// tests/integration_test.rs
//
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

async fn start_mock_server(port: u16) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel(10);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = vec![0; 1024];
            let n = socket.read(&mut buf).await.unwrap();
            let msg = String::from_utf8(buf[..n].to_vec()).unwrap();
            tx.send(msg).await.unwrap();
        }
    });
    rx
}

#[tokio::test]
async fn test_logger_sends_correct_message() {
    let mut receiver = start_mock_server(12345).await;
    let level = "INFO";
    let mut logger = rust_vector_logger::Logger::init("TestApp", &level, "127.0.0.1", 12345)
        .await
        .unwrap();
    logger.info("Test Message").await;
    // Use a timeout to prevent the test from hanging
    let timeout = tokio::time::timeout(Duration::from_secs(5), receiver.recv()).await;
    assert!(timeout.is_ok(), "Test timeoud out waiting for response");

    let message = timeout.unwrap().expect("No message received");
    assert!(
        message.contains("Test Message"),
        "Message does not contain expected message"
    );
}
