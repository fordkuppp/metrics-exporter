use std::collections::HashMap;
use crate::settings::Settings;
use crate::trackers::steam::client::SteamClient;
use crate::trackers::steam::instruments::SteamInstruments;
use crate::trackers::steam::player_summaries_models::PlayerInfo;
use anyhow::Result;
use opentelemetry::KeyValue;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{error, info};
use crate::trackers::steam::{PlayerSession, SharedState};

pub enum GameEvent {
    Started { game_id: String, game_name: String },
    Stopped { game_id: String },
    Switched { old_game_id: String, new_game_id: String, new_game_name: String },
    Playing { game_id: String, game_name: String },
    Idle,
}

pub fn detect_game_event(
    last_game_id: &Option<String>,
    current_game_id: &Option<String>,
    current_game_name: &Option<String>,
) -> GameEvent {
    let game_name = || current_game_name.clone().unwrap_or_else(|| "Unknown".to_string());

    match (last_game_id, current_game_id) {
        (Some(last), Some(current)) if last == current => GameEvent::Playing {
            game_id: current.clone(),
            game_name: game_name(),
        },
        (Some(last), Some(current)) => GameEvent::Switched {
            old_game_id: last.clone(),
            new_game_id: current.clone(),
            new_game_name: game_name(),
        },
        (None, Some(current)) => GameEvent::Started {
            game_id: current.clone(),
            game_name: game_name(),
        },
        (Some(last), None) => GameEvent::Stopped {
            game_id: last.clone(),
        },
        (None, None) => GameEvent::Idle,
    }
}

pub struct SteamTracker {
    steam_client: Arc<SteamClient>,
    steam_ids: Vec<String>,
    instruments: Arc<SteamInstruments>,
    player_state: SharedState,
    db: PgPool,
}

impl SteamTracker {
    pub fn new(db: PgPool) -> Result<Self> {
        let settings = Settings::get();
        let player_state = SharedState::default();
        let instruments = Arc::new(SteamInstruments::new(Arc::clone(&player_state)));
        Ok(Self {
            steam_client: Arc::new(SteamClient::new()?),
            steam_ids: settings.steam.steam_ids.clone(),
            instruments,
            player_state,
            db,
        })
    }

    pub fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            let polling_interval = Settings::get().steam.polling_interval_seconds;

            let mut last_game_states: HashMap<String, Option<String>> =
                self.steam_ids.iter().map(|id| (id.clone(), None)).collect();

            let mut interval = tokio::time::interval(std::time::Duration::from_secs(polling_interval as u64));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            info!("Starting Steam poller with {}s interval", polling_interval);

            loop {
                interval.tick().await;

                for steam_id in &self.steam_ids {
                    info!("Polling steam id: {}", steam_id);
                    let start = std::time::Instant::now();
                    let result = self.steam_client.fetch_player_summaries(steam_id).await;
                    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                    self.instruments.summary_latency.record(latency_ms, &[]);

                    match result {
                        Ok(response) => {
                            if let Some(player_info) = response.response.players.first() {
                                if let Err(e) = self.save_player_summary(player_info).await {
                                    error!("Failed to save snapshot: {}", e);
                                }

                                let last_game_id = last_game_states.get_mut(steam_id).unwrap();
                                let event = detect_game_event(
                                    last_game_id,
                                    &player_info.game_id,
                                    &player_info.game_extra_info,
                                );

                                self.handle_event(steam_id, &event, polling_interval as u64).await;

                                *last_game_id = player_info.game_id.clone();
                            }
                        }
                        Err(e) => {
                            error!("Error polling Steam API for {}: {}", steam_id, e);
                            self.instruments.summary_errors_total.add(1, &[]);
                        }
                    }
                }
            }
        })
    }

    async fn handle_event(&self, steam_id: &str, event: &GameEvent, interval_secs: u64) {
        match event {
            GameEvent::Playing { game_id, game_name } => {
                self.instruments.game_time_total.add(interval_secs, &[
                    KeyValue::new("game_id", game_id.clone()),
                    KeyValue::new("steam_id", steam_id.to_string()),
                    KeyValue::new("game_name", game_name.clone()),
                ]);
            }
            GameEvent::Started { game_id, game_name } => {
                info!("User {} started playing {}", steam_id, game_name);
                self.instruments.session_count_total.add(1, &[
                    KeyValue::new("steam_id", steam_id.to_string()),
                    KeyValue::new("game_id", game_id.clone()),
                    KeyValue::new("game_name", game_name.clone()),
                ]);
                if let Err(e) = self.start_session(steam_id, game_id, game_name).await {
                    error!("Failed to start session in DB: {}", e);
                }
                let mut state = self.player_state.lock().expect("lock poisoned");
                state.insert(steam_id.to_string(), PlayerSession {
                    game_id: game_id.clone(),
                    game_name: game_name.clone(),
                    started_at: std::time::Instant::now(),
                });
            }
            GameEvent::Stopped { game_id } => {
                info!("User {} stopped playing {}", steam_id, game_id);
                if let Err(e) = self.end_session(steam_id).await {
                    error!("Failed to end session in DB: {}", e);
                }
                let mut state = self.player_state.lock().expect("lock poisoned");
                if let Some(session) = state.remove(steam_id) {
                    let duration = session.started_at.elapsed().as_secs_f64();
                    self.instruments.session_duration_seconds.record(duration, &[
                        KeyValue::new("steam_id", steam_id.to_string()),
                        KeyValue::new("game_id", session.game_id),
                        KeyValue::new("game_name", session.game_name),
                    ]);
                }
            }
            GameEvent::Switched { old_game_id, new_game_id, new_game_name } => {
                info!("User {} switched from {} to {}", steam_id, old_game_id, new_game_name);
                if let Err(e) = self.end_session(steam_id).await {
                    error!("Failed to end session in DB: {}", e);
                }
                if let Err(e) = self.start_session(steam_id, new_game_id, new_game_name).await {
                    error!("Failed to start session in DB: {}", e);
                }
                let mut state = self.player_state.lock().expect("lock poisoned");
                // End the old session
                if let Some(old_session) = state.remove(steam_id) {
                    let duration = old_session.started_at.elapsed().as_secs_f64();
                    self.instruments.session_duration_seconds.record(duration, &[
                        KeyValue::new("steam_id", steam_id.to_string()),
                        KeyValue::new("game_id", old_session.game_id),
                        KeyValue::new("game_name", old_session.game_name),
                    ]);
                }
                // Start the new session
                self.instruments.session_count_total.add(1, &[
                    KeyValue::new("steam_id", steam_id.to_string()),
                    KeyValue::new("game_id", new_game_id.clone()),
                    KeyValue::new("game_name", new_game_name.clone()),
                ]);
                state.insert(steam_id.to_string(), PlayerSession {
                    game_id: new_game_id.clone(),
                    game_name: new_game_name.clone(),
                    started_at: std::time::Instant::now(),
                });
            }
            GameEvent::Idle => {}
        }
    }

    async fn save_player_summary(&self, player: &PlayerInfo) -> Result<()> {
        sqlx::query(
            "INSERT INTO steam_player_summaries (
                steam_id, persona_name, profile_url, avatar, avatar_medium, avatar_full,
                persona_state, community_visibility_state, profile_state, last_logoff,
                comment_permission, real_name, primary_clan_id, time_created,
                game_id, game_server_ip, game_extra_info,
                loc_country_code, loc_state_code, loc_city_id
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)",
        )
        .bind(&player.steam_id)
        .bind(&player.persona_name)
        .bind(&player.profile_url)
        .bind(&player.avatar)
        .bind(&player.avatar_medium)
        .bind(&player.avatar_full)
        .bind(player.persona_state.as_i16())
        .bind(player.community_visibility_state.as_i16())
        .bind(player.profile_state)
        .bind(player.last_logoff.map(|v| v as i64))
        .bind(player.comment_permission)
        .bind(&player.real_name)
        .bind(&player.primary_clan_id)
        .bind(player.time_created.map(|v| v as i64))
        .bind(&player.game_id)
        .bind(&player.game_server_ip)
        .bind(&player.game_extra_info)
        .bind(&player.loc_country_code)
        .bind(&player.loc_state_code)
        .bind(player.loc_city_id.map(|v| v as i32))
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn start_session(&self, steam_id: &str, game_id: &str, game_name: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO steam_sessions (steam_id, game_id, game_name) VALUES ($1, $2, $3)",
        )
        .bind(steam_id)
        .bind(game_id)
        .bind(game_name)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn end_session(&self, steam_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE steam_sessions SET ended_at = NOW() WHERE steam_id = $1 AND ended_at IS NULL",
        )
        .bind(steam_id)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
