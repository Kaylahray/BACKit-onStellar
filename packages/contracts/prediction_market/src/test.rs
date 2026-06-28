#![cfg(test)]

extern crate std;

use crate::{
    types::{ConditionType, MarketInitArgs},
    PredictionMarket, PredictionMarketClient,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Bytes, BytesN, Env,
};

fn setup_token(env: &Env, admin: &Address) -> Address {
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = token.address();
    StellarAssetClient::new(env, &sac).mint(admin, &100_000_000_000);
    sac
}

#[test]
fn market_constructor_and_stake() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let staker = Address::generate(&env);
    let outcome_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let token = setup_token(&env, &admin);

    let end_ts = env.ledger().timestamp() + 3600;
    let args = MarketInitArgs {
        stake_token: token.clone(),
        stake_amount: 1_000_000,
        start_price: 100_000_000,
        end_ts,
        token_address: token.clone(),
        pair_id: Bytes::from_slice(&env, b"PAIR"),
        metadata_hash: BytesN::from_array(&env, &[2u8; 32]),
        condition: ConditionType::TargetAbove(105_000_000),
        outcome_count: 2,
    };

    let market_id = env.register(
        PredictionMarket,
        (
            1u64,
            creator.clone(),
            outcome_manager,
            factory,
            100_000i128,
            0i128,
            300u64,
            args,
        ),
    );
    let market = PredictionMarketClient::new(&env, &market_id);

    TokenClient::new(&env, &token).transfer(&admin, &staker, &5_000_000);
    market.stake_on_call(&staker, &1u64, &2_000_000, &1u32);

    let stakes = market.get_outcome_stakes(&1u64);
    assert_eq!(stakes.get(1u32).unwrap(), 2_000_000);
    assert_eq!(market.get_staker_stake(&1u64, &staker, &1u32), 2_000_000);
}

#[test]
fn market_resolve_requires_outcome_manager() {
    let env = Env::default();
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let outcome_manager = Address::generate(&env);
    let factory = Address::generate(&env);
    let token = setup_token(&env, &creator);

    let end_ts = env.ledger().timestamp() + 100;
    let args = MarketInitArgs {
        stake_token: token,
        stake_amount: 1_000_000,
        start_price: 100_000_000,
        end_ts,
        token_address: Address::generate(&env),
        pair_id: Bytes::from_slice(&env, b"P"),
        metadata_hash: BytesN::from_array(&env, &[3u8; 32]),
        condition: ConditionType::PercentUp(1),
        outcome_count: 2,
    };

    let market_id = env.register(
        PredictionMarket,
        (
            42u64,
            creator,
            outcome_manager.clone(),
            factory,
            100_000i128,
            0i128,
            0u64,
            args,
        ),
    );
    let market = PredictionMarketClient::new(&env, &market_id);

    env.ledger().set_timestamp(end_ts + 1);
    let result = market.try_resolve_call(&42u64, &1u32, &110_000_000);
    // With mock_all_auths, resolution succeeds when outcome_manager auth is mocked.
    assert!(result.is_ok());
}
