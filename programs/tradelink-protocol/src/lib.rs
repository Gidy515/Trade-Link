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
}

#[derive(Accounts)]
pub struct Initialize {}
