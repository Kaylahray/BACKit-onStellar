use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ScoringWeights {
    pub volume_weight_bps: u32,
    pub uniqueness_weight_bps: u32,
    pub accuracy_weight_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum TournamentStatus {
    Active,
    Finalized,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Tournament {
    pub id: u64,
    pub name: soroban_sdk::String,
    pub start_ts: u64,
    pub end_ts: u64,
    pub prize_pool: i128,
    pub scoring_weights: ScoringWeights,
    pub top_n: u32,
    pub status: TournamentStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MarketEntry {
    pub call_id: u64,
    pub total_stake: i128,
    pub unique_stakers: u32,
    pub resolved_correct: u32,
    pub resolved_total: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TournamentStanding {
    pub participant: Address,
    pub score: i128,
    pub rank: u32,
    pub prize: i128,
}
