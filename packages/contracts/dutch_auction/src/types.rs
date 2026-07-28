use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub struct AuctionInfo {
    pub call_id: u64,
    pub condition_type: u32,
    pub start_price: i128,
    pub start_ts: u64,
    pub settled: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct DutchAuctionConfig {
    pub admin: Address,
    pub outcome_manager: Address,
    pub auction_duration_secs: u64,
    pub oracle_deadline_secs: u64,
    pub settler_reward_bps: u32,
}
