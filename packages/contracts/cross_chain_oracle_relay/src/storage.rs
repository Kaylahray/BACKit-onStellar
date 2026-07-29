use crate::types::{CrossChainSource, OracleRelayConfig};
use soroban_sdk::{contracttype, BytesN, Env, Map, String};

#[contracttype]
pub enum DataKey {
    Config,
    TrustedHashes,
    RelayedBlocks(u64),
    CrossChainSources,
    Source(String),
}

pub fn set_config(env: &Env, config: &OracleRelayConfig) {
    env.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(env: &Env) -> Option<OracleRelayConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn add_trusted_hash(env: &Env, block_number: u64, hash: &BytesN<32>) {
    let mut hashes: Map<u64, BytesN<32>> = env
        .storage()
        .instance()
        .get(&DataKey::TrustedHashes)
        .unwrap_or_else(|| Map::new(env));
    hashes.set(block_number, hash.clone());
    env.storage().instance().set(&DataKey::TrustedHashes, &hashes);
}

pub fn get_trusted_hash(env: &Env, block_number: u64) -> Option<BytesN<32>> {
    let hashes: Map<u64, BytesN<32>> = env
        .storage()
        .instance()
        .get(&DataKey::TrustedHashes)
        .unwrap_or_else(|| Map::new(env));
    hashes.get(block_number)
}

pub fn mark_block_relayed(env: &Env, block_number: u64) {
    env.storage().instance().set(&DataKey::RelayedBlocks(block_number), &true);
}

pub fn is_block_relayed(env: &Env, block_number: u64) -> bool {
    env.storage().instance().get::<_, bool>(&DataKey::RelayedBlocks(block_number)).unwrap_or(false)
}

pub fn set_source(env: &Env, chain_name: &String, source: &CrossChainSource) {
    env.storage().instance().set(&DataKey::Source(chain_name.clone()), source);
}

pub fn get_source(env: &Env, chain_name: &String) -> Option<CrossChainSource> {
    env.storage().instance().get(&DataKey::Source(chain_name.clone()))
}
