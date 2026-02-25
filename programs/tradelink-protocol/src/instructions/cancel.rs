use anchor_lang::prelude::*;

use crate::Trade;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{
        CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, close_account, transfer_checked
    }
};

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_usd: InterfaceAccount<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint_usd,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        close = buyer,
        has_one = mint_usd,
        has_one = buyer,
        seeds = [b"trade", buyer.key().as_ref(), &escrow.seed.to_le_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Trade>,

    #[account(
        mut,
        associated_token::mint = mint_usd,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl <'info> Cancel<'info> {
    pub fn refund_and_close(&mut self) -> Result<()> {

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"trade",
            self.buyer.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]]; 

        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.buyer.to_account_info(),
            mint: self.mint_usd.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            transfer_accounts, 
            &signer_seeds,
        );

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_usd.decimals)?;

        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.buyer.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let close_cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            close_accounts, 
            &signer_seeds
        );

        close_account(close_cpi_ctx)?;

        Ok(())
    }
}