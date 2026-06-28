#![allow(deprecated)]

use soroban_sdk::{Address, Env};

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
