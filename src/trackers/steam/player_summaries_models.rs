use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum CommunityVisibilityState {
    Private = 1,
    Public = 3,
    #[default]
    Unknown,
}


impl TryFrom<u8> for CommunityVisibilityState {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(CommunityVisibilityState::Private),
            3 => Ok(CommunityVisibilityState::Public),
            _ => Ok(CommunityVisibilityState::Unknown),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum PlayerState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6,
    #[default]
    Unknown,
}


impl TryFrom<u8> for PlayerState {
    type Error = (); // Should never have error
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PlayerState::Offline),
            1 => Ok(PlayerState::Online),
            2 => Ok(PlayerState::Busy),
            3 => Ok(PlayerState::Away),
            4 => Ok(PlayerState::Snooze),
            5 => Ok(PlayerState::LookingToTrade),
            6 => Ok(PlayerState::LookingToPlay),
            _ => Ok(PlayerState::Unknown),
        }
    }
}

fn deserialize_personastate<'de, D>(deserializer: D) -> Result<PlayerState, D::Error>
where
    D: Deserializer<'de>,
{
    let state_value: u8 = u8::deserialize(deserializer)?;
    Ok(PlayerState::try_from(state_value).unwrap())
}

fn deserialize_communityvisibilitystate<'de, D>(
    deserializer: D,
) -> Result<CommunityVisibilityState, D::Error>
where
    D: Deserializer<'de>,
{
    let state_value: u8 = u8::deserialize(deserializer)?;
    Ok(CommunityVisibilityState::try_from(state_value).unwrap())
}

fn deserialize_one_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Option<u8> = Option::deserialize(deserializer)?;
    Ok(val == Some(1))
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlayerInfo {
    // Public Data
    #[serde(alias = "steamid")]
    pub(crate) steam_id: String,
    #[serde(alias = "personaname")]
    pub(crate) persona_name: Option<String>,
    #[serde(alias = "profileurl")]
    pub(crate) profile_url: String,
    #[serde(alias = "avatar")]
    pub(crate) avatar: Option<String>,
    #[serde(alias = "avatarmedium")]
    pub(crate) avatar_medium: Option<String>,
    #[serde(alias = "avatarfull")]
    pub(crate) avatar_full: Option<String>,
    #[serde(alias = "personastate")]
    #[serde(deserialize_with = "deserialize_personastate")]
    #[serde(default = "PlayerState::default")]
    pub(crate) persona_state: PlayerState,
    #[serde(alias = "communityvisibility_state")]
    #[serde(deserialize_with = "deserialize_communityvisibilitystate")]
    #[serde(default = "CommunityVisibilityState::default")]
    pub(crate) community_visibility_state: CommunityVisibilityState,
    #[serde(alias = "profilestate")]
    #[serde(deserialize_with = "deserialize_one_to_bool")]
    pub(crate) profile_state: bool,
    #[serde(alias = "lastlogoff")]
    pub(crate) last_logoff: Option<u64>,
    #[serde(alias = "commentpermission")]
    #[serde(deserialize_with = "deserialize_one_to_bool")]
    pub(crate) comment_permission: bool,

    // Private Data
    #[serde(alias = "realname")]
    pub(crate) real_name: Option<String>,
    #[serde(alias = "primaryclanid")]
    pub(crate) primary_clan_id: Option<String>,
    #[serde(alias = "timecreated")]
    pub(crate) time_created: Option<u64>,
    #[serde(alias = "gameid")]
    pub(crate) game_id: Option<String>,
    #[serde(alias = "gameserverip")]
    pub(crate) game_server_ip: Option<String>,
    #[serde(alias = "gameextrainfo")]
    pub(crate) game_extra_info: Option<String>,
    #[serde(alias = "loccountrycode")]
    pub(crate) loc_country_code: Option<String>,
    #[serde(alias = "locstatecode")]
    pub(crate) loc_state_code: Option<String>,
    #[serde(alias = "loccityid")]
    pub(crate) loc_city_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlayersArray {
    pub(crate) players: Vec<PlayerInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PlayerSummariesResponse {
    pub(crate) response: PlayersArray,
}
