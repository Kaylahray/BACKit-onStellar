use soroban_sdk::{Address, BytesN, Env, String};

pub fn emit_header_submitted(env: &Env, relayer: &Address, block_number: u64, block_hash: &BytesN<32>) {
    env.events().publish(
        ("cross_chain_oracle_relay", "BlockHeaderSubmitted"),
        (relayer.clone(), block_number, block_hash.clone()),
    );
}

pub fn emit_price_relayed(env: &Env, chain_name: &String, price: i128, block_number: u64) {
    env.events().publish(
        ("cross_chain_oracle_relay", "PriceRelayed"),
        (chain_name.clone(), price, block_number),
    );
}
