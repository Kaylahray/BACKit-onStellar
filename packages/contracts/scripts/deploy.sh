#!/usr/bin/env bash
set -euo pipefail

NETWORK=${1:---network testnet}
ENV_FILE="$(dirname "$0")/../.env.contracts"

# Required environment variables (export these before running this script):
#   STELLAR_SOURCE_ACCOUNT  - funded source account used to sign all deploy/invoke txs
#   ADMIN_ADDRESS           - admin address for outcome_manager / call_registry / factory
#   FEE_COLLECTOR_ADDRESS   - fee collector address for outcome_manager
#   ORACLE_PUBKEYS          - comma-separated BytesN<32> hex oracle pubkeys, e.g. "aaaa...,bbbb..."
# Optional (defaults shown):
#   QUORUM=1
#   FEE_BPS=0
#   DISPUTE_WINDOW_SECS=3600
#   MIN_STAKE=10000000

: "${STELLAR_SOURCE_ACCOUNT:?Set STELLAR_SOURCE_ACCOUNT to a funded source account}"
: "${ADMIN_ADDRESS:?Set ADMIN_ADDRESS to the admin address}"
: "${FEE_COLLECTOR_ADDRESS:?Set FEE_COLLECTOR_ADDRESS to the outcome_manager fee collector address}"
: "${ORACLE_PUBKEYS:?Set ORACLE_PUBKEYS to a comma-separated list of BytesN<32> hex oracle pubkeys}"
QUORUM=${QUORUM:-1}
FEE_BPS=${FEE_BPS:-0}
DISPUTE_WINDOW_SECS=${DISPUTE_WINDOW_SECS:-3600}
MIN_STAKE=${MIN_STAKE:-10000000}

echo "Deploying contracts on ${NETWORK}..."

# Build a JSON array (`["aaaa...","bbbb..."]`) from the comma-separated pubkey list
# for the outcome_manager `oracles: Vec<BytesN<32>>` constructor argument.
IFS=',' read -ra ORACLE_ARR <<< "$ORACLE_PUBKEYS"
ORACLES_JSON="["
for i in "${!ORACLE_ARR[@]}"; do
  [[ $i -gt 0 ]] && ORACLES_JSON+=","
  ORACLES_JSON+="\"${ORACLE_ARR[$i]}\""
done
ORACLES_JSON+="]"

# 1. Deploy call_registry
CALL_REGISTRY_ID=$(stellar contract deploy --wasm target/wasm32-unknown-unknown/release/call_registry.wasm $NETWORK --source "$STELLAR_SOURCE_ACCOUNT")

# 2. Deploy outcome_manager (renamed from result_oracle)
OUTCOME_MANAGER_ID=$(stellar contract deploy --wasm target/wasm32-unknown-unknown/release/outcome_manager.wasm $NETWORK --source "$STELLAR_SOURCE_ACCOUNT")

stellar contract invoke --id "$CALL_REGISTRY_ID" $NETWORK -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --outcome_manager "$OUTCOME_MANAGER_ID" \
  --min_stake "$MIN_STAKE"

stellar contract invoke --id "$OUTCOME_MANAGER_ID" $NETWORK -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --oracles "$ORACLES_JSON" \
  --quorum "$QUORUM" \
  --fee_collector "$FEE_COLLECTOR_ADDRESS" \
  --fee_bps "$FEE_BPS" \
  --dispute_window_secs "$DISPUTE_WINDOW_SECS"

# 3. Upload (do NOT deploy) prediction_market.wasm. The market contract's
# `__constructor` requires per-market args (call_id, creator, min_stake, ...) that
# only the factory knows at `deploy_market` time, so this workspace's pattern
# (see prediction_market_factory/build.rs) is: upload the wasm once to install it
# on the network and obtain its hash, then hand that hash to the factory, which
# instantiates real market instances itself via `env.deployer().deploy_v2(...)`.
MARKET_WASM_HASH=$(stellar contract upload --wasm target/wasm32-unknown-unknown/release/prediction_market.wasm $NETWORK --source "$STELLAR_SOURCE_ACCOUNT")

# 4. Deploy prediction_market_factory (after prediction_market is uploaded, since
# its constructor call below needs the market wasm hash) and initialize it.
PREDICTION_MARKET_FACTORY_ID=$(stellar contract deploy --wasm target/wasm32-unknown-unknown/release/prediction_market_factory.wasm $NETWORK --source "$STELLAR_SOURCE_ACCOUNT")

stellar contract invoke --id "$PREDICTION_MARKET_FACTORY_ID" $NETWORK -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --outcome_manager "$OUTCOME_MANAGER_ID" \
  --market_wasm_hash "$MARKET_WASM_HASH" \
  --min_stake "$MIN_STAKE"

# Health checks: confirm each contract is live and was initialized correctly.
echo "Verifying deployments..."

stellar contract invoke --id "$CALL_REGISTRY_ID" $NETWORK -- get_config >/dev/null
echo "  call_registry:               OK (get_config)"

# outcome_manager has no get_config view; get_quorum reads back the quorum set
# during initialize and panics if the contract was never initialized, so it
# serves as the equivalent health check.
stellar contract invoke --id "$OUTCOME_MANAGER_ID" $NETWORK -- get_quorum >/dev/null
echo "  outcome_manager:              OK (get_quorum)"

stellar contract invoke --id "$PREDICTION_MARKET_FACTORY_ID" $NETWORK -- get_config >/dev/null
echo "  prediction_market_factory:    OK (get_config)"

printf 'CALL_REGISTRY_ID=%s\nOUTCOME_MANAGER_CONTRACT_ADDRESS=%s\nPREDICTION_MARKET_WASM_HASH=%s\nPREDICTION_MARKET_FACTORY_ID=%s\n' \
  "$CALL_REGISTRY_ID" "$OUTCOME_MANAGER_ID" "$MARKET_WASM_HASH" "$PREDICTION_MARKET_FACTORY_ID" > "$ENV_FILE"

echo "Deployed. IDs saved to $ENV_FILE"
