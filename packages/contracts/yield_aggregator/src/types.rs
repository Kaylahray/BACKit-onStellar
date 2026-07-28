use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AggregatorConfig {
    pub admin: Address,
    pub fee_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AggregatorStats {
    pub total_value_locked: i128,
    pub total_shares: i128,
    pub share_price: i128,
    pub total_profits_earned: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct UserPosition {
    pub user: Address,
    pub shares: i128,
    pub deposited_amount: i128,
    pub pending_payout: i128,
}
