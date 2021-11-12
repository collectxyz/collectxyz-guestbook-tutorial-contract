// This file defines a specialized MockQuerier for use in testing that allows us to mock
// the xyz NFT contract's behavior. To use this MockQuerier, use mock_dependencies_from_xyz
// instead of cosmwasm_std's mock_dependencies function, setting the desired balance of
// xyz tokens based on what you're trying to test.
//
// See contract_tests.rs for example usage.

use std::collections::HashMap;

use collectxyz::nft::{QueryMsg as XyzQueryMsg, XyzTokenInfo};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult,
    QueryRequest, StdError, SystemError, SystemResult, WasmQuery,
};
use terra_cosmwasm::TerraQueryWrapper;

pub fn mock_dependencies_xyz(
    xyz_balances: HashMap<String, XyzTokenInfo>,
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, XyzMockQuerier> {
    let xyz_querier = XyzMockQuerier::new(
        MockQuerier::<TerraQueryWrapper>::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]),
        xyz_balances,
    );
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: xyz_querier,
    }
}

pub struct XyzMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    xyz_balances: HashMap<String, XyzTokenInfo>,
}

impl XyzMockQuerier {
    pub fn new(
        base: MockQuerier<TerraQueryWrapper>,
        xyz_balances: HashMap<String, XyzTokenInfo>,
    ) -> Self {
        XyzMockQuerier { base, xyz_balances }
    }
}

impl<'a> XyzMockQuerier {
    pub fn update_xyz_balances(&'a mut self, xyz_balances: HashMap<String, XyzTokenInfo>) {
        self.xyz_balances = xyz_balances;
    }

    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if contract_addr.starts_with("xyz-nft-contract") {
                    if let XyzQueryMsg::XyzNftInfo { token_id } =
                        from_binary::<XyzQueryMsg>(&msg).unwrap()
                    {
                        return self
                            .xyz_balances
                            .get(&token_id)
                            .map(|xyz| SystemResult::Ok(ContractResult::from(to_binary(xyz))))
                            .unwrap_or(SystemResult::Ok(ContractResult::from(Err(
                                StdError::not_found("xyz"),
                            ))));
                    } else {
                        panic!("unsupported message type! {}", msg)
                    }
                }
                panic!("unsupported query");
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl Querier for XyzMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}
