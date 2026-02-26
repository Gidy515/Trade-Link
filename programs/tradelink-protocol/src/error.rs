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


    #[msg("Cannot submit docs on no or empty trade")]
    InvalidState,

    #[msg("Documents already submitted")]
    UnexpectedStateTransition,

    #[msg("Documents have not been submitted")]
    UnreadyState,

    #[msg("Vault balance does not match escrow amount")]
    VaultMismatch,

    #[msg("No documents submitted")]
    MissingDocuments,

}