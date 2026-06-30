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

    let expected_paths = [
        "target/wasm32v1-none/release/prediction_market.wasm",
        "target/wasm32-unknown-unknown/release/prediction_market.wasm",
    ];

    panic!(
        "prediction_market WASM not found.\n\
         The factory contract requires prediction_market.wasm to be pre-built.\n\
         Run this before building the workspace:\n  \
           cd prediction_market && stellar contract build\n\n\
         Expected one of:\n  {}",
        expected_paths.join("\n  ")
    );
}
