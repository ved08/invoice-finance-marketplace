use crate::state::{InvoiceListing, Status};
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
#[derive(Accounts)]
#[instruction(invoice_id: u64)]
pub struct CreateInvoiceListing<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + InvoiceListing::INIT_SPACE,
        seeds = [b"invoice", invoice_id.to_le_bytes().as_ref()],
        bump
    )]
    pub invoice: Account<'info, InvoiceListing>,
    #[account(
        mut,
        seeds = [b"invoiceVault", invoice.key().as_ref()],
        bump
    )]
    pub invoice_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> CreateInvoiceListing<'info> {
    pub fn create_invoice_listing(
        &mut self,
        invoice_id: u64,
        face_value: u64,
        bumps: &CreateInvoiceListingBumps,
    ) -> Result<()> {
        self.invoice.set_inner(InvoiceListing {
            invoice_id,
            face_value,
            status: Status::Open,
            current_bidder: None,
            current_bid: 0,
            bump: bumps.invoice,
        });
        let rent = Rent::get()?;
        let rent_amount = rent.minimum_balance(self.invoice_vault.data_len());
        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.invoice_vault.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, rent_amount)?;
        Ok(())
    }
}
