#![allow(deprecated)]

use soroban_sdk::{Address, Env};

pub fn emit_deposited(env: &Env, user: &Address, amount: i128, lp_shares_minted: i128) {
    env.events().publish(
        ("lending_pool", "Deposited"),
        (user.clone(), amount, lp_shares_minted),
    );
}

pub fn emit_withdrawn(env: &Env, user: &Address, lp_shares_burned: i128, amount_out: i128) {
    env.events().publish(
        ("lending_pool", "Withdrawn"),
        (user.clone(), lp_shares_burned, amount_out),
    );
}

pub fn emit_capital_allocated(
    env: &Env,
    call_id: u64,
    market_address: &Address,
    position: u32,
    amount: i128,
) {
    env.events().publish(
        ("lending_pool", "CapitalAllocated"),
        (call_id, market_address.clone(), position, amount),
    );
}

pub fn emit_yield_harvested(
    env: &Env,
    call_id: u64,
    won: bool,
    gross_change: i128,
    protocol_fee: i128,
    net_yield: i128,
) {
    env.events().publish(
        ("lending_pool", "YieldHarvested"),
        (call_id, won, gross_change, protocol_fee, net_yield),
    );
}
