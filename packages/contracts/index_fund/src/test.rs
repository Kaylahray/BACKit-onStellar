#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::IndexFund;

fn create_test_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let stake_token = Address::generate(&env);
    let factory = Address::generate(&env);
    (env, admin, stake_token, factory)
}

fn setup_fund(env: &Env, admin: &Address, stake_token: &Address, factory: &Address) {
    env.mock_all_auths();
    IndexFund::initialize(
        env.clone(),
        admin.clone(),
        stake_token.clone(),
        factory.clone(),
        3600,  // rebalance every hour
        50,    // 0.5 % deposit fee
        50,    // 0.5 % withdraw fee
    );
}

#[test]
fn test_initialize() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    assert_eq!(IndexFund::get_admin(&env).unwrap(), admin);
    assert_eq!(IndexFund::get_stake_token(&env).unwrap(), stake_token);
    assert_eq!(IndexFund::get_nav(&env).unwrap(), 0);
    assert_eq!(IndexFund::get_index_composition(&env).unwrap().len(), 0);
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_double_initialize() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);
    setup_fund(&env, &admin, &stake_token, &factory);
}

#[test]
fn test_first_deposit_mints_at_par() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    let user = Address::generate(&env);
    env.mock_all_auths();

    // First deposit: 1000 USDC -> 1000 * 1e7 INDEX tokens
    let index_tokens = IndexFund::deposit(&env, user.clone(), 1_000_000_000).unwrap(); // 1000 USDC (6 dec)
    assert_eq!(index_tokens, 1_000_000_000 * 10_000_000);

    assert_eq!(IndexFund::get_user_balance(&env, user), 1_000_000_000 * 10_000_000);
}

#[test]
fn test_second_deposit_mints_proportional() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    env.mock_all_auths();

    // First deposit: 1000 USDC
    let usdc_amount = 1_000_000_000i128; // 1000 USDC
    let fee1 = usdc_amount * 50 / 10_000; // 0.5% fee = 5 USDC
    let net1 = usdc_amount - fee1;
    let index1 = IndexFund::deposit(&env, user1.clone(), usdc_amount).unwrap();

    // NAV after first deposit: net1 * 1e7 / index1
    // Since index1 = net1 * 1e7, NAV = 1e7 (i.e., 1.0)

    // Second deposit: 500 USDC
    let usdc_amount2 = 500_000_000i128; // 500 USDC
    let fee2 = usdc_amount2 * 50 / 10_000;
    let net2 = usdc_amount2 - fee2;
    let index2 = IndexFund::deposit(&env, user2.clone(), usdc_amount2).unwrap();

    // index2 = net2 * index1 / net1
    let expected_index2 = net2 * index1 / net1;
    assert_eq!(index2, expected_index2);
}

#[test]
fn test_withdraw_returns_proportional_usdc() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    let user = Address::generate(&env);
    env.mock_all_auths();

    let usdc_amount = 1_000_000_000i128;
    let index_tokens = IndexFund::deposit(&env, user.clone(), usdc_amount).unwrap();

    // Withdraw half
    let half = index_tokens / 2;
    let usdc_out = IndexFund::withdraw(&env, user.clone(), half).unwrap();

    // Should get approximately half of net USDC back (minus withdraw fee)
    let fee = usdc_amount * 50 / 10_000;
    let net_usdc = usdc_amount - fee;
    let expected_gross = net_usdc / 2;
    let expected_fee = expected_gross * 50 / 10_000;
    let expected_net = expected_gross - expected_fee;
    assert_eq!(usdc_out, expected_net);

    // Remaining balance should be half
    assert_eq!(IndexFund::get_user_balance(&env, user), index_tokens - half);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_deposit_zero() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    let user = Address::generate(&env);
    env.mock_all_auths();
    IndexFund::deposit(&env, user, 0);
}

#[test]
#[should_panic(expected = "InsufficientLiquidity")]
fn test_withdraw_more_than_balance() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    let user = Address::generate(&env);
    env.mock_all_auths();

    let index_tokens = IndexFund::deposit(&env, user.clone(), 1_000_000_000).unwrap();
    IndexFund::withdraw(&env, user, index_tokens + 1);
}

#[test]
fn test_nav_after_first_deposit() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    let user = Address::generate(&env);
    env.mock_all_auths();

    IndexFund::deposit(&env, user, 1_000_000_000);

    // NAV should be 1e7 (1.0 USDC per INDEX token scaled by 1e7)
    let nav = IndexFund::get_nav(&env).unwrap();
    assert_eq!(nav, 10_000_000);
}

#[test]
fn test_performance_after_deposit() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    let user = Address::generate(&env);
    env.mock_all_auths();

    IndexFund::deposit(&env, user.clone(), 1_000_000_000);

    let perf = IndexFund::get_index_performance(&env).unwrap();
    assert_eq!(perf.nav, 10_000_000);
    assert_eq!(perf.total_markets, 0);
    assert!(perf.total_aum > 0);
    assert!(perf.total_index_supply > 0);
}

#[test]
fn test_get_index_composition_empty() {
    let (env, admin, stake_token, factory) = create_test_env();
    setup_fund(&env, &admin, &stake_token, &factory);

    let composition = IndexFund::get_index_composition(&env).unwrap();
    assert_eq!(composition.len(), 0);
}

#[test]
#[should_panic(expected = "FeeTooHigh")]
fn test_initialize_fee_too_high() {
    let (env, admin, stake_token, factory) = create_test_env();
    env.mock_all_auths();
    IndexFund::initialize(
        env.clone(),
        admin,
        stake_token,
        factory,
        3600,
        5000, // 50% fee – way too high
        50,
    );
}
