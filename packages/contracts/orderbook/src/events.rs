#![allow(deprecated)]

use soroban_sdk::{Address, Env};

pub fn emit_order_placed(env: &Env, order_id: u64, user: &Address, call_id: u64, outcome: u32, is_bid: bool) {
    env.events().publish(
        ("orderbook", "OrderPlaced"),
        (order_id, user.clone(), call_id, outcome, is_bid),
    );
}

pub fn emit_order_cancelled(env: &Env, order_id: u64) {
    env.events().publish(
        ("orderbook", "OrderCancelled"),
        (order_id,),
    );
}

pub fn emit_order_executed(
    env: &Env,
    maker: &Address,
    taker: &Address,
    call_id: u64,
    outcome: u32,
    amount: i128,
    price: u32,
) {
    env.events().publish(
        ("orderbook", "OrderExecuted"),
        (maker.clone(), taker.clone(), call_id, outcome, amount, price),
    );
}
