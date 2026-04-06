use std::collections::HashMap;
use crate::settings::Settings;
use crate::trackers::steam::client::SteamClient;
use crate::trackers::steam::instruments::{SteamInstruments};
use crate::trackers::steam::player_summaries_models::{PlayerInfo, PlayerState};
use anyhow::Result;
use opentelemetry::KeyValue;
use std::sync::{Arc, Mutex};
use tracing::{error, info};
use crate::trackers::steam::{PlayerSession, SharedState};

pub struct SteamTracker {
    steam_client: Arc<SteamClient>,
    steam_ids: Vec<String>,
    instruments: Arc<SteamInstruments>,
    player_state: SharedState,
}

impl SteamTracker {
    pub fn new() -> Result<Self> {
        let settings = Settings::get();
        let player_state: SharedState = Arc::new(Mutex::new(HashMap::new()));
        let instruments = Arc::new(SteamInstruments::new(Arc::clone(&player_state)));
        Ok(Self {
            steam_client: Arc::new(SteamClient::new()?),
            steam_ids: settings.steam.steam_ids.clone(),
            instruments,
            player_state
        })
    }

    pub async fn start(self) {
        tokio::spawn(async move {
            let polling_interval = Settings::get().steam.polling_interval_seconds;

            let mut last_game_states: std::collections::HashMap<String, Option<String>> =
                self.steam_ids.iter().map(|id| (id.clone(), None)).collect();

            let mut interval = tokio::time::interval(std::time::Duration::from_secs(polling_interval as u64));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            info!("Starting Steam poller with {}s interval", polling_interval);

            loop {
                interval.tick().await;

                self.instruments.summary_latency.record(10f64, &[]);
                for steam_id in &self.steam_ids {
                    info!("Polling steam id: {}", steam_id);
                    match self.steam_client.fetch_player_summaries(steam_id).await {
                        Ok(response) => {
                            if let Some(player_info) = response.response.players.first() {
                                let current_game_id = &player_info.game_id;
                                let last_game_id = last_game_states.get_mut(steam_id).unwrap();

                                self.handle_metrics(
                                    player_info,
                                    last_game_id,
                                    current_game_id,
                                    polling_interval as u64,
                                );

                                *last_game_id = current_game_id.clone();
                            }
                        }
                        Err(e) => {
                            error!("Error polling Steam API for {}: {}", steam_id, e);
                            self.instruments.summary_errors_total.add(1, &[]);
                        }
                    }
                }
            }
        });
    }

    fn handle_metrics(
        &self,
        player_info: &PlayerInfo,
        last_game_id: &Option<String>,
        current_game_id: &Option<String>,
        interval_secs: u64,
    ) {
        match (last_game_id, current_game_id) {
            // Case: Still playing the same game
            (Some(last), Some(current)) if last == current => {
                let attributes = [
                    KeyValue::new("game_id", current.clone()),
                    KeyValue::new("steam_id", player_info.steam_id.clone()),
                    KeyValue::new(
                        "game_name",
                        player_info
                            .game_extra_info
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string()),
                    ),
                ];

                self.instruments
                    .game_time_total
                    .add(interval_secs, &attributes);
            }

            // Case: Started a new game
            (None, Some(current)) => {
                info!("User {} started playing {}", player_info.steam_id, current);
                let game_name = player_info
                    .game_extra_info
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string());
                let mut state = self.player_state.lock().expect("lock poisoned");
                state.insert(
                    player_info.steam_id.clone(),
                    PlayerSession {
                        game_id: current.clone(),
                        game_name,
                    },
                );
            }

            // Case: Stopped playing
            (Some(last), None) => {
                info!("User {} stopped playing {}", player_info.steam_id, last);
                let mut state = self.player_state.lock().expect("lock poisoned");
                state.remove(&player_info.steam_id);            }

            _ => {} // No change (offline -> offline)
        }
    }
}
