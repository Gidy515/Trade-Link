use anchor_lang::prelude::*;

#[error_code]
pub enum TradeError {

    #[msg("Amount must be greater than zero.")]
    InvalidAmount,
}