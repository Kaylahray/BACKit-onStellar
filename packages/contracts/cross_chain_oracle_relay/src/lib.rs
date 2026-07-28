#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use errors::OracleRelayError;
use events::{emit_header_submitted, emit_price_relayed};
use soroban_sdk::{
    contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec,
};
use storage::*;
use types::{BlockHeader, CrossChainSource, OracleRelayConfig};

const ETH_CHAIN_ID: u64 = 1;

#[contract]
pub struct CrossChainOracleRelay;

#[contractimpl]
impl CrossChainOracleRelay {
    pub fn initialize(
        env: Env,
        admin: Address,
        outcome_manager: Address,
        stellar_oracle: Address,
        min_signatures: u32,
        fee_per_submission: i128,
        fee_token: Address,
        trusted_relayers: Vec<Address>,
    ) -> Result<(), OracleRelayError> {
        if get_config(&env).is_some() {
            return Err(OracleRelayError::AlreadyInitialized);
        }
        admin.require_auth();
        let config = OracleRelayConfig {
            admin: admin.clone(),
            outcome_manager,
            stellar_oracle,
            min_signatures,
            fee_per_submission,
            fee_token,
            trusted_relayers,
        };
        set_config(&env, &config);
        Ok(())
    }

    pub fn submit_block_header(
        env: Env,
        relayer: Address,
        header: BlockHeader,
    ) -> Result<(), OracleRelayError> {
        relayer.require_auth();
        let config = get_config(&env).ok_or(OracleRelayError::NotInitialized)?;

        let authorized = config.trusted_relayers.iter().any(|a| a == relayer);
        if !authorized {
            return Err(OracleRelayError::UnauthorizedRelayer);
        }

        if is_block_relayed(&env, header.block_number) {
            return Err(OracleRelayError::BlockAlreadyRelayed);
        }

        let block_hash = Self::compute_block_hash(&env, &header);
        add_trusted_hash(&env, header.block_number, &block_hash);
        mark_block_relayed(&env, header.block_number);

        emit_header_submitted(&env, &relayer, header.block_number, &block_hash);
        Ok(())
    }

    pub fn relay_price(
        env: Env,
        relayer: Address,
        chain_name: String,
        price: i128,
        block_number: u64,
        merkle_proof: Vec<BytesN<32>>,
        expected_root: BytesN<32>,
    ) -> Result<(), OracleRelayError> {
        relayer.require_auth();
        let config = get_config(&env).ok_or(OracleRelayError::NotInitialized)?;

        if price <= 0 || price > 1_000_000_000_000_000_000 {
            return Err(OracleRelayError::PriceOutOfRange);
        }

        let authorized = config.trusted_relayers.iter().any(|a| a == relayer);
        if !authorized {
            return Err(OracleRelayError::UnauthorizedRelayer);
        }

        let trusted = get_trusted_hash(&env, block_number);
        if trusted.is_none() {
            return Err(OracleRelayError::InvalidBlockHeader);
        }

        Self::verify_merkle_proof(&env, &merkle_proof, &expected_root);

        let source = CrossChainSource {
            chain_name: chain_name.clone(),
            latest_block: block_number,
            last_relayed_at: env.ledger().timestamp(),
            last_price: price,
        };
        set_source(&env, &chain_name, &source);

        emit_price_relayed(&env, &chain_name, price, block_number);
        Ok(())
    }

    pub fn get_cross_chain_source(env: Env, chain_name: String) -> Option<CrossChainSource> {
        storage::get_source(&env, &chain_name)
    }

    pub fn get_all_sources(env: Env) -> Vec<CrossChainSource> {
        let _config = match get_config(&env) {
            Some(c) => c,
            None => return Vec::new(&env),
        };
        let mut v = Vec::new(&env);
        let eth = String::from_str(&env, "ethereum:1");
        if let Some(s) = storage::get_source(&env, &eth) {
            v.push_back(s);
        }
        v
    }

    pub fn get_trusted_block_hash(env: Env, block_number: u64) -> Option<BytesN<32>> {
        get_trusted_hash(&env, block_number)
    }

    pub fn fallback_to_stellar_oracle(env: Env) -> Result<i128, OracleRelayError> {
        let config = get_config(&env).ok_or(OracleRelayError::NotInitialized)?;
        let result: Result<i128, soroban_sdk::Error> = env.invoke_contract(
            &config.stellar_oracle,
            &soroban_sdk::Symbol::new(&env, "latest_price"),
            soroban_sdk::vec![&env],
        );
        result.map_err(|_| OracleRelayError::InvalidBlockHeader)
    }

    fn compute_block_hash(env: &Env, header: &BlockHeader) -> BytesN<32> {
        let mut raw = Bytes::from_slice(env, b"eth_block:");
        raw.append(&Bytes::from_slice(env, &header.block_number.to_be_bytes()));
        raw.append(&Bytes::from_slice(env, &header.parent_hash.to_array()));
        raw.append(&Bytes::from_slice(env, &header.state_root.to_array()));
        raw.append(&Bytes::from_slice(env, &header.transactions_root.to_array()));
        raw.append(&Bytes::from_slice(env, &header.receipts_root.to_array()));
        raw.append(&Bytes::from_slice(env, &header.timestamp.to_be_bytes()));
        env.crypto().sha256(&raw).into()
    }

    fn verify_merkle_proof(
        env: &Env,
        proof: &Vec<BytesN<32>>,
        _expected_root: &BytesN<32>,
    ) {
        if proof.is_empty() {
            soroban_sdk::panic_with_error!(env, OracleRelayError::MerkleProofVerificationFailed);
        }
        let mut computed: BytesN<32> = proof.get(0).unwrap();
        for i in 1..proof.len() {
            let next: BytesN<32> = proof.get(i).unwrap();
            let mut combined = Bytes::from_slice(env, b"merkle:");
            combined.append(&Bytes::from_slice(env, &computed.to_array()));
            combined.append(&Bytes::from_slice(env, &next.to_array()));
            computed = env.crypto().sha256(&combined).into();
        }
    }
}
