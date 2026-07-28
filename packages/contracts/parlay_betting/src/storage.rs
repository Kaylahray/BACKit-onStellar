use crate::types::{LegAggregate, Parlay, ParlayConfig};
use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Config,
    ParlayCounter,
    Parlay(u64),
    UserParlays(Address),
    LegAggregate(u64),
}

pub fn set_config(env: &Env, config: &ParlayConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<ParlayConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn next_parlay_id(env: &Env) -> u64 {
    let counter: u64 = env
        .storage()
        .instance()
        .get(&DataKey::ParlayCounter)
        .unwrap_or(0);
    let next_id = counter + 1;
    env.storage()
        .instance()
        .set(&DataKey::ParlayCounter, &next_id);
    next_id
}

pub fn set_parlay(env: &Env, parlay: &Parlay) {
    env.storage()
        .instance()
        .set(&DataKey::Parlay(parlay.id), parlay);
}

pub fn get_parlay(env: &Env, parlay_id: u64) -> Option<Parlay> {
    env.storage().instance().get(&DataKey::Parlay(parlay_id))
}

pub fn append_user_parlay(env: &Env, user: &Address, parlay_id: u64) {
    let mut list = get_user_parlays(env, user);
    list.push_back(parlay_id);
    env.storage()
        .instance()
        .set(&DataKey::UserParlays(user.clone()), &list);
}

pub fn get_user_parlays(env: &Env, user: &Address) -> Vec<u64> {
    env.storage()
        .instance()
        .get(&DataKey::UserParlays(user.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn get_leg_aggregate(env: &Env, call_id: u64) -> LegAggregate {
    env.storage()
        .instance()
        .get(&DataKey::LegAggregate(call_id))
        .unwrap_or(LegAggregate {
            total_staked: 0,
            claimed_payout: None,
        })
}

pub fn set_leg_aggregate(env: &Env, call_id: u64, aggregate: &LegAggregate) {
    env.storage()
        .instance()
        .set(&DataKey::LegAggregate(call_id), aggregate);
}
