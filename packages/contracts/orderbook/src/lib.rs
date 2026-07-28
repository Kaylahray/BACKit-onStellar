#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

pub use types::{Order, OrderbookConfig, OrderSide};

use errors::OrderbookError;
use events::{emit_order_cancelled, emit_order_executed, emit_order_placed};
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
use storage::*;

const MAX_MATCHES_PER_ORDER: u32 = 20;
const PROTOCOL_FEE_BPS: u32 = 25;
const LP_FEE_BPS: u32 = 5;

#[contract]
pub struct Orderbook;

#[contractimpl]
impl Orderbook {
    pub fn initialize(env: Env, admin: Address) -> Result<(), OrderbookError> {
        if get_config(&env).is_some() {
            return Err(OrderbookError::AlreadyInitialized);
        }
        admin.require_auth();

        let config = OrderbookConfig {
            admin,
            fee_bps: 30,
            protocol_fee_bps: PROTOCOL_FEE_BPS,
            lp_fee_bps: LP_FEE_BPS,
        };
        set_config(&env, &config);
        Ok(())
    }

    pub fn place_bid(
        env: Env,
        user: Address,
        call_id: u64,
        outcome: u32,
        amount: i128,
        price_bps: u32,
    ) -> Result<u64, OrderbookError> {
        user.require_auth();

        if amount <= 0 {
            return Err(OrderbookError::InvalidAmount);
        }
        if price_bps == 0 || price_bps > 10000 {
            return Err(OrderbookError::InvalidPrice);
        }

        let order_id = next_order_id(&env);
        let order = Order {
            id: order_id,
            user: user.clone(),
            call_id,
            outcome,
            side: OrderSide::Bid,
            amount,
            price_bps,
            filled: 0,
            created_at: env.ledger().timestamp(),
            active: true,
        };

        set_order(&env, order_id, &order);
        add_user_order(&env, &user, order_id);
        add_to_book(&env, call_id, outcome, order_id, true);
        emit_order_placed(&env, order_id, &user, call_id, outcome, true);

        Self::match_orders(&env, order_id)?;

        Ok(order_id)
    }

    pub fn place_ask(
        env: Env,
        user: Address,
        call_id: u64,
        outcome: u32,
        share_amount: i128,
        price_bps: u32,
    ) -> Result<u64, OrderbookError> {
        user.require_auth();

        if share_amount <= 0 {
            return Err(OrderbookError::InvalidAmount);
        }
        if price_bps == 0 || price_bps > 10000 {
            return Err(OrderbookError::InvalidPrice);
        }

        let order_id = next_order_id(&env);
        let order = Order {
            id: order_id,
            user: user.clone(),
            call_id,
            outcome,
            side: OrderSide::Ask,
            amount: share_amount,
            price_bps,
            filled: 0,
            created_at: env.ledger().timestamp(),
            active: true,
        };

        set_order(&env, order_id, &order);
        add_user_order(&env, &user, order_id);
        add_to_book(&env, call_id, outcome, order_id, false);
        emit_order_placed(&env, order_id, &user, call_id, outcome, false);

        Self::match_orders(&env, order_id)?;

        Ok(order_id)
    }

    pub fn cancel_order(env: Env, user: Address, order_id: u64) -> Result<(), OrderbookError> {
        user.require_auth();

        let mut order = get_order(&env, order_id).ok_or(OrderbookError::OrderNotFound)?;
        if order.user != user {
            return Err(OrderbookError::Unauthorized);
        }
        if !order.active {
            return Err(OrderbookError::OrderNotFound);
        }

        order.active = false;
        set_order(&env, order_id, &order);
        emit_order_cancelled(&env, order_id);

        Ok(())
    }

    fn match_orders(env: &Env, taker_order_id: u64) -> Result<(), OrderbookError> {
        let taker = get_order(env, taker_order_id).ok_or(OrderbookError::OrderNotFound)?;
        if !taker.active {
            return Ok(());
        }

        let is_taker_bid = matches!(taker.side, OrderSide::Bid);
        let book_ids = get_book_ids(env, taker.call_id, taker.outcome, !is_taker_bid);
        let mut matches_count: u32 = 0;

        for i in 0..book_ids.len() {
            if matches_count >= MAX_MATCHES_PER_ORDER {
                break;
            }

            let maker_id = book_ids.get(i).unwrap();
            let mut maker = match get_order(env, maker_id) {
                Some(o) => o,
                None => continue,
            };

            if !maker.active || maker.user == taker.user {
                continue;
            }

            let price_compatible = if is_taker_bid {
                maker.price_bps <= taker.price_bps
            } else {
                maker.price_bps >= taker.price_bps
            };

            if !price_compatible {
                continue;
            }

            let taker_remaining = taker.amount - taker.filled;
            let maker_remaining = maker.amount - maker.filled;
            let fill_amount = core::cmp::min(taker_remaining, maker_remaining);

            if fill_amount <= 0 {
                continue;
            }

            let exec_price = maker.price_bps;

            maker.filled += fill_amount;
            if maker.filled >= maker.amount {
                maker.active = false;
            }
            set_order(env, maker_id, &maker);

            emit_order_executed(env, &maker.user, &taker.user, taker.call_id, taker.outcome, fill_amount, exec_price);

            matches_count += 1;
        }

        let mut updated_taker = taker;
        let total_filled: i128 = get_order(env, taker_order_id)
            .map(|o| o.filled)
            .unwrap_or(0);
        updated_taker.filled = total_filled;

        Ok(())
    }

    pub fn get_orderbook(
        env: Env,
        call_id: u64,
        outcome: u32,
    ) -> (Vec<Order>, Vec<Order>) {
        let bid_ids = get_book_ids(&env, call_id, outcome, true);
        let ask_ids = get_book_ids(&env, call_id, outcome, false);

        let mut bids = Vec::new(&env);
        let mut asks = Vec::new(&env);

        for i in 0..bid_ids.len() {
            if let Some(order) = get_order(&env, bid_ids.get(i).unwrap()) {
                if order.active {
                    bids.push_back(order);
                }
            }
        }

        for i in 0..ask_ids.len() {
            if let Some(order) = get_order(&env, ask_ids.get(i).unwrap()) {
                if order.active {
                    asks.push_back(order);
                }
            }
        }

        (bids, asks)
    }

    pub fn get_user_orders(env: Env, user: Address) -> Vec<Order> {
        let ids = get_user_order_ids(&env, &user);
        let mut result = Vec::new(&env);
        for i in 0..ids.len() {
            if let Some(order) = get_order(&env, ids.get(i).unwrap()) {
                result.push_back(order);
            }
        }
        result
    }

    pub fn get_config_view(env: Env) -> Result<OrderbookConfig, OrderbookError> {
        get_config(&env).ok_or(OrderbookError::NotInitialized)
    }
}
