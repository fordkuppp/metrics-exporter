use lazy_static::lazy_static;
use opentelemetry::{
    metrics::{Counter, Meter, Histogram},
    global,
};

lazy_static! {
    static ref METER: Meter = global::meter("steam_metrics");

    pub static ref STEAM_SUMMARY_LATENCY: Histogram<f64> = METER
        .f64_histogram("steam_summary_latency")
        .with_description("The duration of requests to the steam summary handler in milliseconds.")
        .build();

    // Define a counter for errors
    pub static ref STEAM_SUMMARY_ERRORS_TOTAL: Counter<u64> = METER
        .u64_counter("steam_summary_errors_total")
        .with_description("The total number of failed requests to the steam summary handler.")
        .build();

    // Define a counter for game time, with `game_id` as a key/attribute
    pub static ref STEAM_GAME_TIME_TOTAL: Counter<u64> = METER
        .u64_counter("steam_game_time_total")
        .with_description("The total time in seconds spent playing a game.")
        .build();
}