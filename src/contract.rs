use collectxyz::nft::{Coordinates, QueryMsg as XyzQueryMsg, XyzTokenInfo};
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{EntriesResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{entries, entries_key, Config, Entry, CONFIG, OWNER};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:entries-tutorial-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &msg.config)?;
    OWNER.save(deps.storage, &info.sender.to_string())?;

    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateEntry {
            author_xyz_id,
            text,
        } => execute_create_entry(deps, env, info, author_xyz_id, text),
        ExecuteMsg::Withdraw { amount } => execute_withdraw(deps, env, info, amount),
    }
}

pub fn execute_create_entry(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    author_xyz_id: String,
    text: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check that text doesn't exceed the character limit
    if text.len() > config.character_limit as usize {
        return Err(ContractError::CharacterLimitExceeded {});
    }

    // Check that the sender provided funds sufficient to cover the entry fee
    if !info.funds.iter().any(|coin| {
        // the denomination matches and the amount is sufficient
        coin.denom == config.entry_fee.denom && coin.amount.u128() >= config.entry_fee.amount.u128()
    }) {
        return Err(ContractError::InsufficientFunds {});
    }

    // Look up the author xyz
    let xyz: XyzTokenInfo = deps.querier.query_wasm_smart(
        config.xyz_nft_contract,
        &XyzQueryMsg::XyzNftInfo {
            token_id: author_xyz_id.clone(),
        },
    )?;

    // Check that the sender owns this xyz
    if xyz.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Check that the xyz isn't currently relocating
    if !xyz.extension.has_arrived(env.block.time) {
        return Err(ContractError::RelocationInProgress {});
    }

    // Construct the new entry
    let new_entry = Entry {
        author_xyz_id: author_xyz_id.clone(),
        coordinates: xyz.extension.coordinates,
        text,
    };
    let new_entry_key = entries_key(author_xyz_id.clone(), xyz.extension.coordinates);

    // Save the entry if one doesn't already exist for this xyz at this location
    entries().update(deps.storage, &new_entry_key, |old_entry| match old_entry {
        Some(_) => Err(ContractError::EntryAlreadyExists {}),
        None => Ok(new_entry),
    })?;

    Ok(Response::default()
        .add_attribute("action", "entry_created")
        .add_attribute("xyz_id", author_xyz_id)
        .add_attribute("xyz_coordinates_x", xyz.extension.coordinates.x.to_string())
        .add_attribute("xyz_coordinates_y", xyz.extension.coordinates.y.to_string())
        .add_attribute("xyz_coordinates_z", xyz.extension.coordinates.z.to_string()))
}

pub fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Vec<Coin>,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::default().add_message(BankMsg::Send {
        amount,
        to_address: owner,
    }))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Entry {
            author_xyz_id,
            coordinates,
        } => to_binary(&query_entry(deps, env, author_xyz_id, coordinates)?),
        QueryMsg::EntriesForXyz {
            author_xyz_id,
            start_after,
            limit,
        } => to_binary(&query_entries_for_xyz(
            deps,
            env,
            author_xyz_id,
            start_after,
            limit,
        )?),
        QueryMsg::EntriesForCoordinates {
            coordinates,
            start_after,
            limit,
        } => to_binary(&query_entries_for_coordinates(
            deps,
            env,
            coordinates,
            start_after,
            limit,
        )?),
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
    }
}

pub fn query_entry(
    deps: Deps,
    _env: Env,
    author_xyz_id: String,
    coordinates: Coordinates,
) -> StdResult<Entry> {
    let entry = entries().load(deps.storage, &entries_key(author_xyz_id, coordinates))?;
    Ok(entry)
}

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub fn query_entries_for_xyz(
    deps: Deps,
    _env: Env,
    author_xyz_id: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<EntriesResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let entries_for_xyz: StdResult<Vec<_>> = entries()
        .idx
        .author_xyz_id
        .prefix(author_xyz_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, entry)| entry))
        .collect();

    Ok(EntriesResponse {
        entries: entries_for_xyz?,
    })
}

pub fn query_entries_for_coordinates(
    deps: Deps,
    _env: Env,
    coordinates: Coordinates,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<EntriesResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let entries_for_coordinates: StdResult<Vec<_>> = entries()
        .idx
        .coordinates
        .prefix(coordinates.to_bytes())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, entry)| entry))
        .collect();

    Ok(EntriesResponse {
        entries: entries_for_coordinates?,
    })
}

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}
