use anchor_lang::prelude::*;

#[error_code]
pub enum TradeError {

    #[msg("Amount must be greater than zero.")]
    InvalidAmount,

    #[msg("Your funds are not locked yet, perhaps not deposited")]
    UnexpectedState,

    #[msg("Documents properly submitted, can't cancel trade now")]
    InvalidStateTransition,


    #[msg("Trade balance empty")]
    VaultBalanceMismatch,
}