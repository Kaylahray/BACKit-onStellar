use soroban_sdk::{Address, Env, Symbol};

/// Default maximum call duration: 30 days in seconds.
pub const DEFAULT_MAX_DURATION_SECS: u64 = 2_592_000;

const MAX_DURATION_KEY: &str = "max_duration";

/// Set the maximum allowed call duration (admin only). Emits AdminParamsChanged.
pub fn set_max_duration(env: &Env, admin: Address, max_duration_secs: u64) {
    admin.require_auth();
    assert!(max_duration_secs > 0, "max_duration_secs must be positive");
    let old = get_max_duration(env);
    env.storage()
        .instance()
        .set(&Symbol::new(env, MAX_DURATION_KEY), &max_duration_secs);
    crate::events::emit_admin_params_changed_u64(
        env,
        "max_duration_secs",
        &admin,
        old,
        max_duration_secs,
    );
}

/// Read the configured max duration, falling back to the default.
pub fn get_max_duration(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get::<_, u64>(&Symbol::new(env, MAX_DURATION_KEY))
        .unwrap_or(DEFAULT_MAX_DURATION_SECS)
}

/// Assert that `end_ts - now <= max_duration_secs`.
/// Call this at the top of `create_call` before persisting the call.
pub fn assert_duration_within_limit(env: &Env, end_ts: u64) {
    let now = env.ledger().timestamp();
    assert!(end_ts > now, "end_ts must be in the future");
    let duration = end_ts - now;
    let max = get_max_duration(env);
    assert!(duration <= max, "call duration exceeds maximum allowed");
}
