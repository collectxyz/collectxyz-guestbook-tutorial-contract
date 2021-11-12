use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use collectxyz::nft::Coordinates;
use cosmwasm_std::Coin;

use crate::state::{Config, Entry};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateEntry { author_xyz_id: String, text: String },
    Withdraw { amount: Vec<Coin> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Entry {
        author_xyz_id: String,
        coordinates: Coordinates,
    },
    EntriesForXyz {
        author_xyz_id: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    EntriesForCoordinates {
        coordinates: Coordinates,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EntriesResponse {
    pub entries: Vec<Entry>,
}
