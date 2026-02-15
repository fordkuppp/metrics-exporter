use reqwest;
use url::Url;
use anyhow::Result;
use log::{debug};
use crate::settings::Settings;
use super::player_summaries_models::PlayerSummariesResponse;

#[derive(Clone)]
pub struct SteamClient {
    api_key: String,
    api_url: Url,
}

impl SteamClient {
    pub fn new() -> Result<Self> {
        let api_key = Settings::get().steam.api_key.clone();
        let api_url = Url::parse("http://api.steampowered.com/")?;
        Ok(Self { api_key, api_url })
    }

    const PLAYER_SUMMARIES_ENDPOINT: &'static str = "ISteamUser/GetPlayerSummaries/v0002/";

    pub async fn fetch_player_summaries(
        &self,
        steam_id: &str,
    ) -> Result<PlayerSummariesResponse> {
        let mut endpoint = self.api_url.join(Self::PLAYER_SUMMARIES_ENDPOINT)?;
        endpoint
            .query_pairs_mut()
            .append_pair("key", self.api_key.as_str())
            .append_pair("steamids", steam_id);
        let response = reqwest::get(endpoint)
            .await?
            .json::<PlayerSummariesResponse>()
            .await?;
        debug!("{:?}", response);
        Ok(response)
    }
}
