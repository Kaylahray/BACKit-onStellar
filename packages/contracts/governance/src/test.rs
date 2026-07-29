#![cfg(test)]

extern crate std;

use crate::{Governance, GovernanceClient};
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, Symbol,
};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let contract_id = env.register(Governance, ());
    let client = GovernanceClient::new(&env, &contract_id);
    client.initialize(&admin, &10, &5000, &1000, &1000, &5000);

    let result = client.try_initialize(&admin, &10, &5000, &1000, &1000, &5000);
    assert!(result.is_err());
}

#[test]
fn test_propose_and_vote() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let target = Address::generate(&env);

    let contract_id = env.register(Governance, ());
    let client = GovernanceClient::new(&env, &contract_id);
    client.initialize(&admin, &1, &1000, &1000, &1000, &5000);

    client.set_reputation(&admin, &proposer, &5, &1_000_000_000);
    client.set_reputation(&admin, &voter, &3, &500_000_000);

    let pid = client.propose_change(&proposer, &target, &Symbol::new(&env, "set_param"));
    assert_eq!(pid, 1);

    client.vote(&voter, &pid, &true);

    let proposal = client.get_proposal_view(&1);
    assert!(proposal.for_votes > 0);
}
