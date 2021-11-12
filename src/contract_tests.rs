use std::collections::HashMap;

use collectxyz::nft::{Coordinates, XyzExtension, XyzTokenInfo};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, BankMsg, Coin, DepsMut, Response, StdError, Timestamp};

use crate::contract;
use crate::error::ContractError;
use crate::mock_querier::mock_dependencies_xyz;
use crate::msg::{EntriesResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Entry};

const OWNER: &str = "owner";
const ADDR1: &str = "addr1";
const ADDR2: &str = "addr2";

fn get_initial_xyz_balances() -> HashMap<String, XyzTokenInfo> {
    HashMap::from([
        (
            "xyz #1".to_string(),
            XyzTokenInfo {
                owner: Addr::unchecked(ADDR1),
                approvals: vec![],
                name: "xyz #1".to_string(),
                description: "".to_string(),
                image: None,
                extension: XyzExtension {
                    coordinates: Coordinates { x: 1, y: 1, z: 1 },
                    arrival: Timestamp::from_nanos(0),
                    prev_coordinates: None,
                },
            },
        ),
        (
            "xyz #2".to_string(),
            XyzTokenInfo {
                owner: Addr::unchecked(ADDR2),
                approvals: vec![],
                name: "xyz #2".to_string(),
                description: "".to_string(),
                image: None,
                extension: XyzExtension {
                    coordinates: Coordinates { x: 2, y: 2, z: 2 },
                    arrival: Timestamp::from_seconds(10000),
                    prev_coordinates: None,
                },
            },
        ),
    ])
}

fn get_initial_config() -> Config {
    Config {
        character_limit: 240,
        entry_fee: Coin::new(1000, "uluna"),
        xyz_nft_contract: Addr::unchecked("xyz-nft-contract"),
    }
}

fn setup_contract(deps: DepsMut) {
    contract::instantiate(
        deps,
        mock_env(),
        mock_info(OWNER, &[]),
        InstantiateMsg {
            config: get_initial_config(),
        },
    )
    .unwrap();
}

#[test]
fn create_entry() {
    let xyz_balances = get_initial_xyz_balances();
    let mut deps = mock_dependencies_xyz(xyz_balances.clone(), &[]);
    setup_contract(deps.as_mut());

    let entry_text = "0xja was here.";

    // can't create an entry at a non-existent xyz
    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[Coin::new(1000, "uluna")]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: "xyz #123456".to_string(),
            text: entry_text.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::generic_err(
            "Querier contract error: xyz not found"
        ))
    );

    // can't create an entry for an xyz you don't own
    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[Coin::new(1000, "uluna")]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: "xyz #2".to_string(),
            text: entry_text.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // can't create an entry if you provide insufficient fees
    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: "xyz #1".to_string(),
            text: entry_text.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // can't create an entry if you provide insufficient fees
    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: "xyz #1".to_string(),
            text: entry_text.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // can't create an entry if your xyz hasn't arrived
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(0);
    let err = contract::execute(
        deps.as_mut(),
        env,
        mock_info(ADDR2, &[Coin::new(1000, "uluna")]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: "xyz #2".to_string(),
            text: entry_text.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::RelocationInProgress {});

    // can create an entry if all conditions are satisfied
    let res = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[Coin::new(1000, "uluna")]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: "xyz #1".to_string(),
            text: entry_text.to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::default()
            .add_attribute("action", "entry_created")
            .add_attribute("xyz_id", "xyz #1".to_string())
            .add_attribute("xyz_coordinates_x", "1")
            .add_attribute("xyz_coordinates_y", "1")
            .add_attribute("xyz_coordinates_z", "1")
    );

    // check that the entry was created
    let entry_coords = xyz_balances.get("xyz #1").unwrap().extension.coordinates;
    let entry = from_binary::<Entry>(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Entry {
                author_xyz_id: "xyz #1".to_string(),
                coordinates: entry_coords.clone(),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        entry,
        Entry {
            author_xyz_id: "xyz #1".to_string(),
            text: entry_text.to_string(),
            coordinates: entry_coords.clone()
        }
    );

    // can't create another entry at the same location
    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[Coin::new(1000, "uluna")]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: "xyz #1".to_string(),
            text: entry_text.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::EntryAlreadyExists {});

    // can create an entry with same xyz at a different location
    let mut new_xyz_balances = xyz_balances.clone();
    let moved_xyz = new_xyz_balances.get_mut(&"xyz #1".to_string()).unwrap();
    moved_xyz.extension.coordinates = Coordinates { x: 3, y: 3, z: 3 };
    deps.querier.update_xyz_balances(new_xyz_balances);
    let res = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[Coin::new(1000, "uluna")]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: "xyz #1".to_string(),
            text: entry_text.to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::default()
            .add_attribute("action", "entry_created")
            .add_attribute("xyz_id", "xyz #1".to_string())
            .add_attribute("xyz_coordinates_x", "3")
            .add_attribute("xyz_coordinates_y", "3")
            .add_attribute("xyz_coordinates_z", "3")
    );
}

#[test]
fn withdraw() {
    let contract_balance = vec![Coin::new(10000, "uluna")];
    let mut deps = mock_dependencies(&contract_balance);
    setup_contract(deps.as_mut());

    // non-owner can't withdraw
    let err = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[]),
        ExecuteMsg::Withdraw {
            amount: vec![Coin::new(100, "uluna")],
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // owner can withdraw
    let res = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(OWNER, &[]),
        ExecuteMsg::Withdraw {
            amount: vec![Coin::new(100, "uluna")],
        },
    )
    .unwrap();
    assert_eq!(
        res.messages[0].msg,
        BankMsg::Send {
            amount: vec![Coin::new(100, "uluna")],
            to_address: mock_info(OWNER, &[]).sender.to_string()
        }
        .into()
    )
}

#[test]
fn read_entries() {
    let xyz_balances = get_initial_xyz_balances();
    let mut deps = mock_dependencies_xyz(xyz_balances.clone(), &[]);
    setup_contract(deps.as_mut());

    let addr1_entry = Entry {
        author_xyz_id: "xyz #1".to_string(),
        text: "xyz #1 was here".to_string(),
        coordinates: Coordinates { x: 1, y: 1, z: 1 },
    };

    let addr2_entry = Entry {
        author_xyz_id: "xyz #2".to_string(),
        text: "xyz #2 was here".to_string(),
        coordinates: Coordinates { x: 2, y: 2, z: 2 },
    };

    // write entries for both xyz owners
    let _ = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR1, &[Coin::new(1000, "uluna")]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: addr1_entry.author_xyz_id.clone(),
            text: addr1_entry.text.clone(),
        },
    )
    .unwrap();
    let _ = contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADDR2, &[Coin::new(1000, "uluna")]),
        ExecuteMsg::CreateEntry {
            author_xyz_id: addr2_entry.author_xyz_id.clone(),
            text: addr2_entry.text.clone(),
        },
    )
    .unwrap();

    // read all entries for xyz #1
    let res = from_binary::<EntriesResponse>(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::EntriesForXyz {
                author_xyz_id: addr1_entry.author_xyz_id.clone(),
                limit: None,
                start_after: None,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        EntriesResponse {
            entries: vec![addr1_entry.clone()]
        }
    );

    // read all entries for xyz #2
    let res = from_binary::<EntriesResponse>(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::EntriesForXyz {
                author_xyz_id: addr2_entry.author_xyz_id.clone(),
                limit: None,
                start_after: None,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        EntriesResponse {
            entries: vec![addr2_entry.clone()]
        }
    );

    // read all entries for [1,1,1]
    let res = from_binary::<EntriesResponse>(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::EntriesForCoordinates {
                coordinates: addr1_entry.coordinates.clone(),
                limit: None,
                start_after: None,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        EntriesResponse {
            entries: vec![addr1_entry.clone()]
        }
    );

    // read all entries for [2,2,2]
    let res = from_binary::<EntriesResponse>(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::EntriesForCoordinates {
                coordinates: addr2_entry.coordinates.clone(),
                limit: None,
                start_after: None,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        EntriesResponse {
            entries: vec![addr2_entry.clone()]
        }
    );
}
