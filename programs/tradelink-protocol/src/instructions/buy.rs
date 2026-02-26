use anchor_lang::prelude::*;

use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{Trade, TradeState};

use crate::error::TradeError;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Buy <'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub seller: SystemAccount<'info>,

    #[account(mut)]
    pub freight_verifier: SystemAccount<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_usd: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_usd,
        associated_token::authority = buyer,
        associated_token::token_program = token_program
    )]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = buyer,
        seeds = [b"trade", buyer.key().as_ref(), seed.to_le_bytes().as_ref()],
        space = Trade::DISCRIMINATOR.len() + Trade::INIT_SPACE,
        bump
    )]
    pub escrow: Account<'info, Trade>,

    #[account(
        init,
        payer = buyer,
        associated_token::mint = mint_usd,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl <'info> Buy<'info> {
    pub fn initialize_trade(&mut self, seed: u64, amount: u64, bumps: &BuyBumps) -> Result<()> {
        
        require!(amount > 0, TradeError::InvalidAmount);

        self.escrow.set_inner(Trade { 
            buyer: self.buyer.key(), 
            seller: self.seller.key(), 
            freight_verifier: self.freight_verifier.key(),
            mint_usd: self.mint_usd.key(), 
            amount, 
            document_hash: None, 
            current_state: TradeState::Initialized,
            seed, 
            bump: bumps.escrow,
        });
        Ok(())
    }

    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: self.buyer_ata.to_account_info(),
            mint: self.mint_usd.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.buyer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        transfer_checked(cpi_ctx, deposit, self.mint_usd.decimals)
    }
}