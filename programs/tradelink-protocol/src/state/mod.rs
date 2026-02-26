use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum TradeState {
    Initialized,
    FundsLocked,
    DocumentsSubmitted,
    Cancelled,
    Failed,
    ShipmentConfirmed,
    Settled,
}

#[account]
#[derive(InitSpace)]
pub struct Trade {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub freight_verifier: Pubkey,
    pub mint_usd: Pubkey,
    pub amount: u64,
    pub document_hash: Option<[u8; 32]>,
    pub current_state: TradeState,
    pub seed: u64,
    pub bump: u8,
}
