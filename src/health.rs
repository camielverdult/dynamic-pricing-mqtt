use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time;

const HEALTH_PORT: u16 = 8080;

struct HealthState {
    price: Option<f32>,
    last_updated: Option<String>,
}

async fn serve_health(health_state: Arc<Mutex<HealthState>>, error_state: Arc<AtomicBool>) {
    let addr = format!("0.0.0.0:{}", HEALTH_PORT);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind health endpoint");
    println!("Health endpoint listening on {}", addr);

    loop {
        let (mut stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Health endpoint accept error: {}", e);
                continue;
            }
        };

        // Read the incoming request to be a good HTTP citizen
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf).await;

        let state = health_state.lock().unwrap();
        let has_error = error_state.load(Ordering::Relaxed);

        let (status_code, status_text, body) = match (&state.price, &state.last_updated) {
            (Some(price), Some(updated)) if !has_error => (
                200,
                "OK",
                format!(
                    r#"{{"status": "ok", "price": {}, "last_updated": "{}"}}"#,
                    price, updated
                ),
            ),
            (Some(price), Some(updated)) => (
                503,
                "Service Unavailable",
                format!(
                    r#"{{"status": "error", "price": {}, "last_updated": "{}"}}"#,
                    price, updated
                ),
            ),
            _ => (
                503,
                "Service Unavailable",
                r#"{"status": "starting", "price": null, "last_updated": null}"#.to_string(),
            ),
        };

        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status_code,
            status_text,
            body.len(),
            body
        );

        let _ = stream.write_all(response.as_bytes()).await;
    }
}
