use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("..")
}

fn wasm_candidates(root: &Path) -> Vec<PathBuf> {
    ["wasm32v1-none", "wasm32-unknown-unknown"]
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

fn soroban_compatible_wasm_exists(root: &Path) -> bool {
    let v1 = root
        .join("target/wasm32v1-none/release")
        .join("prediction_market.wasm");
    let optimized = [
        "wasm32v1-none",
        "wasm32-unknown-unknown",
    ]
    .into_iter()
    .map(|target| {
        root.join("target")
            .join(target)
            .join("release")
            .join("prediction_market.optimized.wasm")
    })
    .any(|path| path.exists());

    v1.exists() || optimized
}

fn stellar_available() -> bool {
    Command::new("stellar")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn build_with_stellar(root: &Path) -> bool {
    let status = Command::new("stellar")
        .current_dir(root.join("prediction_market"))
        .args(["contract", "build"])
        .status();

    match status {
        Ok(status) if status.success() => true,
        Ok(status) => {
            eprintln!("stellar contract build failed with status {status}");
            false
        }
        Err(err) => {
            eprintln!("failed to run stellar contract build: {err}");
            false
        }
    }
}

fn build_with_cargo(root: &Path) -> bool {
    let status = Command::new("cargo")
        .current_dir(root)
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
        eprintln!("cargo build for prediction-market failed with status {status}");
        return false;
    }

    let raw_wasm = root
        .join("target/wasm32-unknown-unknown/release")
        .join("prediction_market.wasm");
    if !raw_wasm.exists() {
        return false;
    }

    if stellar_available() {
        let status = Command::new("stellar")
            .args([
                "contract",
                "optimize",
                "--wasm",
                raw_wasm.to_str().expect("non-utf8 wasm path"),
            ])
            .status()
            .expect("failed to spawn stellar contract optimize");

        if status.success() {
            return true;
        }
        eprintln!("stellar contract optimize failed with status {status}");
    }

    false
}

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let root = workspace_root(&manifest_dir);

    println!("cargo:rerun-if-changed=../prediction_market/src");
    println!("cargo:rerun-if-changed=../prediction_market/Cargo.toml");
    println!("cargo:rerun-if-changed=../.cargo/config.toml");

    if soroban_compatible_wasm_exists(&root) {
        return;
    }

    eprintln!("prediction_market WASM not found — building for factory integration tests...");

    let built = if stellar_available() {
        build_with_stellar(&root)
    } else {
        false
    } || build_with_cargo(&root);

    if !built || !soroban_compatible_wasm_exists(&root) {
        panic!(
            "failed to produce Soroban-compatible prediction_market WASM — expected one of:\n  {}",
            wasm_candidates(&root)
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }
}
