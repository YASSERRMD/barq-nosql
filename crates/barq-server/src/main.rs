use barq_server::BarqEngine;
use barq_server::router::router;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let data_dir = std::env::var("BARQ_DATA_DIR").unwrap_or_else(|_| "data".to_string());
    
    let engine = Arc::new(BarqEngine::new(data_dir).await?);
    
    let app = router(engine);
    
    let addr = std::env::var("BARQ_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:7070".to_string());
    
    println!("Starting BARQ-NOSQL server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
