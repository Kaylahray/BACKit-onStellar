#![cfg(test)]

extern crate std;

use crate::{CrossChainOracleRelay, CrossChainOracleRelayClient};
use soroban_sdk::{
    testutils::Address as _,
    Address, BytesN, Env, String, Vec,
};

#[test]
fn test_initialize_and_relay() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    let outcome_manager = Address::generate(&env);
    let stellar_oracle = Address::generate(&env);
    let fee_token = Address::generate(&env);

    let contract_id = env.register(CrossChainOracleRelay, ());
    let client = CrossChainOracleRelayClient::new(&env, &contract_id);

    let relayers = Vec::from_array(&env, [relayer.clone()]);
    client.initialize(&admin, &outcome_manager, &stellar_oracle, &1, &0, &fee_token, &relayers);

    let header = crate::types::BlockHeader {
        parent_hash: BytesN::from_array(&env, &[1u8; 32]),
        state_root: BytesN::from_array(&env, &[2u8; 32]),
        transactions_root: BytesN::from_array(&env, &[3u8; 32]),
        receipts_root: BytesN::from_array(&env, &[4u8; 32]),
        block_number: 100,
        timestamp: 1234567890,
    };
    client.submit_block_header(&relayer, &header);

    let proof = Vec::from_array(&env, [BytesN::from_array(&env, &[5u8; 32])]);
    let root = BytesN::from_array(&env, &[0u8; 32]);
    client.relay_price(
        &relayer,
        &String::from_str(&env, "ethereum:1"),
        &1_000_000_000,
        &100,
        &proof,
        &root,
    );

    let source = client.get_cross_chain_source(&String::from_str(&env, "ethereum:1"));
    assert!(source.is_some());
    assert_eq!(source.unwrap().last_price, 1_000_000_000);
}
