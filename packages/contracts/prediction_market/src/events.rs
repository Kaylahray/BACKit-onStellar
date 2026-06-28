#![allow(deprecated)]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{Address, Bytes, BytesN, Env};

pub fn emit_market_initialized(
    env: &Env,
    call_id: u64,
    creator: &Address,
    stake_token: &Address,
    end_ts: u64,
) {
    env.events().publish(
        ("prediction_market", "initialized"),
        (call_id, creator.clone(), stake_token.clone(), end_ts),
    );
}

pub fn emit_stake_added(env: &Env, call_id: u64, staker: &Address, amount: i128, position: u32) {
    env.events().publish(
        ("prediction_market", "stake_added"),
        (call_id, staker.clone(), amount, position),
    );
}

pub fn emit_call_resolved(env: &Env, call_id: u64, outcome: u32, end_price: i128) {
    env.events().publish(
        ("prediction_market", "resolved"),
        (call_id, outcome, end_price),
    );
}

pub fn emit_call_created(
    env: &Env,
    call_id: u64,
    creator: &Address,
    stake_token: &Address,
    stake_amount: i128,
    start_price: i128,
    end_ts: u64,
    token_address: &Address,
    pair_id: &Bytes,
    metadata_hash: &BytesN<32>,
    outcome_count: u32,
) {
    env.events().publish(
        ("prediction_market", "created"),
        (
            call_id,
            creator.clone(),
            stake_token.clone(),
            stake_amount,
            start_price,
            end_ts,
            token_address.clone(),
            pair_id.clone(),
            metadata_hash.clone(),
            outcome_count,
        ),
    );
}
