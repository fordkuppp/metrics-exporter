use std::sync::{Arc};
use opentelemetry::metrics::{ObservableGauge};
use opentelemetry::{global, metrics::{Counter, Histogram}, KeyValue};
use crate::trackers::steam::SharedState;

pub struct SteamInstruments {
    pub game_time_total: Counter<u64>,
    pub session_count_total: Counter<u64>,
    pub session_duration_seconds: Histogram<f64>,
    pub summary_latency: Histogram<f64>,
    pub summary_errors_total: Counter<u64>,
    // Held to keep the observable gauge callback alive.
    _session_active: ObservableGauge<u64>,
}

impl SteamInstruments {
    pub fn new(state: SharedState) -> SteamInstruments {
        let meter = global::meter("steam_meter");
        let state_clone = Arc::clone(&state);

        let session_active = meter
            .u64_observable_gauge("steam_session_active")
            .with_description("1 if the user is playing this game")
            .with_callback(move |observer| {
                if let Ok(current_sessions) = state_clone.lock() {
                    for (steam_id, session) in current_sessions.iter() {
                        observer.observe(1, &[
                            KeyValue::new("steam_id", steam_id.clone()),
                            KeyValue::new("game_id", session.game_id.clone()),
                            KeyValue::new("game_name", session.game_name.clone()),
                        ]);
                    }
                }
            })
            .build();

        Self {
            game_time_total: meter
                .u64_counter("steam_game_time_total")
                .with_description("The total time in seconds spent playing a game.")
                .build(),
            session_count_total: meter
                .u64_counter("steam_session_count_total")
                .with_description("The total number of gaming sessions started.")
                .build(),
            session_duration_seconds: meter
                .f64_histogram("steam_session_duration_seconds")
                .with_description("The duration of completed gaming sessions in seconds.")
                .build(),
            summary_latency: meter
                .f64_histogram("steam_summary_latency")
                .with_description(
                    "The duration of requests to the steam summary handler in milliseconds.",
                )
                .build(),
            summary_errors_total: meter
                .u64_counter("steam_summary_errors_total")
                .with_description(
                    "The total number of failed requests to the steam summary handler.",
                )
                .build(),
            _session_active: session_active,
        }
    }
}
