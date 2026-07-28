#![cfg(test)]

extern crate std;

use crate::{ReputationNft, ReputationNftClient};
use soroban_sdk::{
    testutils::Address as AddressTest,
    symbol_short, Address, Env,
};

#[test]
fn initialize_reputation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(ReputationNft, (&admin,));
    let client = ReputationNftClient::new(&env, &contract_id);

    let config = client.get_config_view();
    assert_eq!(config.admin, admin);
}

#[test]
fn award_badge_and_prevent_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register(ReputationNft, (&admin,));
    let client = ReputationNftClient::new(&env, &contract_id);

    let token_id = client.award_badge(
        &user,
        &symbol_short!("EARLY"),
        &soroban_sdk::String::from_str(&env, "ipfs://early"),
    );

    assert_eq!(token_id, 1);

    let badges = client.get_user_badges(&user);
    assert_eq!(badges.len(), 1);

    let result = client.try_transfer(&user, &Address::generate(&env), &token_id);
    assert!(result.is_err());
}
