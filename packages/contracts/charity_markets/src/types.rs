use soroban_sdk::{contracttype, Address, Map};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CharityCallInit {
    pub stake_amount: i128,
    pub outcome_count: u32,
    pub creator_outcome: u32,
    pub end_ts: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CharityCall {
    pub id: u64,
    pub creator: Address,
    pub stake_token: Address,
    pub stake_amount: i128,
    pub outcome_count: u32,
    pub creator_outcome: u32,
    pub charity_address: Address,
    pub charity_split_bps: u32,
    pub total_donated: i128,
    pub outcome_manager: Address,
    pub resolved: bool,
    pub final_outcome: u32,
    pub created_at: u64,
    pub outcome_stakes: Map<u32, i128>,
    pub user_stakes: Map<u32, Map<Address, i128>>,
}
