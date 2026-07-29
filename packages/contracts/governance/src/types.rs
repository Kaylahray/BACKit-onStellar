use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct GovernanceConfig {
    pub admin: Address,
    pub proposal_threshold_calls: u32,
    pub threshold_accuracy_bps: u32,
    pub voting_period_ledgers: u32,
    pub quorum_bps: u32,
    pub pass_threshold_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub target_contract: Address,
    pub function_name: Symbol,
    pub for_votes: i128,
    pub against_votes: i128,
    pub created_ledger: u32,
    pub executed: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    pub voter: Address,
    pub support: bool,
    pub power: i128,
}
