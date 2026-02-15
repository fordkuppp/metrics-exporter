use crate::trackers::steam::tracker::SteamTracker;
use anyhow::Result;
use log::{info, error};
use crate::settings::Settings;

mod trackers;
mod settings;
mod otlp;

#[tokio::main]
async fn main() -> Result<()> {
    Settings::init()?;

    let meter_provider = otlp::metrics::init_metrics();

    SteamTracker::new().await?;

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received...");
        },
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        },
    }
    info!("Shutting down...");
    meter_provider.shutdown()?;
    Ok(())
}
