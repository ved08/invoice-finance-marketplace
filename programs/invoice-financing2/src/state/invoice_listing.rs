use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, InitSpace)]
pub enum Status {
    Open,
    Funded,
    Repaid,
    Defaulted,
}

#[account]
#[derive(InitSpace)]
pub struct InvoiceListing {
    pub invoice_id: u64,
    pub face_value: u64,
    pub status: Status,
    pub current_bidder: Option<Pubkey>,
    pub current_bid: u64,
    pub bump: u8,
}
