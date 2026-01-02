// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use log::{error, info};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

mod camera;
mod display;
mod protocol;

/// System initialization for PID 1 mode
fn bootstrap_system() -> Result<()> {
    info!("Performing early system bootstrap...");

    // 1. Mount essential filesystems
    let mounts = [
        ("proc", "/proc", "proc"),
        ("sysfs", "/sys", "sysfs"),
        ("devtmpfs", "/dev", "devtmpfs"),
        ("tmpfs", "/run", "tmpfs"),
    ];

    for (source, target, fstype) in mounts {
        info!("Mounting {} to {}...", source, target);
        let status = Command::new("mount")
            .arg("-t")
            .arg(fstype)
            .arg(source)
            .arg(target)
            .status();

        if let Err(e) = status {
            error!("Failed to mount {}: {}", target, e);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (standard out for kernel logs)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("==========================================");
    info!("   CLARIGGGZ ENGINE V2 - PID 1 MODE       ");
    info!("==========================================");

    // Check if we are running as init
    if std::process::id() == 1 {
        info!("Detected PID 1. Bootstrapping OS environment.");
        bootstrap_system().context("Failed to bootstrap system")?;
    }

    #[cfg(feature = "k1")]
    info!("Target: SpacemiT K1 (RISC-V + RVV 1.0)");

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

    info!("Engine ready. Entering main event loop.");

    // Signal handling for PID 1
    use tokio::signal::unix::{SignalKind, signal};
    let mut sigchld = signal(SignalKind::child())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        tokio::select! {
            _ = sigchld.recv() => {
                // Reap zombie processes if any
                while let Ok(Some(status)) = (|| -> Result<Option<std::process::ExitStatus>> {
                    // This is a simplified mock of reaping
                    Ok(None)
                })() {
                    info!("Child process exited: {:?}", status);
                }
            }
            _ = sigterm.recv() => {
                info!("SIGTERM received. Shutting down Clarigggz.");
                break;
            }
            _ = sigint.recv() => {
                info!("SIGINT received.");
                break;
            }
        }
    }

    info!("Synchronizing storage and stopping.");
    let _ = Command::new("sync").status();

    Ok(())
}
