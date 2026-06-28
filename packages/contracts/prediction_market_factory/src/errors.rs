use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum FactoryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidStakeAmount = 4,
    InvalidEndTime = 5,
    InvalidOutcomeCount = 6,
    TokenNotWhitelisted = 7,
    ContractPaused = 8,
    MarketWasmNotSet = 9,
    MarketNotFound = 10,
}
