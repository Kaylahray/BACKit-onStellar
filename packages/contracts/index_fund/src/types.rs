use soroban_sdk::{contracttype, Address};

/// A single market held by the index fund.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct IndexConstituent {
    pub call_id: u64,
    pub weight_bps: u32,
    pub stake_amount: i128,
    pub outcome: u32,
    pub market_address: Address,
}

/// Performance snapshot of the index fund.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct IndexPerformance {
    pub nav: i128,
    pub total_aum: i128,
    pub total_index_supply: i128,
    pub total_markets: u32,
}

/// Lightweight read-only snapshot of a market's state.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MarketSnapshot {
    pub call_id: u64,
    pub pool_size: i128,
    pub majority_outcome: u32,
    pub is_resolved: bool,
}
