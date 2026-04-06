use crate::settings::Settings;
use crate::trackers::steam::tracker::SteamTracker;
use anyhow::Result;
use tracing::{error, info};

mod otlp;
mod settings;
mod trackers;

#[tokio::main]
async fn main() -> Result<()> {
    Settings::init()?;

    let logger = otlp::logger::init_logger();
    let meter_provider = otlp::metrics::init_metrics();

    let tracker_handle = SteamTracker::new()?.start();

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received...");
        }
        result = tracker_handle => {
            match result {
                Ok(()) => error!("Steam tracker exited unexpectedly"),
                Err(e) => error!("Steam tracker panicked: {}", e),
            }
        }
    }
    info!("Shutting down...");

    meter_provider.shutdown()?;
    logger.shutdown()?;

    Ok(())
}
