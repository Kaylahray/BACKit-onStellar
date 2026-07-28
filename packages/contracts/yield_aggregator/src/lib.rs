#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

pub use types::{AggregatorConfig, AggregatorStats, UserPosition};

use errors::AggregatorError;
use events::{emit_deposited, emit_fee_collected, emit_reinvested, emit_withdrawn};
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct YieldAggregator;

#[contractimpl]
impl YieldAggregator {
    pub fn initialize(env: Env, admin: Address, fee_bps: u32) -> Result<(), AggregatorError> {
        if storage::get_config(&env).is_some() {
            return Err(AggregatorError::AlreadyInitialized);
        }
        admin.require_auth();

        let config = AggregatorConfig { admin, fee_bps };
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn deposit(
        env: Env,
        user: Address,
        usdc_amount: i128,
    ) -> Result<i128, AggregatorError> {
        user.require_auth();

        if usdc_amount <= 0 {
            return Err(AggregatorError::InvalidAmount);
        }

        let total_shares = storage::get_total_shares(&env);
        let total_deposited = storage::get_total_deposited(&env);

        let shares_minted = if total_shares == 0 {
            usdc_amount
        } else {
            usdc_amount
                .checked_mul(total_shares)
                .ok_or(AggregatorError::InvalidAmount)?
                .checked_div(total_deposited)
                .ok_or(AggregatorError::ZeroTotalAssets)?
        };

        let user_shares = storage::get_user_shares(&env, &user);
        storage::set_user_shares(&env, &user, user_shares + shares_minted);
        storage::set_total_shares(&env, total_shares + shares_minted);
        storage::set_total_deposited(&env, total_deposited + usdc_amount);

        emit_deposited(&env, &user, usdc_amount, shares_minted);

        Ok(shares_minted)
    }

    pub fn withdraw(
        env: Env,
        user: Address,
        a_backit_amount: i128,
    ) -> Result<i128, AggregatorError> {
        user.require_auth();

        let user_shares = storage::get_user_shares(&env, &user);
        if user_shares < a_backit_amount {
            return Err(AggregatorError::InsufficientShares);
        }

        let total_shares = storage::get_total_shares(&env);
        let total_deposited = storage::get_total_deposited(&env);

        let usdc_returned = a_backit_amount
            .checked_mul(total_deposited)
            .ok_or(AggregatorError::InvalidAmount)?
            .checked_div(total_shares)
            .ok_or(AggregatorError::ZeroTotalAssets)?;

        storage::set_user_shares(&env, &user, user_shares - a_backit_amount);
        storage::set_total_shares(&env, total_shares - a_backit_amount);
        storage::set_total_deposited(&env, total_deposited - usdc_returned);

        emit_withdrawn(&env, &user, a_backit_amount, usdc_returned);

        Ok(usdc_returned)
    }

    pub fn claim_and_reinvest(
        env: Env,
        call_id: u64,
    ) -> Result<i128, AggregatorError> {
        let config = storage::get_config(&env).ok_or(AggregatorError::NotInitialized)?;

        let total_deposited = storage::get_total_deposited(&env);
        let total_shares = storage::get_total_shares(&env);

        if total_shares == 0 {
            return Err(AggregatorError::ZeroTotalAssets);
        }

        let profit = total_deposited / 10;
        let fee = profit
            .checked_mul(config.fee_bps as i128)
            .ok_or(AggregatorError::InvalidAmount)?
            .checked_div(10000)
            .ok_or(AggregatorError::InvalidAmount)?;

        let total_profits = storage::get_total_profits(&env);
        storage::set_total_profits(&env, total_profits + profit - fee);

        if fee > 0 {
            emit_fee_collected(&env, fee);
        }

        emit_reinvested(&env, call_id, profit);

        Ok(profit)
    }

    pub fn get_aggregator_stats(env: Env) -> Result<AggregatorStats, AggregatorError> {
        let total_shares = storage::get_total_shares(&env);
        let total_deposited = storage::get_total_deposited(&env);
        let total_profits = storage::get_total_profits(&env);

        let share_price = if total_shares > 0 {
            total_deposited
                .checked_mul(10000)
                .ok_or(AggregatorError::InvalidAmount)?
                .checked_div(total_shares)
                .ok_or(AggregatorError::ZeroTotalAssets)?
        } else {
            10000
        };

        Ok(AggregatorStats {
            total_value_locked: total_deposited,
            total_shares,
            share_price,
            total_profits_earned: total_profits,
        })
    }

    pub fn get_user_position(env: Env, user: Address) -> Result<UserPosition, AggregatorError> {
        let shares = storage::get_user_shares(&env, &user);
        let total_shares = storage::get_total_shares(&env);
        let total_deposited = storage::get_total_deposited(&env);

        let deposited_amount = if total_shares > 0 {
            shares
                .checked_mul(total_deposited)
                .ok_or(AggregatorError::InvalidAmount)?
                .checked_div(total_shares)
                .ok_or(AggregatorError::ZeroTotalAssets)?
        } else {
            0
        };

        Ok(UserPosition {
            user,
            shares,
            deposited_amount,
            pending_payout: 0,
        })
    }

    pub fn get_config_view(env: Env) -> Result<AggregatorConfig, AggregatorError> {
        storage::get_config(&env).ok_or(AggregatorError::NotInitialized)
    }
}
