use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not Found")]
    NotFound {},

    #[error("Entry Already Exists")]
    EntryAlreadyExists {},

    #[error("Character Limit Exceeded")]
    CharacterLimitExceeded {},

    #[error("Insufficient Funds")]
    InsufficientFunds {},

    #[error("Relocation in Progress")]
    RelocationInProgress {},
}
