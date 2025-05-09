use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata as Metaplex,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    errors::MarketplaceErrors,
    state::{InvoiceListing, Status},
};
#[derive(Accounts)]
pub struct SettleInvoicePayment<'info> {
    /// CHECK: This is the contract owner i.e admin wallet
    #[account(mut)]
    pub main_wallet: UncheckedAccount<'info>,
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
    #[account(
        seeds = [b"mint", main_wallet.key().as_ref(), invoice.invoice_id.to_le_bytes().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = invoice
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub investor: Signer<'info>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = investor
    )]
    pub investor_nft_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = investor,
        associated_token::mint = mint,
        associated_token::authority = main_wallet
    )]
    pub program_nft_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
impl<'info> SettleInvoicePayment<'info> {
    pub fn settle_invoice_payment(&mut self, bumps: &SettleInvoicePaymentBumps) -> Result<()> {
        require!(
            self.invoice.status == Status::Funded,
            MarketplaceErrors::NotFundedError
        );
        let accounts = TransferChecked {
            from: self.investor_nft_account.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.program_nft_account.to_account_info(),
            authority: self.investor.to_account_info(),
        };
        let ctx = CpiContext::new(self.token_program.to_account_info(), accounts);
        transfer_checked(ctx, 1, 0)?;
        let accounts = Transfer {
            from: self.bidder_vault.to_account_info(),
            to: self.investor.to_account_info(),
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
        transfer(ctx, self.bidder_vault.lamports())?;
        self.invoice.status = Status::Repaid;
        Ok(())
    }
}
