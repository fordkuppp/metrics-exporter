use std::sync::Arc;
use opentelemetry::KeyValue;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::trackers::steam::client::SteamClient;
use anyhow::{Result};
use crate::settings::Settings;
use log::{info, error};

pub struct SteamTracker {
    job_scheduler: JobScheduler,
    steam_client: Arc<SteamClient>,
    steam_ids: Vec<String>
}

impl SteamTracker {
    pub async fn new() -> Result<Self> {
        let steam_ids = Settings::get().steam.steam_ids.clone();
        let job_scheduler = JobScheduler::new().await?;
        let steam_client = Arc::new(SteamClient::new()?);
        let scheduler = Self {
            job_scheduler,
            steam_client,
            steam_ids,
        };

        scheduler.add_player_summaries_job().await?;

        Ok(scheduler)
    }

    pub async fn add_player_summaries_job(&self) -> Result<()> {
        async fn polling_logic(polling_client: Arc<SteamClient>, last_game_id_clone: Arc<Mutex<Option<String>>>, steam_ids: Vec<String>) {
            let steam_ids = steam_ids.clone();
            let steam_id = steam_ids.first().unwrap();
            let result = polling_client.fetch_player_summaries(&steam_id).await;
            match result {
                Ok(response) => {
                    if let Some(player_info) = response.response.players.first() {
                        let mut last_game_id = last_game_id_clone.lock().await;
                        if let Some(game_id) = &player_info.game_id {
                            if let Some(ref last_id) = *last_game_id {
                                if last_id == game_id {
                                    let attributes = [
                                        KeyValue::new("game_id", game_id.clone()),
                                        KeyValue::new("steam_id", player_info.steam_id.clone())
                                    ];
                                    super::instruments::STEAM_GAME_TIME_TOTAL.add(300, &attributes);
                                    info!("User is still playing game {}, incremented counter by 300s", game_id);
                                } else {
                                    info!("User switched from game {} to {}. Resetting timer.", last_id, game_id);
                                }
                            } else {
                                info!("User started playing game {}. Starting timer.", game_id);
                            }
                            *last_game_id = Some((*game_id.clone()).to_owned());
                        } else {
                            if last_game_id.is_some() {
                                info!("User stopped playing game. Resetting timer.");
                                *last_game_id = None;
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Error polling Steam API: {}", e);
                }
            }

        }
        let polling_interval = format!("1/{} * * * * *", Settings::get().steam.polling_interval_seconds);

        let steam_ids = self.steam_ids.clone();

        let polling_client = self.steam_client.clone();
        let last_game_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        self.job_scheduler.add(Job::new_async(polling_interval, move |_uuid, _l| {
            let polling_client = polling_client.clone();
            let last_game_id_clone = last_game_id.clone();
            let steam_ids = steam_ids.clone();
            Box::pin(async move {
                let steam_ids_clone = steam_ids.clone();
                polling_logic(polling_client, last_game_id_clone, steam_ids_clone).await;
            })
        })?).await?;
        self.job_scheduler.start().await?;
        Ok(())
    }
}