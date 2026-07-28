use soroban_sdk::{contracttype, Address};

/// Pool-wide configuration, set once at `initialize` and tunable afterwards
/// via admin-only setters.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct PoolConfig {
    pub admin: Address,
    /// Recipient of the protocol's cut of realized profits.
    pub treasury: Address,
    /// The USDC-equivalent SAC (or native XLM sentinel) this pool denominates
    /// deposits, allocations and yield in.
    pub stake_token: Address,
    /// Factory used to resolve a `call_id` to its deployed market address.
    pub prediction_market_factory: Address,
    /// `outcome_manager` instance used to claim settled winnings.
    pub outcome_manager: Address,
    /// Smallest single deposit accepted.
    pub min_deposit: i128,
    /// Hard cap on total value locked; deposits that would push the pool
    /// above this are rejected, bounding concentration risk.
    pub max_pool_size: i128,
    /// Minimum total value locked required before `allocate_capital` will
    /// stake anything (avoids dust-sized, gas-inefficient allocations).
    pub min_allocation_pool_size: i128,
    /// Per-market allocation cap, in basis points of TVL. Default 500 (5%).
    pub max_allocation_bps_per_market: u32,
    /// Protocol's cut of realized profit on a harvested market, in basis
    /// points. Default 1000 (10%).
    pub protocol_fee_bps: u32,
    /// Minimum `edge` (see `allocate_capital`) required before the pool
    /// bothers allocating to a market, in basis points. Default 100 (1%).
    pub edge_threshold_bps: u32,
}

/// A single open (unsettled) stake the pool currently holds in a
/// `prediction_market` instance, staked as the pool contract itself.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Allocation {
    pub call_id: u64,
    pub market_address: Address,
    /// The outcome position (1 or 2 — this pool only allocates to binary
    /// markets, see `allocate_capital`'s doc comment) the pool staked on.
    pub position: u32,
    /// Principal staked, in `stake_token` base units.
    pub amount: i128,
    /// `true` once `harvest_yield` has processed this call's resolution.
    pub settled: bool,
    /// `true` if the pool's staked position matched the resolved outcome.
    pub won: bool,
    /// Gross tokens recovered from the claim (0 if the pool lost the stake).
    pub payout: i128,
    pub created_at: u64,
}

/// One realized profit-or-loss data point, recorded every time
/// `harvest_yield` settles an allocation. Used to compute a rolling 7-day
/// APY estimate; entries older than the window are pruned opportunistically.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct YieldEvent {
    pub timestamp: u64,
    /// `payout - amount_staked`, net of the protocol fee already deducted to
    /// the treasury (positive on a win, `-amount_staked` on a total loss).
    pub net_yield: i128,
}

/// Read-only performance snapshot, see [`crate::LendingPool::get_pool_stats`].
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct PoolStats {
    /// Lifetime sum of all `deposit` amounts (never decreases).
    pub total_deposited: i128,
    /// Current total pool value: liquid balance + capital locked in open
    /// allocations. This is the number LP share price is derived from.
    pub total_value_locked: i128,
    /// Lifetime net-of-protocol-fee yield credited to the pool (can be
    /// negative if losses outweigh wins).
    pub total_yield_earned: i128,
    /// Annualized rolling 7-day yield, in basis points (can be negative).
    pub current_apy_bps: i128,
    pub total_lp_shares: i128,
    /// Token balance currently held directly by the pool contract.
    pub liquid_balance: i128,
    /// Principal currently staked in unsettled markets.
    pub total_allocated_locked: i128,
    pub open_market_count: u32,
    pub min_deposit: i128,
    pub max_pool_size: i128,
    pub protocol_fee_bps: u32,
    pub max_allocation_bps_per_market: u32,
}

/// Caller-supplied per-market input to [`crate::LendingPool::allocate_capital`].
///
/// `oracle_probability_bps` is the caller's estimate of the probability
/// (0..=10_000) that outcome position `1` wins. See the doc comment on
/// `allocate_capital` for why this is a trusted caller-supplied value rather
/// than a live oracle feed.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct MarketAllocationInput {
    pub call_id: u64,
    pub oracle_probability_bps: u32,
}

/// Outcome of considering a single market during `allocate_capital`.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum AllocationOutcome {
    /// The pool staked `amount` on `position`.
    Allocated(i128, u32),
    /// `edge` (see `allocate_capital`) was below `edge_threshold_bps`.
    SkippedEdgeTooSmall,
    /// The pool already has an open (unsettled) allocation for this `call_id`.
    SkippedAlreadyOpen,
    /// The market has more than 2 outcomes; this pool's Kelly-edge math only
    /// supports binary (yes/no) markets.
    SkippedNotBinary,
    /// The market is closed to new stakes, already resolved, or otherwise
    /// unavailable (lookup failed, settled/cancelled/voided).
    SkippedMarketUnavailable,
    /// The market's `stake_token` doesn't match this pool's `stake_token`.
    SkippedTokenMismatch,
    /// `oracle_probability_bps` was out of the valid `0..=10_000` range.
    SkippedInvalidOracleProbability,
    /// The computed allocation amount was zero or the pool ran out of
    /// liquid capital partway through a multi-market batch.
    SkippedInsufficientLiquidity,
    /// `prediction_market::stake_on_call` itself rejected the stake (e.g.
    /// below the market's own minimum, or the staking cutoff window is
    /// active).
    SkippedStakeRejected,
}
