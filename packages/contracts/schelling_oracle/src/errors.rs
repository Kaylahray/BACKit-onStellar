use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum OracleError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidPeriod = 4,
    InvalidBondBps = 5,
    InvalidOutcome = 6,
    SameOutcome = 7,
    InvalidPoolAmount = 8,
    InvalidBondAmount = 9,
    BondBelowMinimum = 10,
    DisputeNotFound = 11,
    CommitPeriodEnded = 12,
    RevealPeriodNotStarted = 13,
    RevealPeriodEnded = 14,
    RevealPeriodNotEnded = 15,
    AlreadyCommitted = 16,
    NoCommitmentFound = 17,
    AlreadyRevealed = 18,
    CommitmentMismatch = 19,
    InvalidVoteOutcome = 20,
    InvalidStakeAmount = 21,
    AlreadyResolved = 22,
    TooManyVoters = 23,
    Overflow = 24,
}
