use crate::display::DisplayManager;
use anyhow::Result;
use axum::{Router, routing::get};
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ProtocolHandler {
    display: Arc<Mutex<DisplayManager>>,
}

impl ProtocolHandler {
    pub fn new(display: Arc<Mutex<DisplayManager>>) -> Self {
        Self { display }
    }

    pub async fn run(&self) -> Result<()> {
        info!("ProtocolHandler: Spawning glass:// bridge on port 9000");

        // The glass:// protocol is simulated over HTTP for the desktop simulator,
        // but uses DMABUF/Shared Memory on the real K1 hardware.
        let app = Router::new()
            .route("/stream", get(Self::handle_stream))
            .with_state(self.display.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
        let listener = tokio::net::TcpListener::bind(addr).await?;

        info!("glass:// bridge listening on {}", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }

    async fn handle_stream() -> &'static str {
        "glass://v1:stream_active"
    }
}
