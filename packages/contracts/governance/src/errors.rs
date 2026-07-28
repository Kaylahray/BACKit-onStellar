use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum GovernanceError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ProposalNotFound = 4,
    AlreadyVoted = 5,
    ProposalInactive = 6,
    VotingPeriodEnded = 7,
    InsufficientReputation = 8,
    AlreadyExecuted = 9,
    ProposalNotPassed = 10,
    QuorumNotMet = 11,
    AlreadyDelegated = 12,
    SelfDelegation = 13,
}
