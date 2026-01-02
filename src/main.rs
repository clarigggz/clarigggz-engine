// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

mod camera;
mod display;
mod protocol;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    info!("==========================================");
    info!("   CLARIGGGZ ENGINE V2 - POWERING ON      ");
    info!("==========================================");

    #[cfg(feature = "k1")]
    info!("CPU Target: SpacemiT K1 (RISC-V + RVV 1.0)");

    #[cfg(feature = "custom-silicon")]
    info!("CPU Target: Clarigggz Custom Silicon");

    // 1. Initialize Display Subsystem (DRM/KMS)
    info!("Initializing display subsystem...");
    let display = Arc::new(Mutex::new(display::DisplayManager::new().await?));

    // 2. Initialize Camera Subsystem (V4L2)
    info!("Initializing camera subsystem...");
    let _camera = camera::CameraManager::new().await?;

    // 3. Start glass:// protocol handler
    info!("Starting glass:// protocol observer...");
    let protocol_handler = protocol::ProtocolHandler::new(display.clone());

    tokio::spawn(async move {
        if let Err(e) = protocol_handler.run().await {
            error!("Protocol handler error: {}", e);
        }
    });

    info!("Engine initialized. Running as system init.");

    // Simple watchdog / zombie reaping loop if running as PID 1
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown request received. Synchronizing hardware...");
                break;
            }
        }
    }

    info!("Engine stopped. Releasing hardware.");
    Ok(())
}
