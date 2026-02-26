use anchor_lang::prelude::*;

use crate::{Trade, TradeState};
use crate::error::TradeError;

use anchor_spl::{associated_token::AssociatedToken, token_interface::{
    CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, close_account, transfer_checked
}
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(mut)]
    pub buyer: SystemAccount<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_usd: InterfaceAccount<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = mint_usd,
        associated_token::authority = seller,
        associated_token::token_program = token_program,
    )]
    pub seller_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        has_one = mint_usd,
        has_one = buyer,
        has_one = seller,
        seeds = [b"trade", buyer.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
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

impl<'info> Sell<'info> {

    pub fn submit_documents(
        &mut self,
        document_hash: [u8; 32],
    ) -> Result<()> {

        let trade = &mut self.escrow;

        // State must be FundsLocked
        require!(
            trade.current_state == TradeState::FundsLocked,
            TradeError::InvalidState
        );

        // Documents must not already be submitted
        require!(
            trade.document_hash.is_none(),
            TradeError::UnexpectedStateTransition
        );

        // Store document commitment
        trade.document_hash = Some(document_hash);

        // Advance state machine
        trade.current_state = TradeState::DocumentsSubmitted;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct VerifyAndSettle<'info> {
    #[account(mut)]
    pub freight_verifier: Signer<'info>,

    #[account(mut)]
    pub buyer: SystemAccount<'info>,

    #[account(mut)]
    pub seller: SystemAccount<'info>,

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
        associated_token::mint = mint_usd,
        associated_token::authority = seller,
        associated_token::token_program = token_program,
    )]
    pub seller_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = buyer, // only closes at settlement
        has_one = buyer,
        has_one = seller,
        has_one = freight_verifier,
        has_one = mint_usd,
        seeds = [b"trade", buyer.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
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

impl <'info> VerifyAndSettle<'info> {
    pub fn reject_documents(&mut self) -> Result<()> {
        require!(
            self.escrow.current_state == TradeState::DocumentsSubmitted,
            TradeError::UnreadyState
        );

        // Vault must match escrow amount
        require!(
            self.vault.amount == self.escrow.amount,
            TradeError::VaultMismatch
        );
    
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"trade",
            self.buyer.key.as_ref(),
            &self.escrow.seed.to_le_bytes(),
            &[self.escrow.bump],
        ]];

        // Refund Buyer
        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.buyer_ata.to_account_info(),
            mint: self.mint_usd.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
    
        let transfer_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );
    
        transfer_checked(
            transfer_ctx,
            self.vault.amount,
            self.mint_usd.decimals,
        )?;

        // Close Vault
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.buyer.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
    
        let close_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            signer_seeds,
        );
    
        close_account(close_ctx)?;
    
        // State Transition
        self.escrow.current_state = TradeState::Failed;
    
        // Escrow auto-closes to buyer via:
        // close = buyer (in account constraints)
    
        Ok(())
    }

    pub fn confirm_shipment_arrival(&mut self) -> Result<()> {
        require!(
            self.escrow.current_state == TradeState::DocumentsSubmitted,
            TradeError::InvalidState
        );

        require!(
            self.escrow.document_hash.is_some(),
            TradeError::MissingDocuments
        );

        self.escrow.current_state = TradeState::ShipmentConfirmed;   
        Ok(())
    }

    pub fn settle_trade(&mut self) -> Result<()> {
        require!(
            self.escrow.current_state == TradeState::ShipmentConfirmed,
            TradeError::InvalidState
        );

        require!(
            self.vault.amount == self.escrow.amount,
            TradeError::VaultMismatch
        );
    
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"trade",
            self.buyer.key.as_ref(),
            &self.escrow.seed.to_le_bytes(),
            &[self.escrow.bump],
        ]];
    
        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.seller_ata.to_account_info(),
            mint: self.mint_usd.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
    
        let transfer_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );
    
        transfer_checked(
            transfer_ctx,
            self.vault.amount,
            self.mint_usd.decimals,
        )?;
    
        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.buyer.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
    
        let close_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            signer_seeds,
        );
    
        close_account(close_ctx)?;
    
        self.escrow.current_state = TradeState::Settled;
    
        // Escrow auto closes to buyer (close = buyer)
    
        Ok(())
    }
}