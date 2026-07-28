use crate::types::{GovernanceConfig, Proposal, Vote};
use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Config,
    ProposalCounter,
    Proposal(u64),
    Votes(u64),
    ProposalIds,
    Delegation(Address),
    UserReputation(Address),
}

pub fn set_config(env: &Env, config: &GovernanceConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<GovernanceConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn next_proposal_id(env: &Env) -> u64 {
    let counter: u64 = env.storage().instance().get(&DataKey::ProposalCounter).unwrap_or(0);
    let id = counter + 1;
    env.storage().instance().set(&DataKey::ProposalCounter, &id);
    id
}

pub fn set_proposal(env: &Env, id: u64, proposal: &Proposal) {
    env.storage().instance().set(&DataKey::Proposal(id), proposal);
    let mut ids: Vec<u64> = env.storage().instance().get(&DataKey::ProposalIds).unwrap_or_else(|| Vec::new(env));
    ids.push_back(id);
    env.storage().instance().set(&DataKey::ProposalIds, &ids);
}

pub fn get_proposal(env: &Env, id: u64) -> Option<Proposal> {
    env.storage().instance().get(&DataKey::Proposal(id))
}

pub fn get_proposal_ids(env: &Env) -> Vec<u64> {
    env.storage().instance().get(&DataKey::ProposalIds).unwrap_or_else(|| Vec::new(env))
}

pub fn set_votes(env: &Env, id: u64, votes: &Vec<Vote>) {
    env.storage().persistent().set(&DataKey::Votes(id), votes);
}

pub fn get_votes(env: &Env, id: u64) -> Vec<Vote> {
    env.storage().persistent().get(&DataKey::Votes(id)).unwrap_or_else(|| Vec::new(env))
}

pub fn set_delegate(env: &Env, delegator: &Address, delegate: &Address) {
    env.storage().instance().set(&DataKey::Delegation(delegator.clone()), delegate);
}

pub fn get_delegate(env: &Env, address: &Address) -> Option<Address> {
    env.storage().instance().get(&DataKey::Delegation(address.clone()))
}

pub fn set_user_reputation(env: &Env, user: &Address, resolved_calls: u32, total_staked: i128) {
    env.storage().instance().set(&DataKey::UserReputation(user.clone()), &(resolved_calls, total_staked));
}

pub fn get_user_reputation(env: &Env, user: &Address) -> (u32, i128) {
    env.storage().instance().get(&DataKey::UserReputation(user.clone())).unwrap_or((0, 0))
}
