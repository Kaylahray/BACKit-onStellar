use soroban_sdk::{Address, Env, Symbol};

pub fn emit_proposal_created(env: &Env, id: u64, proposer: &Address, target: &Address, function_name: &Symbol) {
    env.events().publish(
        ("governance", "ProposalCreated"),
        (id, proposer.clone(), target.clone(), function_name.clone()),
    );
}

pub fn emit_vote_cast(env: &Env, proposal_id: u64, voter: &Address, support: bool, power: i128) {
    env.events().publish(
        ("governance", "VoteCast"),
        (proposal_id, voter.clone(), support, power),
    );
}

pub fn emit_proposal_executed(env: &Env, id: u64) {
    env.events().publish(("governance", "ProposalExecuted"), (id,));
}

pub fn emit_delegation(env: &Env, delegator: &Address, delegate: &Address) {
    env.events().publish(
        ("governance", "DelegationChanged"),
        (delegator.clone(), delegate.clone()),
    );
}
