use crate::types::AggregatorConfig;
use soroban_sdk::{contracttype, Address, Env, Map};

#[contracttype]
pub enum DataKey {
    Config,
    UserShares(Address),
    TotalShares,
    TotalDeposited,
    TotalProfits,
}

pub fn set_config(env: &Env, config: &AggregatorConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<AggregatorConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn set_user_shares(env: &Env, user: &Address, shares: i128) {
    env.storage()
        .instance()
        .set(&DataKey::UserShares(user.clone()), &shares);
}

pub fn get_user_shares(env: &Env, user: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::UserShares(user.clone()))
        .unwrap_or(0)
}

pub fn get_total_shares(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalShares)
        .unwrap_or(0)
}

pub fn set_total_shares(env: &Env, total: i128) {
    env.storage().instance().set(&DataKey::TotalShares, &total);
}

pub fn get_total_deposited(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalDeposited)
        .unwrap_or(0)
}

pub fn set_total_deposited(env: &Env, total: i128) {
    env.storage()
        .instance()
        .set(&DataKey::TotalDeposited, &total);
}

pub fn get_total_profits(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalProfits)
        .unwrap_or(0)
}

pub fn set_total_profits(env: &Env, total: i128) {
    env.storage()
        .instance()
        .set(&DataKey::TotalProfits, &total);
}
