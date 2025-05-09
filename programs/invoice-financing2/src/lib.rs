use anchor_lang::prelude::*;
mod instructions;
use instructions::*;
mod errors;
mod state;
declare_id!("Atz8fEeWh8t78QXvSj8uFUEYxcKsmKhU1Fy1wDkubSoK");

#[program]
pub mod invoice_financing2 {
    use super::*;

    pub fn create_invoice_listing(
        ctx: Context<CreateInvoiceListing>,
        invoice_id: u64,
        face_value: u64,
    ) -> Result<()> {
        ctx.accounts
            .create_invoice_listing(invoice_id, face_value, &ctx.bumps)?;
        Ok(())
    }
    pub fn place_bid(ctx: Context<PlaceBid>, amount: u64) -> Result<()> {
        ctx.accounts.bid(amount, &ctx.bumps)?;
        Ok(())
    }
    pub fn resolve_auction(ctx: Context<ResolveAuction>) -> Result<()> {
        ctx.accounts.resolve(&ctx.bumps)?;
        ctx.accounts.mint_nft_to_winner()?;
        Ok(())
    }
    pub fn settle_invoice_payment(ctx: Context<SettleInvoicePayment>) -> Result<()> {
        ctx.accounts.settle_invoice_payment(&ctx.bumps)?;
        Ok(())
    }
}
