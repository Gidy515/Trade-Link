use anchor_lang::prelude::*;

declare_id!("S4Zy9tboDLQ8Qj8UxhcSFc9z4K4GrYzUyDVenbKyr3Z");

pub mod instructions;
pub use instructions::*;

pub mod state;
pub use state::*;

pub mod error;

#[program]
pub mod tradelink_protocol {
    use super::*;

    pub fn buy(ctx: Context<Buy>, seed: u64, deposit: u64, amount: u64) -> Result<()> {
        ctx.accounts.initialize_trade(seed, amount, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)?;
        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        ctx.accounts.refund_and_close()?;
        Ok(())
    }

    pub fn sell(ctx: Context<Sell>, document_hash: [u8; 32]) -> Result<()> {
        ctx.accounts.submit_documents(document_hash)?;
        Ok(())
    }

    pub fn settlement(ctx: Context<VerifyAndSettle>) -> Result<()> {
        ctx.accounts.reject_documents()?;
        ctx.accounts.confirm_shipment_arrival()?;
        ctx.accounts.settle_trade()?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
