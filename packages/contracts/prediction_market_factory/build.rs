use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("..")
}

fn wasm_candidates(root: &Path) -> Vec<PathBuf> {
    ["wasm32-unknown-unknown", "wasm32v1-none"]
        .into_iter()
        .flat_map(|target| {
            let dir = root.join("target").join(target).join("release");
            [
                dir.join("prediction_market.optimized.wasm"),
                dir.join("prediction_market.wasm"),
            ]
        })
        .collect()
}

fn any_wasm_exists(root: &Path) -> bool {
    wasm_candidates(root).iter().any(|path| path.exists())
}

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let root = workspace_root(&manifest_dir);

    println!("cargo:rerun-if-changed=../prediction_market/src");
    println!("cargo:rerun-if-changed=../prediction_market/Cargo.toml");
    println!("cargo:rerun-if-changed=../.cargo/config.toml");

    if any_wasm_exists(&root) {
        return;
    }

    eprintln!("prediction_market WASM not found — building for factory integration tests...");

    let status = Command::new("cargo")
        .current_dir(&root)
        .args([
            "build",
            "--release",
            "-p",
            "prediction-market",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .status()
        .expect("failed to spawn cargo build for prediction-market");

    if !status.success() {
        panic!("cargo build for prediction-market failed with status {status}");
    }

    if !any_wasm_exists(&root) {
        panic!(
            "prediction_market WASM still missing after build — expected one of:\n  {}",
            wasm_candidates(&root)
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }
}
