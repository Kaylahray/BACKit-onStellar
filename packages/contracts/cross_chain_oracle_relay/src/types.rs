use soroban_sdk::{contracttype, Address, BytesN, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct BlockHeader {
    pub parent_hash: BytesN<32>,
    pub state_root: BytesN<32>,
    pub transactions_root: BytesN<32>,
    pub receipts_root: BytesN<32>,
    pub block_number: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct OracleRelayConfig {
    pub admin: Address,
    pub outcome_manager: Address,
    pub stellar_oracle: Address,
    pub min_signatures: u32,
    pub fee_per_submission: i128,
    pub fee_token: Address,
    pub trusted_relayers: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CrossChainSource {
    pub chain_name: soroban_sdk::String,
    pub latest_block: u64,
    pub last_relayed_at: u64,
    pub last_price: i128,
}
