//! Lightweight single-market prediction contract.
//!
//! Each instance holds exactly one market's call data, stake tracking, and
//! resolution logic. Deployed exclusively via [`prediction_market_factory`].
#![no_std]
#![allow(clippy::too_many_arguments)]

mod errors;
mod events;
mod storage;
mod types;

pub use types::{Call, ConditionType, MarketConfig, MarketInitArgs};

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, Env, Map};
use storage::*;

use errors::MarketError;
use events::{emit_call_created, emit_call_resolved, emit_market_initialized, emit_stake_added};

#[cfg(not(test))]
#[inline]
fn is_native_xlm(env: &Env, addr: &Address) -> bool {
    let sentinel = Address::from_string(&soroban_sdk::String::from_str(
        env,
        "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
    ));
    *addr == sentinel
}

#[cfg(test)]
fn is_native_xlm(env: &Env, addr: &Address) -> bool {
    let key = soroban_sdk::Symbol::new(env, "xlm_sac_addr");
    if let Some(sentinel) = env.storage().instance().get::<_, Address>(&key) {
        return *addr == sentinel;
    }
    false
}

fn transfer_token(env: &Env, stake_token: &Address, from: &Address, to: &Address, amount: i128) {
    if is_native_xlm(env, stake_token) {
        token::StellarAssetClient::new(env, stake_token).transfer(from, to, &amount);
    } else {
        token::Client::new(env, stake_token).transfer(from, to, &amount);
    }
}

fn require_call_id(config: &MarketConfig, call_id: u64) -> Result<(), MarketError> {
    if config.call_id != call_id {
        return Err(MarketError::InvalidCallId);
    }
    Ok(())
}

macro_rules! reentrancy_guard {
    ($env:expr) => {
        if storage::is_locked($env) {
            return Err(MarketError::Unauthorized);
        }
        storage::acquire_lock($env);
    };
}

#[contract]
pub struct PredictionMarket;

#[contractimpl]
impl PredictionMarket {
    /// Constructor invoked by the factory via `deploy_v2`.
    pub fn __constructor(
        env: Env,
        call_id: u64,
        creator: Address,
        outcome_manager: Address,
        factory: Address,
        min_stake: i128,
        max_stake_per_user: i128,
        staking_cutoff_secs: u64,
        args: MarketInitArgs,
    ) {
        if get_config(&env).is_some() {
            soroban_sdk::panic_with_error!(&env, MarketError::AlreadyInitialized);
        }

        let MarketInitArgs {
            stake_token,
            stake_amount,
            start_price,
            end_ts,
            token_address,
            pair_id,
            metadata_hash,
            condition,
            outcome_count,
        } = args;

        if stake_amount < min_stake || stake_amount <= 0 {
            soroban_sdk::panic_with_error!(&env, MarketError::InvalidStakeAmount);
        }
        if start_price <= 0 {
            soroban_sdk::panic_with_error!(&env, MarketError::InvalidStakeAmount);
        }
        if outcome_count < 2 {
            soroban_sdk::panic_with_error!(&env, MarketError::InvalidOutcomeCount);
        }

        let current_timestamp = env.ledger().timestamp();
        if end_ts <= current_timestamp {
            soroban_sdk::panic_with_error!(&env, MarketError::InvalidEndTime);
        }

        let config = MarketConfig {
            call_id,
            creator: creator.clone(),
            outcome_manager: outcome_manager.clone(),
            factory: factory.clone(),
            min_stake,
            max_stake_per_user,
            staking_cutoff_secs,
            paused: false,
        };
        set_config(&env, &config);

        let mut outcome_stakes = Map::new(&env);
        let mut stakes = Map::new(&env);
        for i in 1..=outcome_count {
            outcome_stakes.set(i, 0);
            stakes.set(i, Map::new(&env));
        }

        let call = Call {
            id: call_id,
            creator: creator.clone(),
            stake_token: stake_token.clone(),
            stake_amount,
            end_ts,
            token_address: token_address.clone(),
            pair_id: pair_id.clone(),
            metadata_hash: metadata_hash.clone(),
            outcome_count,
            outcome_stakes,
            stakes,
            outcome: 0,
            start_price,
            end_price: 0,
            condition,
            settled: false,
            voided: false,
            created_at: current_timestamp,
            cancelled: false,
            metadata_version: 0,
            share_tokens: Map::new(&env),
        };

        set_call(&env, &call);
        emit_market_initialized(&env, call_id, &creator, &stake_token, end_ts);
        emit_call_created(
            &env,
            call_id,
            &creator,
            &stake_token,
            stake_amount,
            start_price,
            end_ts,
            &token_address,
            &pair_id,
            &metadata_hash,
            outcome_count,
        );
    }

    /// Stake tokens on an outcome position.
    pub fn stake_on_call(
        env: Env,
        staker: Address,
        call_id: u64,
        amount: i128,
        position: u32,
    ) -> Result<Call, MarketError> {
        staker.require_auth();
        reentrancy_guard!(&env);

        let config = get_config(&env).ok_or(MarketError::NotInitialized)?;
        require_call_id(&config, call_id)?;
        if config.paused {
            return Err(MarketError::ContractPaused);
        }
        if amount <= 0 || amount < config.min_stake {
            return Err(MarketError::InvalidStakeAmount);
        }

        let mut call = get_call(&env).ok_or(MarketError::CallNotFound)?;
        let current_timestamp = env.ledger().timestamp();

        if current_timestamp >= call.end_ts {
            return Err(MarketError::CallEnded);
        }

        let cutoff = config.staking_cutoff_secs;
        if cutoff > 0 && call.end_ts > cutoff && current_timestamp >= call.end_ts - cutoff {
            return Err(MarketError::StakingCutoffActive);
        }

        if call.settled || call.cancelled || call.voided {
            return Err(MarketError::CallSettled);
        }

        if position < 1 || position > call.outcome_count {
            return Err(MarketError::InvalidPosition);
        }

        let mut outcome_stakers = call.stakes.get(position).unwrap_or_else(|| Map::new(&env));
        let current_staker_stake = outcome_stakers.get(staker.clone()).unwrap_or(0);
        let updated_staker_stake = current_staker_stake + amount;

        if config.max_stake_per_user > 0 && updated_staker_stake > config.max_stake_per_user {
            return Err(MarketError::InvalidStakeAmount);
        }

        transfer_token(
            &env,
            &call.stake_token,
            &staker,
            &env.current_contract_address(),
            amount,
        );

        let current_total = call.outcome_stakes.get(position).unwrap_or(0);
        call.outcome_stakes.set(position, current_total + amount);
        outcome_stakers.set(staker.clone(), updated_staker_stake);
        call.stakes.set(position, outcome_stakers);

        set_user_stake(&env, &staker, position, updated_staker_stake);
        set_call(&env, &call);
        emit_stake_added(&env, call_id, &staker, amount, position);

        storage::release_lock(&env);
        Ok(call)
    }

    /// Resolve the market (outcome_manager only).
    pub fn resolve_call(
        env: Env,
        call_id: u64,
        outcome: u32,
        end_price: i128,
    ) -> Result<Call, MarketError> {
        let config = get_config(&env).ok_or(MarketError::NotInitialized)?;
        require_call_id(&config, call_id)?;
        config.outcome_manager.require_auth();
        reentrancy_guard!(&env);

        let mut call = get_call(&env).ok_or(MarketError::CallNotFound)?;

        if outcome < 1 || outcome > call.outcome_count {
            return Err(MarketError::InvalidOutcome);
        }
        if env.ledger().timestamp() < call.end_ts {
            return Err(MarketError::CallNotEnded);
        }
        if call.voided {
            return Err(MarketError::Unauthorized);
        }

        call.outcome = outcome;
        call.end_price = end_price;
        set_call(&env, &call);
        emit_call_resolved(&env, call_id, outcome, end_price);

        storage::release_lock(&env);
        Ok(call)
    }

    /// Mark the market as settled (outcome_manager only).
    pub fn mark_settled(env: Env, call_id: u64) -> Result<(), MarketError> {
        let config = get_config(&env).ok_or(MarketError::NotInitialized)?;
        require_call_id(&config, call_id)?;
        config.outcome_manager.require_auth();

        let mut call = get_call(&env).ok_or(MarketError::CallNotFound)?;
        if call.settled {
            return Err(MarketError::CallSettled);
        }
        call.settled = true;
        set_call(&env, &call);
        Ok(())
    }

    /// Release escrowed tokens (outcome_manager only).
    pub fn release_escrow(
        env: Env,
        call_id: u64,
        to: Address,
        amount: i128,
    ) -> Result<(), MarketError> {
        let config = get_config(&env).ok_or(MarketError::NotInitialized)?;
        require_call_id(&config, call_id)?;
        config.outcome_manager.require_auth();

        let call = get_call(&env).ok_or(MarketError::CallNotFound)?;
        transfer_token(
            &env,
            &call.stake_token,
            &env.current_contract_address(),
            &to,
            amount,
        );
        Ok(())
    }

    /// Return the full call struct (compatible with outcome_manager cross-contract reads).
    pub fn get_call(env: Env, call_id: u64) -> Result<Call, MarketError> {
        let config = get_config(&env).ok_or(MarketError::NotInitialized)?;
        require_call_id(&config, call_id)?;
        get_call(&env).ok_or(MarketError::CallNotFound)
    }

    /// Return a staker's stake on a given outcome position.
    pub fn get_staker_stake(
        env: Env,
        call_id: u64,
        staker: Address,
        position: u32,
    ) -> Result<i128, MarketError> {
        let config = get_config(&env).ok_or(MarketError::NotInitialized)?;
        require_call_id(&config, call_id)?;
        Ok(get_user_stake(&env, &staker, position))
    }

    /// Return total stakes per outcome.
    pub fn get_outcome_stakes(env: Env, call_id: u64) -> Result<Map<u32, i128>, MarketError> {
        let call = Self::get_call(env, call_id)?;
        Ok(call.outcome_stakes)
    }

    /// Return this market's global call ID assigned by the factory.
    pub fn get_call_id(env: Env) -> Result<u64, MarketError> {
        let config = get_config(&env).ok_or(MarketError::NotInitialized)?;
        Ok(config.call_id)
    }

    /// Return the factory address that deployed this market.
    pub fn get_factory(env: Env) -> Result<Address, MarketError> {
        let config = get_config(&env).ok_or(MarketError::NotInitialized)?;
        Ok(config.factory)
    }
}
