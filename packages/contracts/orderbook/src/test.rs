#![cfg(test)]

extern crate std;

use crate::{Orderbook, OrderbookClient};
use soroban_sdk::{
    testutils::Address as AddressTest,
    Address, Env,
};

#[test]
fn initialize_orderbook() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(Orderbook, (&admin,));
    let client = OrderbookClient::new(&env, &contract_id);

    let config = client.get_config_view();
    assert_eq!(config.admin, admin);
    assert_eq!(config.fee_bps, 30);
}
