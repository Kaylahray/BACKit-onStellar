use soroban_sdk::contracttype;

/// A user's active gas-sponsorship agreement with the gas station.
///
/// Registered via [`crate::GasStation::sponsor_transaction`]. `winning_cut_bps`
/// is read back at payout time by [`crate::GasStation::claim_sponsored_payout`]
/// to compute the gas station's cut.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct SponsorshipInfo {
    /// Upper bound on the gas cost (in stroops of the configured XLM token)
    /// the gas station has committed to cover for this user.
    pub max_gas_xlm: i128,
    /// Basis points of the sponsored payout the gas station keeps if the
    /// user wins (e.g. `300` = 3%).
    pub winning_cut_bps: u32,
    /// Ledger timestamp at which this sponsorship was registered.
    pub sponsored_at: u64,
    /// Whether this sponsorship is currently active (admin-revocable).
    pub active: bool,
}

/// Running totals for the gas station's sponsorship program.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct GasStationMetrics {
    /// Count of `sponsor_transaction` registrations.
    pub total_transactions_sponsored: u64,
    /// Cumulative estimated gas (in stroops) committed across all
    /// sponsorships, whether or not the sponsored user ultimately won.
    pub total_xlm_spent: i128,
    /// Cumulative winning cuts collected via `claim_sponsored_payout`.
    pub total_winnings_collected: i128,
    /// `total_winnings_collected - total_xlm_spent`. Can be negative when
    /// losses (gas fronted for users who didn't win) outweigh cuts earned.
    pub net_profit_loss: i128,
}

impl GasStationMetrics {
    pub fn zero() -> Self {
        GasStationMetrics {
            total_transactions_sponsored: 0,
            total_xlm_spent: 0,
            total_winnings_collected: 0,
            net_profit_loss: 0,
        }
    }
}
