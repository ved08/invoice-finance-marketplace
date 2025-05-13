use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceErrors {
    #[msg("Bid is not higher than current bid")]
    LowBidError,
    #[msg("Bidder cannot be the same as current bidder")]
    SameBidderError,
    #[msg("Bidders dont match")]
    BiddersMatchError,
    #[msg("Auction has ended")]
    AuctionEnded,
    #[msg("Auction has not ended")]
    AuctionNotEnded,
    #[msg("Keys not equal")]
    KeysDontMatch,
    #[msg("Invalid winner. Cannot perform this transaction")]
    NotWinnerError,
    #[msg("cannot settle payment yet")]
    NotFundedError,
    #[msg("Not enough funds in vault to repay")]
    NotEnoughFundsError,
}
