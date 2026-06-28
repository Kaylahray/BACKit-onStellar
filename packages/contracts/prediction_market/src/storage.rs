use crate::types::{Call, MarketConfig};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Config,
    Call,
    UserStake(Address, u32),
    Locked,
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

pub fn get_user_stake(
    env: &Env,
    staker: &soroban_sdk::Address,
    position: u32,
) -> i128 {
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
