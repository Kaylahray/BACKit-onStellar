use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum OracleRelayError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    UnauthorizedRelayer = 4,
    InvalidBlockHeader = 5,
    MerkleProofVerificationFailed = 6,
    PriceOutOfRange = 7,
    InsufficientSignatures = 8,
    BlockAlreadyRelayed = 9,
}
