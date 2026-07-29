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

pub fn emit_strategy_created(env: &Env, strategy_id: u64, user: &Address, escrow_amount: i128) {
    env.events().publish(
        ("prediction_market_factory", "StrategyCreated"),
        (strategy_id, user.clone(), escrow_amount),
    );
}

pub fn emit_strategy_executed(
    env: &Env,
    strategy_id: u64,
    actions_executed: u32,
    keeper_reward: i128,
) {
    env.events().publish(
        ("prediction_market_factory", "StrategyExecuted"),
        (strategy_id, actions_executed, keeper_reward),
    );
}

pub fn emit_strategy_cancelled(env: &Env, strategy_id: u64) {
    env.events().publish(
        ("prediction_market_factory", "StrategyCancelled"),
        (strategy_id,),
    );
}
