#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env,
};

use crate::DutchAuction;
use crate::DutchAuctionClient;

fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let outcome_manager = Address::generate(&env);
    let caller = Address::generate(&env);
    (env, admin, outcome_manager, caller)
}

fn default_ledger(ts: u64) -> LedgerInfo {
    LedgerInfo {
        protocol_version: 23,
        sequence_number: 1,
        timestamp: ts,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: 0,
    }
}

fn init_contract(
    env: &Env,
    admin: &Address,
    outcome_manager: &Address,
) -> DutchAuctionClient {
    let contract_id = env.register(DutchAuction, ());
    let client = DutchAuctionClient::new(env, &contract_id);
    client.initialize(admin, outcome_manager, &3600, &86400, &100);
    client
}

#[test]
fn test_initialize_success() {
    let (env, admin, outcome_manager, _) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    let info = client.get_auction_info(&1);
    assert_eq!(info, None);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialize_fails() {
    let (env, admin, outcome_manager, _) = setup();
    let contract_id = env.register(DutchAuction, ());
    let client = DutchAuctionClient::new(&env, &contract_id);
    client.initialize(&admin, &outcome_manager, &3600, &86400, &100);
    client.initialize(&admin, &outcome_manager, &3600, &86400, &100);
}

#[test]
fn test_start_and_get_auction_info() {
    let (env, admin, outcome_manager, caller) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    env.ledger().set(default_ledger(1_000_000));
    client.start_dutch_auction(&1, &1000_0000000, &1);
    let info = client.get_auction_info(&1).unwrap();
    assert_eq!(info.call_id, 1);
    assert_eq!(info.condition_type, 1);
    assert_eq!(info.start_price, 1000_0000000);
    assert_eq!(info.settled, false);
}

#[test]
fn test_price_decay_target_above() {
    let (env, admin, outcome_manager, _) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    env.ledger().set(default_ledger(1_000_000));
    let start_price: i128 = 1000_0000000;
    client.start_dutch_auction(&1, &start_price, &1);
    let price_at_start = client.get_dutch_auction_price(&1);
    assert_eq!(price_at_start, start_price * 2);

    env.ledger().set(default_ledger(1_001_800));
    let price_mid = client.get_dutch_auction_price(&1);
    assert_eq!(price_mid, start_price * 2 * (3600 - 1800) / 3600);

    env.ledger().set(default_ledger(1_003_600));
    let price_end = client.get_dutch_auction_price(&1);
    assert_eq!(price_end, 0);
}

#[test]
fn test_price_decay_target_below() {
    let (env, admin, outcome_manager, _) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    env.ledger().set(default_ledger(1_000_000));
    let start_price: i128 = 1000_0000000;
    client.start_dutch_auction(&1, &start_price, &2);
    let price_at_start = client.get_dutch_auction_price(&1);
    assert_eq!(price_at_start, start_price / 2);

    env.ledger().set(default_ledger(1_001_800));
    let price_mid = client.get_dutch_auction_price(&1);
    let numerator = start_price * (3600i128 + 3 * 1800i128);
    let expected_mid = numerator / (2 * 3600i128);
    assert_eq!(price_mid, expected_mid);

    env.ledger().set(default_ledger(1_003_600));
    let price_end = client.get_dutch_auction_price(&1);
    assert_eq!(price_end, start_price * 2);
}

#[test]
fn test_settle_auction() {
    let (env, admin, outcome_manager, caller) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    env.ledger().set(default_ledger(1_000_000));
    client.start_dutch_auction(&1, &1000_0000000, &1);
    env.ledger().set(default_ledger(1_001_800));
    let price = client.settle_dutch_auction(&caller, &1);
    let expected_price = 1000_0000000 * 2 * (3600 - 1800) / 3600;
    assert_eq!(price, expected_price);
    let info = client.get_auction_info(&1).unwrap();
    assert_eq!(info.settled, true);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_double_settle_fails() {
    let (env, admin, outcome_manager, caller) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    env.ledger().set(default_ledger(1_000_000));
    client.start_dutch_auction(&1, &1000_0000000, &1);
    env.ledger().set(default_ledger(1_001_800));
    client.settle_dutch_auction(&caller, &1);
    client.settle_dutch_auction(&caller, &1);
}

#[test]
fn test_update_params() {
    let (env, admin, outcome_manager, _) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    client.update_params(&7200, &43200, &200);
    env.ledger().set(default_ledger(1_000_000));
    client.start_dutch_auction(&1, &1000_0000000, &1);
    env.ledger().set(default_ledger(1_003_600));
    let price_mid = client.get_dutch_auction_price(&1);
    let expected = 1000_0000000 * 2 * (7200 - 3600) / 7200;
    assert_eq!(price_mid, expected);
}

#[test]
fn test_settle_at_end_price_target_above() {
    let (env, admin, outcome_manager, caller) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    env.ledger().set(default_ledger(1_000_000));
    client.start_dutch_auction(&1, &500_0000000, &1);
    env.ledger().set(default_ledger(1_003_600));
    let price = client.settle_dutch_auction(&caller, &1);
    assert_eq!(price, 0);
}

#[test]
fn test_start_with_invalid_price_fails() {
    let (env, admin, outcome_manager, _) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    env.ledger().set(default_ledger(1_000_000));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.start_dutch_auction(&1, &0, &1);
    }));
    assert!(result.is_err());
}

#[test]
fn test_settle_unstarted_auction_fails() {
    let (env, admin, outcome_manager, caller) = setup();
    let client = init_contract(&env, &admin, &outcome_manager);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.settle_dutch_auction(&caller, &42);
    }));
    assert!(result.is_err());
}
