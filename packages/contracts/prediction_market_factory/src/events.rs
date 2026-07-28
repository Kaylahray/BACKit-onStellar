#![allow(deprecated)]

use soroban_sdk::{Address, Env, String};

pub fn emit_factory_initialized(env: &Env, admin: &Address, outcome_manager: &Address) {
    env.events().publish(
        ("prediction_market_factory", "initialized"),
        (admin.clone(), outcome_manager.clone()),
    );
}

pub fn emit_market_deployed(
    env: &Env,
    call_id: u64,
    market_address: &Address,
    creator: &Address,
    stake_token: &Address,
    end_ts: u64,
) {
    env.events().publish(
        ("prediction_market_factory", "MarketDeployed"),
        (
            call_id,
            market_address.clone(),
            creator.clone(),
            stake_token.clone(),
            end_ts,
        ),
    );
}

pub fn emit_swarm_created(
    env: &Env,
    swarm_id: u64,
    creator: &Address,
    title: &String,
    stages: u32,
) {
    env.events().publish(
        ("prediction_market_factory", "SwarmCreated"),
        (swarm_id, creator.clone(), title.clone(), stages),
    );
}

pub fn emit_swarm_market_created(
    env: &Env,
    swarm_id: u64,
    position: u32,
    call_id: u64,
    market_address: &Address,
) {
    env.events().publish(
        ("prediction_market_factory", "SwarmMarketCreated"),
        (swarm_id, position, call_id, market_address.clone()),
    );
}

pub fn emit_swarm_auto_staked(
    env: &Env,
    swarm_id: u64,
    position: u32,
    user: &Address,
    amount: i128,
) {
    env.events().publish(
        ("prediction_market_factory", "SwarmAutoStaked"),
        (swarm_id, position, user.clone(), amount),
    );
}
