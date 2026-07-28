//! Factory contract that deploys isolated [`prediction_market`] instances.
//!
//! Each market is a separate contract deployed from a pre-uploaded WASM hash,
//! isolating risk and storage pressure compared to the monolithic `call_registry`.
#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use errors::FactoryError;
use events::{
    emit_factory_initialized, emit_market_deployed, emit_swarm_auto_staked, emit_swarm_created,
    emit_swarm_market_created,
};
use prediction_market::MarketInitArgs;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, Map, String, Vec};
use storage::*;
use types::{FactoryConfig, Swarm, SwarmStage};

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

fn market_deploy_salt(env: &Env, call_id: u64) -> BytesN<32> {
    let mut raw = Bytes::from_slice(env, b"market:");
    raw.append(&Bytes::from_slice(env, &call_id.to_be_bytes()));
    env.crypto().sha256(&raw).into()
}

#[contract]
pub struct PredictionMarketFactory;

#[contractimpl]
impl PredictionMarketFactory {
    /// Initialise the factory with admin, outcome manager, and market WASM hash.
    pub fn initialize(
        env: Env,
        admin: Address,
        outcome_manager: Address,
        market_wasm_hash: BytesN<32>,
        min_stake: i128,
    ) -> Result<(), FactoryError> {
        if get_config(&env).is_some() {
            return Err(FactoryError::AlreadyInitialized);
        }
        admin.require_auth();

        let config = FactoryConfig {
            admin: admin.clone(),
            outcome_manager: outcome_manager.clone(),
            market_wasm_hash,
            min_stake,
            max_stake_per_user: 0,
            staking_cutoff_secs: 300,
            paused: false,
            whitelisted_tokens: Map::new(&env),
        };
        set_config(&env, &config);
        emit_factory_initialized(&env, &admin, &outcome_manager);
        Ok(())
    }

    /// Deploy a new prediction market instance and return its contract address.
    pub fn deploy_market(
        env: Env,
        creator: Address,
        args: MarketInitArgs,
    ) -> Result<Address, FactoryError> {
        creator.require_auth();

        let config = get_config(&env).ok_or(FactoryError::NotInitialized)?;
        if config.paused {
            return Err(FactoryError::ContractPaused);
        }

        let MarketInitArgs {
            stake_token,
            stake_amount,
            start_price,
            end_ts,
            token_address: _,
            pair_id: _,
            metadata_hash: _,
            condition: _,
            outcome_count,
        } = &args;

        if *stake_amount < config.min_stake || *stake_amount <= 0 {
            return Err(FactoryError::InvalidStakeAmount);
        }
        if *start_price <= 0 {
            return Err(FactoryError::InvalidStakeAmount);
        }
        if *outcome_count < 2 {
            return Err(FactoryError::InvalidOutcomeCount);
        }
        if *end_ts <= env.ledger().timestamp() {
            return Err(FactoryError::InvalidEndTime);
        }

        if !is_native_xlm(&env, stake_token)
            && !config
                .whitelisted_tokens
                .get(stake_token.clone())
                .unwrap_or(false)
        {
            return Err(FactoryError::TokenNotWhitelisted);
        }

        let call_id = next_market_id(&env);
        let salt = market_deploy_salt(&env, call_id);
        let factory_addr = env.current_contract_address();

        let market_addr = env
            .deployer()
            .with_address(factory_addr.clone(), salt)
            .deploy_v2(
                config.market_wasm_hash.clone(),
                (
                    call_id,
                    creator.clone(),
                    config.outcome_manager.clone(),
                    factory_addr,
                    config.min_stake,
                    config.max_stake_per_user,
                    config.staking_cutoff_secs,
                    args.clone(),
                ),
            );

        set_market(&env, call_id, &market_addr);
        append_market_list(&env, &market_addr);

        emit_market_deployed(&env, call_id, &market_addr, &creator, stake_token, *end_ts);

        Ok(market_addr)
    }

    /// Return a paginated slice of deployed market addresses.
    pub fn get_all_markets(env: Env, start: u32, limit: u32) -> Vec<Address> {
        let list = get_market_list(&env);
        let total = list.len();
        let mut result = Vec::new(&env);

        if start >= total {
            return result;
        }

        let end = core::cmp::min(start.saturating_add(limit), total);
        for i in start..end {
            result.push_back(list.get(i).unwrap());
        }
        result
    }

    /// Return the total number of markets deployed by this factory.
    pub fn get_market_count(env: Env) -> u32 {
        storage::get_market_count(&env)
    }

    /// Look up a market address by its global call ID.
    pub fn get_market(env: Env, call_id: u64) -> Result<Address, FactoryError> {
        storage::get_market(&env, call_id).ok_or(FactoryError::MarketNotFound)
    }

    /// Whitelist a SAC token for use as a stake token in new markets.
    pub fn whitelist_token(env: Env, token: Address) -> Result<(), FactoryError> {
        let mut config = get_config(&env).ok_or(FactoryError::NotInitialized)?;
        config.admin.require_auth();
        config.whitelisted_tokens.set(token, true);
        set_config(&env, &config);
        Ok(())
    }

    /// Update the market WASM hash (admin only). New deployments use the updated hash.
    pub fn set_market_wasm_hash(
        env: Env,
        market_wasm_hash: BytesN<32>,
    ) -> Result<(), FactoryError> {
        let mut config = get_config(&env).ok_or(FactoryError::NotInitialized)?;
        config.admin.require_auth();
        config.market_wasm_hash = market_wasm_hash;
        set_config(&env, &config);
        Ok(())
    }

    /// Set the trusted outcome manager address (admin only).
    pub fn set_outcome_manager(env: Env, outcome_manager: Address) -> Result<(), FactoryError> {
        let mut config = get_config(&env).ok_or(FactoryError::NotInitialized)?;
        config.admin.require_auth();
        config.outcome_manager = outcome_manager;
        set_config(&env, &config);
        Ok(())
    }

    /// Pause new market deployments (admin only).
    pub fn pause(env: Env) -> Result<(), FactoryError> {
        let mut config = get_config(&env).ok_or(FactoryError::NotInitialized)?;
        config.admin.require_auth();
        config.paused = true;
        set_config(&env, &config);
        Ok(())
    }

    /// Unpause market deployments (admin only).
    pub fn unpause(env: Env) -> Result<(), FactoryError> {
        let mut config = get_config(&env).ok_or(FactoryError::NotInitialized)?;
        config.admin.require_auth();
        config.paused = false;
        set_config(&env, &config);
        Ok(())
    }

    /// Return the factory configuration.
    pub fn get_config(env: Env) -> Result<FactoryConfig, FactoryError> {
        storage::get_config(&env).ok_or(FactoryError::NotInitialized)
    }

    // ──── Swarm (Linked Markets) ──────────────────────────────────────────

    pub fn create_swarm(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        stages: Vec<SwarmStage>,
    ) -> Result<u64, FactoryError> {
        creator.require_auth();
        if get_config(&env).is_none() {
            return Err(FactoryError::NotInitialized);
        }

        if stages.len() < 2 {
            return Err(FactoryError::InvalidSwarmStage);
        }

        let id = next_swarm_id(&env);
        let swarm = Swarm {
            id,
            creator: creator.clone(),
            title: title.clone(),
            description: description.clone(),
            stages: stages.len(),
            created_at: env.ledger().timestamp(),
            active: true,
        };
        set_swarm(&env, id, &swarm);

        emit_swarm_created(&env, id, &creator, &title, stages.len());
        Ok(id)
    }

    pub fn stake_on_swarm(
        env: Env,
        user: Address,
        swarm_id: u64,
        _position: u32,
    ) -> Result<(), FactoryError> {
        user.require_auth();
        let config = get_config(&env).ok_or(FactoryError::NotInitialized)?;
        let swarm = get_swarm(&env, swarm_id).ok_or(FactoryError::SwarmNotFound)?;
        if !swarm.active {
            return Err(FactoryError::SwarmComplete);
        }
        let call_id = next_market_id(&env);
        let salt = market_deploy_salt(&env, call_id);
        let factory_addr = env.current_contract_address();

        let token = config.whitelisted_tokens.keys().get(0).ok_or(FactoryError::TokenNotWhitelisted)?;
        let pair = prediction_market::ConditionType::TargetAbove(100_000_0000i128);
        let args = MarketInitArgs {
            stake_token: token.clone(),
            stake_amount: config.min_stake,
            start_price: 100_000_0000,
            end_ts: env.ledger().timestamp() + 86400,
            token_address: token,
            pair_id: Bytes::from_slice(&env, b"swarm"),
            metadata_hash: BytesN::from_array(&env, &[0u8; 32]),
            condition: pair,
            outcome_count: 2,
        };

        let market_addr = env
            .deployer()
            .with_address(factory_addr.clone(), salt)
            .deploy_v2(
                config.market_wasm_hash.clone(),
                (
                    call_id,
                    user.clone(),
                    config.outcome_manager.clone(),
                    factory_addr,
                    config.min_stake,
                    config.max_stake_per_user,
                    config.staking_cutoff_secs,
                    args.clone(),
                ),
            );

        set_market(&env, call_id, &market_addr);
        append_market_list(&env, &market_addr);
        set_swarm_market(&env, swarm_id, _position, call_id);
        set_call_swarm(&env, call_id, swarm_id, _position);

        emit_swarm_market_created(&env, swarm_id, _position, call_id, &market_addr);
        emit_swarm_auto_staked(&env, swarm_id, _position, &user, config.min_stake);
        Ok(())
    }

    pub fn get_swarm(env: Env, swarm_id: u64) -> Result<Swarm, FactoryError> {
        storage::get_swarm(&env, swarm_id).ok_or(FactoryError::SwarmNotFound)
    }

    pub fn get_swarm_markets(env: Env, swarm_id: u64) -> Vec<u64> {
        storage::get_swarm_markets(&env, swarm_id)
    }

    pub fn get_call_swarm(env: Env, call_id: u64) -> Option<(u64, u32)> {
        storage::get_call_swarm(&env, call_id)
    }
}
