use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{
    errors::MarketplaceErrors,
    state::{InvoiceListing, Status},
};
#[derive(Accounts)]
pub struct ResolveAuction<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
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
        init,
        seeds = [b"mint", signer.key().as_ref(), invoice.invoice_id.to_le_bytes().as_ref()],
        bump,
        payer = signer,
        mint::decimals = 0,
        mint::authority = invoice
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    /// CHECK: This is the winner of auction. check is happening in the instruction
    pub investor: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = investor
    )]
    pub investor_nft_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: This is the metadata account from metaplex
    pub nft_metadata: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
impl<'info> ResolveAuction<'info> {
    pub fn resolve(&mut self, bumps: &ResolveAuctionBumps) -> Result<()> {
        require_keys_eq!(
            self.investor.key(),
            self.invoice.current_bidder.unwrap().key(),
            MarketplaceErrors::NotWinnerError
        );
        let accounts = Transfer {
            from: self.bidder_vault.to_account_info(),
            to: self.signer.to_account_info(),
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
        self.invoice.status = Status::Funded;

        let binding = self.invoice.invoice_id.to_le_bytes();
        let seeds = &[b"invoice", binding.as_ref(), &[self.invoice.bump]];
        let signer = [&seeds[..]];
        let nft_data = DataV2 {
            name: String::from("Invoice"),
            symbol: String::from("INV"),
            uri: String::from("https://arweave.net/sUEsfmH7DzhI8AmCnozxcTIcGYDZsPv1gupPbw4551E"),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };
        let accounts = CreateMetadataAccountsV3 {
            metadata: self.nft_metadata.to_account_info(),
            mint: self.mint.to_account_info(),
            mint_authority: self.invoice.to_account_info(),
            payer: self.signer.to_account_info(),
            update_authority: self.invoice.to_account_info(),
            system_program: self.system_program.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        let metadata_ctx = CpiContext::new_with_signer(
            self.token_metadata_program.to_account_info(),
            accounts,
            &signer,
        );
        create_metadata_accounts_v3(metadata_ctx, nft_data, false, true, None)?;

        Ok(())
    }
    pub fn mint_nft_to_winner(&mut self) -> Result<()> {
        let binding = self.invoice.invoice_id.to_le_bytes();
        let seeds = &[b"invoice", binding.as_ref(), &[self.invoice.bump]];
        let signer = [&seeds[..]];
        let accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.investor_nft_account.to_account_info(),
            authority: self.invoice.to_account_info(),
        };
        let mint_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, &signer);
        mint_to(mint_ctx, 1)?;
        Ok(())
    }
}
