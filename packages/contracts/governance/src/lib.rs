#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use errors::GovernanceError;
use events::{emit_delegation, emit_proposal_created, emit_proposal_executed, emit_vote_cast};
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};
use storage::*;
use types::{GovernanceConfig, Proposal, Vote};

const VOTING_POWER_DENOM: i128 = 1_000_000;

#[contract]
pub struct Governance;

#[contractimpl]
impl Governance {
    pub fn initialize(
        env: Env,
        admin: Address,
        proposal_threshold_calls: u32,
        threshold_accuracy_bps: u32,
        voting_period_ledgers: u32,
        quorum_bps: u32,
        pass_threshold_bps: u32,
    ) -> Result<(), GovernanceError> {
        if get_config(&env).is_some() {
            return Err(GovernanceError::AlreadyInitialized);
        }
        admin.require_auth();
        let config = GovernanceConfig {
            admin,
            proposal_threshold_calls,
            threshold_accuracy_bps,
            voting_period_ledgers,
            quorum_bps,
            pass_threshold_bps,
        };
        set_config(&env, &config);
        Ok(())
    }

    pub fn propose_change(
        env: Env,
        proposer: Address,
        target_contract: Address,
        function_name: Symbol,
    ) -> Result<u64, GovernanceError> {
        proposer.require_auth();
        let config = get_config(&env).ok_or(GovernanceError::NotInitialized)?;

        let reputation = Self::get_user_reputation(&env, &proposer);
        let accuracy = reputation.0;
        let _total_staked = reputation.1;

        if accuracy < config.proposal_threshold_calls || accuracy == 0 {
            return Err(GovernanceError::InsufficientReputation);
        }

        let id = next_proposal_id(&env);
        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            target_contract,
            function_name: function_name.clone(),
            for_votes: 0,
            against_votes: 0,
            created_ledger: env.ledger().sequence(),
            executed: false,
        };
        set_proposal(&env, id, &proposal);
        emit_proposal_created(&env, id, &proposer, &proposal.target_contract, &function_name);
        Ok(id)
    }

    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        support: bool,
    ) -> Result<(), GovernanceError> {
        voter.require_auth();
        let config = get_config(&env).ok_or(GovernanceError::NotInitialized)?;
        let mut proposal = get_proposal(&env, proposal_id).ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.executed {
            return Err(GovernanceError::AlreadyExecuted);
        }

        let current_ledger = env.ledger().sequence();
        if current_ledger > proposal.created_ledger + config.voting_period_ledgers {
            return Err(GovernanceError::VotingPeriodEnded);
        }

        let delegate = get_delegate(&env, &voter);
        let effective_voter = delegate.unwrap_or_else(|| voter.clone());

        let mut votes = get_votes(&env, proposal_id);
        for i in 0..votes.len() {
            if votes.get(i).unwrap().voter == effective_voter {
                let mut v = votes.get(i).unwrap();
                v.support = support;
                let power = v.power;
                votes.set(i, v);
                set_votes(&env, proposal_id, &votes);
                emit_vote_cast(&env, proposal_id, &effective_voter, support, power);
                return Ok(());
            }
        }

        let reputation = Self::get_user_reputation(&env, &effective_voter);
        let accuracy_bps = reputation.0;
        let total_staked = reputation.1;

        let power = (accuracy_bps as i128 * VOTING_POWER_DENOM) + total_staked / 1_000_000;
        let vote = Vote {
            voter: effective_voter.clone(),
            support,
            power,
        };
        votes.push_back(vote);
        set_votes(&env, proposal_id, &votes);

        if support {
            proposal.for_votes = proposal.for_votes.checked_add(power).ok_or(GovernanceError::ProposalInactive)?;
        } else {
            proposal.against_votes = proposal.against_votes.checked_add(power).ok_or(GovernanceError::ProposalInactive)?;
        }
        set_proposal(&env, proposal_id, &proposal);

        emit_vote_cast(&env, proposal_id, &effective_voter, support, power);
        Ok(())
    }

    pub fn execute_proposal(env: Env, caller: Address, proposal_id: u64) -> Result<(), GovernanceError> {
        caller.require_auth();
        let config = get_config(&env).ok_or(GovernanceError::NotInitialized)?;
        let mut proposal = get_proposal(&env, proposal_id).ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.executed {
            return Err(GovernanceError::AlreadyExecuted);
        }

        let current_ledger = env.ledger().sequence();
        if current_ledger <= proposal.created_ledger + config.voting_period_ledgers {
            return Err(GovernanceError::ProposalInactive);
        }

        let total_power = proposal.for_votes + proposal.against_votes;
        let quorum = (get_possible_voting_power(&env) * config.quorum_bps as i128) / 10000;
        if total_power < quorum {
            return Err(GovernanceError::QuorumNotMet);
        }

        let pass_threshold = (total_power * config.pass_threshold_bps as i128) / 10000;
        if proposal.for_votes <= pass_threshold {
            return Err(GovernanceError::ProposalNotPassed);
        }

        proposal.executed = true;
        set_proposal(&env, proposal_id, &proposal);

        env.invoke_contract::<()>(
            &proposal.target_contract,
            &proposal.function_name,
            soroban_sdk::vec![&env],
        );

        emit_proposal_executed(&env, proposal_id);
        Ok(())
    }

    pub fn delegate_votes(
        env: Env,
        delegator: Address,
        delegate: Address,
    ) -> Result<(), GovernanceError> {
        delegator.require_auth();
        if delegator == delegate {
            return Err(GovernanceError::SelfDelegation);
        }
        set_delegate(&env, &delegator, &delegate);
        emit_delegation(&env, &delegator, &delegate);
        Ok(())
    }

    pub fn get_proposal_view(env: Env, proposal_id: u64) -> Result<Proposal, GovernanceError> {
        get_proposal(&env, proposal_id).ok_or(GovernanceError::ProposalNotFound)
    }

    pub fn get_active_proposals(env: Env) -> Vec<Proposal> {
        let config = match get_config(&env) {
            Some(c) => c,
            None => return Vec::new(&env),
        };
        let ids = get_proposal_ids(&env);
        let current = env.ledger().sequence();
        let mut result = Vec::new(&env);
        for i in 0..ids.len() {
            let id = ids.get(i).unwrap();
            if let Some(p) = get_proposal(&env, id) {
                if !p.executed && current <= p.created_ledger + config.voting_period_ledgers {
                    result.push_back(p);
                }
            }
        }
        result
    }

    pub fn get_voting_power(env: Env, user: Address) -> i128 {
        let rep = Self::get_user_reputation(&env, &user);
        (rep.0 as i128 * VOTING_POWER_DENOM) + rep.1 / 1_000_000
    }

    pub fn get_delegate(env: Env, address: Address) -> Option<Address> {
        storage::get_delegate(&env, &address)
    }

    pub fn set_reputation(
        env: Env,
        admin: Address,
        user: Address,
        resolved_calls: u32,
        total_staked: i128,
    ) -> Result<(), GovernanceError> {
        admin.require_auth();
        let config = get_config(&env).ok_or(GovernanceError::NotInitialized)?;
        if admin != config.admin {
            return Err(GovernanceError::Unauthorized);
        }
        set_user_reputation(&env, &user, resolved_calls, total_staked);
        Ok(())
    }

    fn get_user_reputation(env: &Env, user: &Address) -> (u32, i128) {
        storage::get_user_reputation(env, user)
    }
}

fn get_possible_voting_power(env: &Env) -> i128 {
    100_000_000_000
}
