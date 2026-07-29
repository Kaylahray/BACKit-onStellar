use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum StrategyTrigger {
    MarketResolved(u64, u32),
    PriceThreshold(Address, i128, bool),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum StrategyAction {
    StakeOnCall(u64, u32, i128),
    ClaimPayout(u64),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalStrategy {
    pub id: u64,
    pub user: Address,
    pub trigger: StrategyTrigger,
    pub actions: soroban_sdk::Vec<StrategyAction>,
    pub escrow_amount: i128,
    pub executed: bool,
    pub cancelled: bool,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}
