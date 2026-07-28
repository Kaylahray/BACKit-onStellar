#![allow(deprecated)]

use soroban_sdk::{Address, Env, Symbol};

pub fn emit_parlay_created(
    env: &Env,
    parlay_id: u64,
    user: &Address,
    initial_stake: i128,
    leg_count: u32,
) {
    env.events().publish(
        ("parlay_betting", "created"),
        (parlay_id, user.clone(), initial_stake, leg_count),
    );
}

pub fn emit_parlay_leg_resolved(env: &Env, parlay_id: u64, leg_index: u32, won: bool) {
    env.events().publish(
        ("parlay_betting", "leg_resolved"),
        (parlay_id, leg_index, won),
    );
}

pub fn emit_parlay_completed(env: &Env, parlay_id: u64, total_payout: i128) {
    env.events()
        .publish(("parlay_betting", "completed"), (parlay_id, total_payout));
}

pub fn emit_parlay_voided(env: &Env, parlay_id: u64, reason: Symbol) {
    env.events()
        .publish(("parlay_betting", "voided"), (parlay_id, reason));
}
