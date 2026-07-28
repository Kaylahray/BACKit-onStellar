use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct OracleProvider {
    pub pubkey: BytesN<32>,
    pub address: Address,
    pub fee_bps: u32,
    pub min_stake: i128,
    pub staked_amount: i128,
    pub total_resolved: u64,
    pub total_disputes: u64,
    pub is_active: bool,
    pub registered_at: u64,
    pub deregister_after: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct OracleRating {
    pub oracle: BytesN<32>,
    pub user: Address,
    pub satisfied: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MarketplaceConfig {
    pub admin: Address,
    pub cooldown_secs: u64,
    pub default_fee_bps: u32,
}
