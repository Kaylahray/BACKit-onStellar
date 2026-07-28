use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum OracleMarketplaceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    OracleAlreadyRegistered = 4,
    OracleNotFound = 5,
    OracleNotActive = 6,
    InsufficientStake = 7,
    CooldownActive = 8,
    InvalidFee = 9,
    CallNotFound = 10,
    OracleNotSelectedForCall = 11,
    AlreadyRated = 12,
    InvalidRating = 13,
}
