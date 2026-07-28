use crate::types::{Call, MarketConfig};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Config,
    Call,
    UserStake(Address, u32),
    Locked,
    EarlyStakerCount,
    TotalEarlyStakerBonusPaid,
    UserStakeTimestamp(Address, u32),
    UserHasWithdrawn(Address, u32),
}

pub fn set_config(env: &Env, config: &MarketConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<MarketConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn set_call(env: &Env, call: &Call) {
    env.storage().instance().set(&DataKey::Call, call);
}

pub fn get_call(env: &Env) -> Option<Call> {
    env.storage().instance().get(&DataKey::Call)
}

pub fn set_user_stake(env: &Env, staker: &soroban_sdk::Address, position: u32, amount: i128) {
    env.storage()
        .instance()
        .set(&DataKey::UserStake(staker.clone(), position), &amount);
}

pub fn get_user_stake(env: &Env, staker: &soroban_sdk::Address, position: u32) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::UserStake(staker.clone(), position))
        .unwrap_or(0)
}

pub fn acquire_lock(env: &Env) {
    env.storage().instance().set(&DataKey::Locked, &true);
}

pub fn release_lock(env: &Env) {
    env.storage().instance().set(&DataKey::Locked, &false);
}

pub fn is_locked(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Locked)
        .unwrap_or(false)
}

pub fn set_user_stake_timestamp(env: &Env, staker: &Address, position: u32, ts: u64) {
    env.storage()
        .instance()
        .set(&DataKey::UserStakeTimestamp(staker.clone(), position), &ts);
}

pub fn get_user_stake_timestamp(env: &Env, staker: &Address, position: u32) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::UserStakeTimestamp(staker.clone(), position))
        .unwrap_or(0)
}

pub fn set_user_has_withdrawn(env: &Env, staker: &Address, position: u32, withdrawn: bool) {
    env.storage()
        .instance()
        .set(&DataKey::UserHasWithdrawn(staker.clone(), position), &withdrawn);
}

pub fn get_user_has_withdrawn(env: &Env, staker: &Address, position: u32) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::UserHasWithdrawn(staker.clone(), position))
        .unwrap_or(false)
}

pub fn get_early_staker_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::EarlyStakerCount)
        .unwrap_or(0)
}

pub fn set_early_staker_count(env: &Env, count: u64) {
    env.storage().instance().set(&DataKey::EarlyStakerCount, &count);
}

pub fn get_total_early_staker_bonus_paid(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalEarlyStakerBonusPaid)
        .unwrap_or(0)
}

pub fn set_total_early_staker_bonus_paid(env: &Env, total: i128) {
    env.storage()
        .instance()
        .set(&DataKey::TotalEarlyStakerBonusPaid, &total);
}
