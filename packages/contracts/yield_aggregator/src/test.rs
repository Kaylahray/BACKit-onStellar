#![cfg(test)]

extern crate std;

use crate::{YieldAggregator, YieldAggregatorClient};
use soroban_sdk::{
    testutils::Address as AddressTest,
    Address, Env,
};

#[test]
fn initialize_aggregator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(YieldAggregator, (&admin, 1000u32));
    let client = YieldAggregatorClient::new(&env, &contract_id);

    let config = client.get_config_view();
    assert_eq!(config.admin, admin);
    assert_eq!(config.fee_bps, 1000);
}

#[test]
fn deposit_and_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register(YieldAggregator, (&admin, 1000u32));
    let client = YieldAggregatorClient::new(&env, &contract_id);

    let shares = client.deposit(&user, &1000_000000);
    assert_eq!(shares, 1000_000000);

    let position = client.get_user_position(&user);
    assert_eq!(position.shares, 1000_000000);

    let withdrawn = client.withdraw(&user, &500_000000);
    assert_eq!(withdrawn, 500_000000);

    let position = client.get_user_position(&user);
    assert_eq!(position.shares, 500_000000);
}
