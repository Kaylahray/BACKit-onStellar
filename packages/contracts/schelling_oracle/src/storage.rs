use crate::types::{Dispute, DisputeConfig, VoteCommitment};
use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Config,
    DisputeCounter,
    Dispute(u64),
    /// Ordered list of every address that committed a vote on a dispute —
    /// walked by `resolve_dispute` to tally revealed votes and forfeit
    /// unrevealed ones. Bounded by `MAX_VOTERS_PER_DISPUTE` in `lib.rs`.
    Voters(u64),
    Commitment(u64, Address),
}

pub fn set_config(env: &Env, config: &DisputeConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<DisputeConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn next_dispute_id(env: &Env) -> u64 {
    let counter: u64 = env
        .storage()
        .instance()
        .get(&DataKey::DisputeCounter)
        .unwrap_or(0);
    let next_id = counter + 1;
    env.storage()
        .instance()
        .set(&DataKey::DisputeCounter, &next_id);
    next_id
}

pub fn set_dispute(env: &Env, dispute: &Dispute) {
    env.storage()
        .instance()
        .set(&DataKey::Dispute(dispute.id), dispute);
}

pub fn get_dispute(env: &Env, dispute_id: u64) -> Option<Dispute> {
    env.storage().instance().get(&DataKey::Dispute(dispute_id))
}

pub fn get_voters(env: &Env, dispute_id: u64) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::Voters(dispute_id))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn append_voter(env: &Env, dispute_id: u64, voter: &Address) {
    let mut voters = get_voters(env, dispute_id);
    voters.push_back(voter.clone());
    env.storage()
        .instance()
        .set(&DataKey::Voters(dispute_id), &voters);
}

pub fn set_commitment(env: &Env, dispute_id: u64, voter: &Address, commitment: &VoteCommitment) {
    env.storage()
        .instance()
        .set(&DataKey::Commitment(dispute_id, voter.clone()), commitment);
}

pub fn get_commitment(env: &Env, dispute_id: u64, voter: &Address) -> Option<VoteCommitment> {
    env.storage()
        .instance()
        .get(&DataKey::Commitment(dispute_id, voter.clone()))
}
