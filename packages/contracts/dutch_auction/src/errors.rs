use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DutchAuctionError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AuctionNotStarted = 3,
    AuctionAlreadySettled = 4,
    CallNotEligible = 5,
    OracleDeadlineNotMet = 6,
    Unauthorized = 7,
    InvalidParams = 8,
    Overflow = 9,
    InvalidPrice = 10,
    UnknownConditionType = 11,
}
