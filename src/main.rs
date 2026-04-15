use crate::settings::Settings;
use crate::trackers::steam::tracker::SteamTracker;
use anyhow::Result;
use tracing::{error, info};

mod db;
mod otlp;
mod settings;
mod trackers;

#[tokio::main]
async fn main() -> Result<()> {
    Settings::init()?;

    let logger_provider = otlp::logger::init_logger();
    let meter_provider = otlp::metrics::init_metrics();

    let pool = db::init_pool(&Settings::get().database_url).await?;
    info!("Database migrations applied");

    let tracker_handle = SteamTracker::new(pool)?.start();

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
    logger_provider.shutdown()?;

    Ok(())
}
