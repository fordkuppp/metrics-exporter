use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct SteamConfig {
    pub api_key: String,
    pub polling_interval_seconds: u16,
    pub steam_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OtlpConfig {
    pub enabled: bool,
    pub collector_endpoint: String,
    pub protocol: OtlpProtocol,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OtlpProtocol {
    Tonic,
    Http,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub steam: SteamConfig,
    pub otlp_config: OtlpConfig,
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

impl Settings {
    pub fn init() -> Result<(), ConfigError> {
        let settings = Config::builder()
            .add_source(File::with_name("config/default").required(true))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("APP").separator("_"))
            .build()?
            .try_deserialize()?;

        SETTINGS.set(settings).expect("Settings already initialized");
        Ok(())
    }

    pub fn get() -> &'static Settings {
        SETTINGS.get().expect("Settings not initialized. Call Settings::init() first.")
    }
}