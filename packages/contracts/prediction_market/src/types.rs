use soroban_sdk::{contracttype, Address, Bytes, BytesN, Map};

/// Describes the price-movement condition that determines the winning outcome.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ConditionType {
    TargetAbove(i128),
    TargetBelow(i128),
    PercentUp(u32),
    PercentDown(u32),
    Range(i128, i128),
}

/// Arguments supplied when the factory deploys a new market instance.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MarketInitArgs {
    pub stake_token: Address,
    pub stake_amount: i128,
    pub start_price: i128,
    pub end_ts: u64,
    pub token_address: Address,
    pub pair_id: Bytes,
    pub metadata_hash: BytesN<32>,
    pub condition: ConditionType,
    pub outcome_count: u32,
}

/// Mirrors `call_registry::Call` so `outcome_manager` can deserialize cross-contract.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub id: u64,
    pub creator: Address,
    pub stake_token: Address,
    pub stake_amount: i128,
    pub end_ts: u64,
    pub token_address: Address,
    pub pair_id: Bytes,
    pub metadata_hash: BytesN<32>,
    pub outcome_count: u32,
    pub outcome_stakes: Map<u32, i128>,
    pub stakes: Map<u32, Map<Address, i128>>,
    pub outcome: u32,
    pub start_price: i128,
    pub end_price: i128,
    pub condition: ConditionType,
    pub settled: bool,
    pub voided: bool,
    pub created_at: u64,
    pub cancelled: bool,
    pub metadata_version: u32,
    pub share_tokens: Map<u32, Address>,
}

/// Per-market configuration set at deploy time.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MarketConfig {
    pub call_id: u64,
    pub creator: Address,
    pub outcome_manager: Address,
    pub factory: Address,
    pub min_stake: i128,
    pub max_stake_per_user: i128,
    pub staking_cutoff_secs: u64,
    pub paused: bool,
}
