use soroban_sdk::{contracttype, Address, Vec};

/// One leg of a parlay: a prediction on a single already-deployed
/// [`prediction_market`] instance.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ParlayLeg {
    pub call_id: u64,
    pub market_address: Address,
    pub outcome: u32,
}

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParlayStatus {
    Active,
    Won,
    Lost,
    Voided,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Parlay {
    pub id: u64,
    pub user: Address,
    pub legs: Vec<ParlayLeg>,
    pub active_leg_index: u32,
    pub total_escrowed: i128,
    pub status: ParlayStatus,
    pub stake_token: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ParlayConfig {
    pub admin: Address,
    pub outcome_manager: Address,
}

/// Aggregate bookkeeping for a single `call_id` that one or more parlays have
/// staked on via this contract's shared address (see the module doc comment
/// in `lib.rs` for why this exists).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct LegAggregate {
    /// Sum of every parlay's stake placed on this `call_id` via this contract.
    pub total_staked: i128,
    /// Set once the aggregate payout for this `call_id` has actually been
    /// claimed from `outcome_manager`. `None` until the first parlay sharing
    /// this leg advances past it.
    pub claimed_payout: Option<i128>,
}
