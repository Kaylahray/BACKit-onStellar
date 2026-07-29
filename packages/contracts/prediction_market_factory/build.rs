use std::path::{Path, PathBuf};

fn workspace_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("..")
}

fn soroban_compatible_wasm_exists(root: &Path) -> bool {
    let candidates = [
        root.join("target/wasm32v1-none/release/prediction_market.optimized.wasm"),
        root.join("target/wasm32v1-none/release/prediction_market.wasm"),
        root.join("target/wasm32-unknown-unknown/release/prediction_market.optimized.wasm"),
        root.join("target/wasm32-unknown-unknown/release/prediction_market.wasm"),
    ];
    candidates.iter().any(|p| p.exists())
}

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let root = workspace_root(&manifest_dir);

    println!("cargo:rerun-if-changed=../prediction_market/src");
    println!("cargo:rerun-if-changed=../prediction_market/Cargo.toml");

    if soroban_compatible_wasm_exists(&root) {
        return;
    }

    println!("cargo:warning=prediction_market WASM not found.");
}
