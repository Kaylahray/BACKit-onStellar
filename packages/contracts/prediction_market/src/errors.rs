use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum MarketError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidStakeAmount = 3,
    InvalidEndTime = 4,
    CallNotFound = 5,
    CallEnded = 6,
    CallSettled = 7,
    InvalidPosition = 8,
    Unauthorized = 9,
    ContractPaused = 10,
    CallNotEnded = 11,
    InvalidOutcome = 12,
    InvalidOutcomeCount = 13,
    StakingCutoffActive = 15,
    InvalidCallId = 16,
    ReserveDiscrepancy = 17,
    NotEligibleForBonus = 18,
}
