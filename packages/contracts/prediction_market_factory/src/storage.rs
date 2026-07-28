use crate::types::{FactoryConfig, Swarm};
use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
pub enum DataKey {
    Config,
    MarketCounter,
    Market(u64),
    MarketList,
    Swarm(u64),
    SwarmCounter,
    SwarmMarkets(u64),
    CallSwarm(u64),
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

pub fn set_swarm(env: &Env, id: u64, swarm: &Swarm) {
    env.storage().instance().set(&DataKey::Swarm(id), swarm);
}

pub fn get_swarm(env: &Env, id: u64) -> Option<Swarm> {
    env.storage().instance().get(&DataKey::Swarm(id))
}

pub fn next_swarm_id(env: &Env) -> u64 {
    let counter: u64 = env.storage().instance().get(&DataKey::SwarmCounter).unwrap_or(0);
    let id = counter + 1;
    env.storage().instance().set(&DataKey::SwarmCounter, &id);
    id
}

pub fn set_swarm_market(env: &Env, swarm_id: u64, _position: u32, call_id: u64) {
    let mut markets: Vec<u64> = env.storage().instance().get(&DataKey::SwarmMarkets(swarm_id)).unwrap_or_else(|| Vec::new(env));
    markets.push_back(call_id);
    env.storage().instance().set(&DataKey::SwarmMarkets(swarm_id), &markets);
}

pub fn get_swarm_markets(env: &Env, swarm_id: u64) -> Vec<u64> {
    env.storage().instance().get(&DataKey::SwarmMarkets(swarm_id)).unwrap_or_else(|| Vec::new(env))
}

pub fn set_call_swarm(env: &Env, call_id: u64, swarm_id: u64, position: u32) {
    env.storage().instance().set(&DataKey::CallSwarm(call_id), &(swarm_id, position));
}

pub fn get_call_swarm(env: &Env, call_id: u64) -> Option<(u64, u32)> {
    env.storage().instance().get(&DataKey::CallSwarm(call_id))
}
