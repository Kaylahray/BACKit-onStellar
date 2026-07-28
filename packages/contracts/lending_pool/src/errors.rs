use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum LendingPoolError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    BelowMinDeposit = 4,
    PoolFull = 5,
    InsufficientShares = 6,
    ZeroSupply = 7,
    InsufficientLiquidity = 8,
    Overflow = 9,
    PoolTooSmall = 10,
    EmptyInput = 11,
    AllocationNotFound = 12,
    AlreadyHarvested = 13,
    MarketNotResolved = 14,
    MarketCallFailed = 15,
    Unauthorized = 16,
    InvalidBps = 17,
    InvalidConfig = 18,
}
