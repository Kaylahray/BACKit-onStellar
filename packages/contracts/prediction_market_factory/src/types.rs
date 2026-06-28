use soroban_sdk::{contracttype, Address, BytesN, Map};

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
