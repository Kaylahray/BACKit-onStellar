#![allow(deprecated)]

use soroban_sdk::{Address, Env};

pub fn emit_deposited(env: &Env, user: &Address, amount: i128, shares_minted: i128) {
    env.events().publish(
        ("yield_aggregator", "Deposited"),
        (user.clone(), amount, shares_minted),
    );
}

pub fn emit_withdrawn(env: &Env, user: &Address, shares_burned: i128, amount_returned: i128) {
    env.events().publish(
        ("yield_aggregator", "Withdrawn"),
        (user.clone(), shares_burned, amount_returned),
    );
}

pub fn emit_reinvested(env: &Env, call_id: u64, amount: i128) {
    env.events().publish(
        ("yield_aggregator", "Reinvested"),
        (call_id, amount),
    );
}

pub fn emit_fee_collected(env: &Env, amount: i128) {
    env.events().publish(
        ("yield_aggregator", "FeeCollected"),
        (amount,),
    );
}
