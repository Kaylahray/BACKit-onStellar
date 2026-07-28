use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ReputationError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    BadgeAlreadyAwarded = 4,
    TransferNotAllowed = 5,
    InvalidBadgeType = 6,
}
