use crate::types::FactoryConfig;
use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Config,
    MarketCounter,
    Market(u64),
    MarketList,
}

pub fn set_config(env: &Env, config: &FactoryConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<FactoryConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn next_market_id(env: &Env) -> u64 {
    let counter: u64 = env
        .storage()
        .instance()
        .get(&DataKey::MarketCounter)
        .unwrap_or(0);
    let next_id = counter + 1;
    env.storage()
        .instance()
        .set(&DataKey::MarketCounter, &next_id);
    next_id
}

pub fn get_market_count(env: &Env) -> u32 {
    let counter: u64 = env
        .storage()
        .instance()
        .get(&DataKey::MarketCounter)
        .unwrap_or(0);
    counter as u32
}

pub fn set_market(env: &Env, call_id: u64, market: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::Market(call_id), market);
}

pub fn get_market(env: &Env, call_id: u64) -> Option<Address> {
    env.storage().instance().get(&DataKey::Market(call_id))
}

pub fn append_market_list(env: &Env, market: &Address) {
    let mut list: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::MarketList)
        .unwrap_or_else(|| Vec::new(env));
    list.push_back(market.clone());
    env.storage().instance().set(&DataKey::MarketList, &list);
}

pub fn get_market_list(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::MarketList)
        .unwrap_or_else(|| Vec::new(env))
}
