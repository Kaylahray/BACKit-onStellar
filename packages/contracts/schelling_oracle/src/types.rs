use soroban_sdk::{contracttype, Address, BytesN};

/// Admin-configurable parameters governing every future dispute.
///
/// `bond_bps` is expressed in basis points (1/100th of a percent) of the
/// caller-supplied `total_pool_amount` — e.g. `500` == 5%, matching the
/// acceptance criteria's "dispute_bond = total_pool * 5%" example.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DisputeConfig {
    pub admin: Address,
    /// Duration (seconds) of the commit phase, starting the moment
    /// `dispute_outcome` is called.
    pub voting_period_secs: u64,
    /// Duration (seconds) of the reveal phase, starting the moment the
    /// commit phase ends.
    pub reveal_period_secs: u64,
    /// Minimum bond, in basis points of `total_pool_amount`.
    pub bond_bps: u32,
}

/// Outcome of a dispute, including its not-yet-resolved state.
///
/// Modeled as a single enum (rather than a separate `resolved: bool` plus
/// `Option<DisputeResult>`) because this SDK version's `#[contracttype]`
/// codegen for unit-only enums only provides a fallible `TryInto<ScVal>`,
/// and the blanket `Option<T>: TryInto<ScVal>` impl in `stellar-xdr`
/// requires an infallible `T: Into<ScVal>` — so wrapping a custom
/// `#[contracttype]` enum in `Option<_>` fails to compile. Folding the
/// "not yet resolved" state into the enum itself (`Pending`) sidesteps that
/// entirely.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisputeResult {
    /// Reveal phase hasn't ended yet (or has, but `resolve_dispute` hasn't
    /// been called).
    Pending,
    /// The revealed majority sided with the disputer's claimed outcome.
    DisputerWon,
    /// The revealed majority sided with the oracle's original outcome (or
    /// tied / nobody sided with the disputer).
    DisputerLost,
    /// Nobody ever committed a vote. The disputer's bond is refunded and no
    /// side is rewarded or slashed. This is a defensive fallback, not part
    /// of the core game — see the doc comment on `resolve_dispute`.
    Void,
}

/// A single dispute opened against an oracle-reported outcome for `call_id`.
///
/// **Scope decision:** this contract does not read `call_id`'s real pool or
/// outcome cross-contract (see the crate-level doc comment in `lib.rs`).
/// `original_outcome`, `total_pool_amount` and `stake_token` are all
/// caller-supplied and trusted, mirroring `outcome_manager::submit_outcome`'s
/// caller-supplied `call_end_ts` pattern elsewhere in this codebase.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Dispute {
    pub id: u64,
    pub call_id: u64,
    pub disputer: Address,
    /// The oracle's original (pre-dispute) outcome for this call.
    pub original_outcome: u32,
    /// The outcome the disputer claims is correct instead.
    pub disputed_outcome: u32,
    pub stake_token: Address,
    /// Bond escrowed by the disputer; locked until `resolve_dispute`.
    pub bond_amount: i128,
    /// Caller-supplied snapshot of the market's total pool, used only to
    /// derive the minimum required bond at open time.
    pub total_pool_amount: i128,
    /// Commit phase ends (exclusive) at this ledger timestamp.
    pub commit_deadline: u64,
    /// Reveal phase ends (exclusive) at this ledger timestamp.
    pub reveal_deadline: u64,
    /// `DisputeResult::Pending` until `resolve_dispute` is called.
    pub result: DisputeResult,
}

/// One voter's commit-reveal state for a single dispute.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VoteCommitment {
    pub commitment_hash: BytesN<32>,
    pub stake_amount: i128,
    pub revealed: bool,
    /// `0` until revealed (0 is never a valid outcome id — outcomes start
    /// at 1 throughout this codebase, mirroring `prediction_market`).
    pub revealed_outcome: u32,
}
