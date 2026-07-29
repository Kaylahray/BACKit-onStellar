use crate::types::{AuctionInfo, DutchAuctionConfig};
use soroban_sdk::{contracttype, Env};

#[contracttype]
pub enum InstanceKey {
    Config,
}

#[contracttype]
pub enum PersistentKey {
    AuctionInfo(u64),
}

pub fn set_config(env: &Env, config: &DutchAuctionConfig) {
    env.storage().instance().set(&InstanceKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<DutchAuctionConfig> {
    env.storage().instance().get(&InstanceKey::Config)
}

pub fn set_auction_info(env: &Env, call_id: u64, info: &AuctionInfo) {
    env.storage()
        .persistent()
        .set(&PersistentKey::AuctionInfo(call_id), info);
}

pub fn get_auction_info(env: &Env, call_id: u64) -> Option<AuctionInfo> {
    env.storage()
        .persistent()
        .get(&PersistentKey::AuctionInfo(call_id))
}
