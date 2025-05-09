use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    errors::MarketplaceErrors,
    state::{InvoiceListing, Status},
};

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,

    #[account(
        mut,
        seeds = [b"invoice", invoice.invoice_id.to_le_bytes().as_ref()],
        bump = invoice.bump
    )]
    pub invoice: Account<'info, InvoiceListing>,
    #[account(
        mut,
        seeds = [b"bidderVault", invoice.key().as_ref()],
        bump
    )]
    pub bidder_vault: SystemAccount<'info>,
    /// CHECK: Previous bidder account
    #[account(mut)]
    pub previous_bidder: Option<UncheckedAccount<'info>>,
    pub system_program: Program<'info, System>,
}

impl<'info> PlaceBid<'info> {
    pub fn bid(&mut self, amount: u64, bumps: &PlaceBidBumps) -> Result<()> {
        if let Some(prev_bidder) = self.invoice.current_bidder {
            require_keys_neq!(
                prev_bidder,
                self.bidder.key(),
                MarketplaceErrors::SameBidderError
            );
            require_keys_eq!(
                prev_bidder,
                self.invoice.current_bidder.unwrap().key(),
                MarketplaceErrors::BiddersMatchError
            );
        }
        // Check auction conditions
        require!(
            self.invoice.status == Status::Open,
            MarketplaceErrors::AuctionEnded
        );
        require!(
            self.invoice.current_bid < amount,
            MarketplaceErrors::LowBidError
        );

        // Transfer old bid amount back to previous bidder
        if self.invoice.current_bidder.is_some() {
            let accounts = Transfer {
                from: self.bidder_vault.to_account_info(),
                to: self.previous_bidder.clone().unwrap().to_account_info(),
            };
            let binding = [bumps.bidder_vault];
            let signer_seeds = &[&[
                b"bidderVault",
                self.invoice.to_account_info().key.as_ref(),
                &binding,
            ][..]];
            let ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds,
            );

            transfer(ctx, self.invoice.current_bid)?;
        }

        // Transfer new bid amount
        let accounts = Transfer {
            from: self.bidder.to_account_info(),
            to: self.bidder_vault.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, amount)?;

        // Update auction state
        self.invoice.current_bid = amount;
        self.invoice.current_bidder = Some(self.bidder.key());

        Ok(())
    }
}
