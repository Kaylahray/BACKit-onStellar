use soroban_sdk::{contracttype, Address, BytesN, Map, String};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct FactoryConfig {
    pub admin: Address,
    pub outcome_manager: Address,
    pub market_wasm_hash: BytesN<32>,
    pub min_stake: i128,
    pub max_stake_per_user: i128,
    pub staking_cutoff_secs: u64,
    pub paused: bool,
    pub whitelisted_tokens: Map<Address, bool>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SwarmStage {
    pub condition: String,
    pub stake_token: Address,
    pub stake_amount: i128,
    pub duration_secs: u64,
    pub start_price: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Swarm {
    pub id: u64,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub stages: u32,
    pub created_at: u64,
    pub active: bool,
}
