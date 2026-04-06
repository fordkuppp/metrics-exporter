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

    SteamTracker::new()?.start().await;

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }
    info!("Shutting down...");

    meter_provider.shutdown()?;
    logger.shutdown()?;

    Ok(())
}
