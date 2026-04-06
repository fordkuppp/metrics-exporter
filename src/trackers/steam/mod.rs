use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod client;
mod instruments;
mod player_summaries_models;
pub(crate) mod tracker;

pub struct PlayerSession {
    pub game_id: String,
    pub game_name: String,
    pub started_at: std::time::Instant,
}

pub type SharedState = Arc<Mutex<HashMap<String, PlayerSession>>>;
