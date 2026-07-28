//! Reputation-weighted individual staking limits.
//!
//! Historically `stake_on_call` enforced a single flat, admin-set
//! `max_stake_per_user` cap for every user. This module replaces that with a
//! *personal* limit derived from each user's on-chain prediction track record
//! (`CreatorStats`) and their historical stake volume relative to the rest of
//! the platform: proven, accurate predictors earn a higher personal limit,
//! while brand-new accounts are held to a conservative baseline. This both
//! curbs spam from disposable accounts and rewards demonstrated skill.
//!
//! # Fixed-point conventions
//!
//! Soroban/WASM contracts must never use floating-point types (`f32`/`f64`):
//! floating-point arithmetic is not guaranteed to be bit-identical across
//! host implementations, which would break consensus. Every ratio in this
//! module is therefore expressed as an integer number of **basis points**
//! (bps), where `10_000` represents `1.0` -- the same convention already used
//! elsewhere in this crate for `fee_bps`:
//!
//! * `accuracy_bps`     -- `total_correct / max(total_resolved, 1)`, scaled to bps
//!   (range `0..=10_000`).
//! * `reputation_multiplier` (config field, `u32`) -- bps scale; `10_000` means
//!   a fully accurate (100%) user's accuracy term contributes a full
//!   *additional* `1.0x` on top of the baseline `1.0x` (see formula below).
//! * `volume_factor_bps` -- `user_volume / platform_average_volume`, scaled to
//!   bps, capped at [`MAX_VOLUME_FACTOR_BPS`] (`20_000` == `2.0x`).
//!
//! # Formula
//!
//! For a user whose `CreatorStats` show `total_resolved >=
//! `[`NEW_USER_RESOLVED_THRESHOLD`]` (10):
//!
//! ```text
//! accuracy_bps   = total_correct * 10_000 / max(total_resolved, 1)
//! factor_bps     = 10_000 + (accuracy_bps * reputation_multiplier / 10_000)
//! volume_factor  = min(user_volume * 10_000 / platform_average_volume, 20_000)
//! reputation_limit = base_stake_limit * factor_bps / 10_000 * volume_factor / 10_000
//! ```
//!
//! Users below the threshold (new accounts) skip the formula entirely and
//! get exactly `base_stake_limit` -- neither their (necessarily low-sample)
//! accuracy nor any stake volume they've accumulated can raise their limit
//! until they have a real resolved track record. This is a deliberate hard
//! gate: the acceptance criteria calls for new users to be "restricted to
//! *only* base_stake_limit", so we do not let the general formula's edge
//! cases (e.g. a new user with 0 resolved calls trivially has `accuracy_bps
//! == 0`, but could still have accumulated large stake `volume_factor`)
//! sneak in a higher limit.
//!
//! `platform_average_volume` approximates the acceptance criteria's
//! "platform median volume" as `GlobalStats.total_stake_volume /
//! GlobalStats.total_unique_stakers`. Note this is a *mean*, not a true
//! median -- the contract does not track a sorted distribution of
//! per-user volumes on-chain (that would be prohibitively expensive in
//! storage/compute), and `GlobalStats` (see `types.rs`) only exposes the
//! aggregate volume and unique-staker count, matching exactly what the
//! issue's technical pointers described. This is called out explicitly here
//! and in the implementation report.
//!
//! # Interaction with `max_stake_per_user`
//!
//! `max_stake_per_user` remains a separate, admin-set *absolute ceiling*.
//! When configured (non-zero), it is combined with the computed reputation
//! limit as `effective_limit = min(reputation_limit, max_stake_per_user)`.
//! In other words, reputation can only ever narrow a user's limit to *below*
//! the admin's platform-wide safety ceiling, never lift them above it -- an
//! admin who sets `max_stake_per_user` is making a hard, platform-wide risk
//! decision that no amount of reputation should override. If
//! `max_stake_per_user == 0` (its existing "unlimited" sentinel), only the
//! reputation limit applies. If `base_stake_limit == 0` (the reputation
//! system left unconfigured, which is the default on a freshly initialized
//! contract), the reputation limit itself is treated as unlimited, so
//! existing/legacy deployments that never call `set_reputation_params`
//! behave exactly as before this feature was added.

use soroban_sdk::{Address, Env};

use crate::errors::overflow;
use crate::events::{
    emit_admin_params_changed_i128, emit_admin_params_changed_u32, emit_stake_limit_updated,
    PARAM_BASE_STAKE_LIMIT, PARAM_REPUTATION_MULTIPLIER,
};
use crate::storage::{
    extend_storage_ttl, get_config, get_creator_stats, get_global_stats,
    get_user_total_stake_volume, set_config,
};
use crate::types::{ContractConfig, CreatorStats};

/// Basis-point scale used throughout this module: `10_000` == `1.0`.
pub const BPS_SCALE: i128 = 10_000;

/// Number of a user's resolved (self-created) calls required before
/// reputation scaling applies at all. Below this, only `base_stake_limit`
/// applies, regardless of accuracy or stake volume.
pub const NEW_USER_RESOLVED_THRESHOLD: u32 = 10;

/// Cap on `volume_factor_bps`: `20_000` bps == `2.0x`.
pub const MAX_VOLUME_FACTOR_BPS: i128 = 20_000;

/// Compute `accuracy_bps` for a `CreatorStats` snapshot:
/// `total_correct / max(total_resolved, 1)`, scaled to basis points.
fn accuracy_bps(env: &Env, stats: &CreatorStats) -> i128 {
    let denom = stats.total_resolved.max(1) as i128;
    (stats.total_correct as i128)
        .checked_mul(BPS_SCALE)
        .unwrap_or_else(|| overflow(env))
        .checked_div(denom)
        .unwrap_or_else(|| overflow(env))
}

/// Compute `volume_factor_bps = min(user_volume / platform_average_volume, 2.0x)`.
/// Returns `BPS_SCALE` (neutral `1.0x`) if the platform has no stake history
/// yet, to avoid dividing by zero.
fn volume_factor_bps(env: &Env, user_volume: i128) -> i128 {
    let stats = get_global_stats(env);
    if stats.total_unique_stakers == 0 || stats.total_stake_volume <= 0 {
        return BPS_SCALE;
    }

    let platform_average_volume = stats
        .total_stake_volume
        .checked_div(stats.total_unique_stakers as i128)
        .unwrap_or_else(|| overflow(env));

    if platform_average_volume <= 0 {
        return BPS_SCALE;
    }

    let raw_bps = user_volume
        .max(0)
        .checked_mul(BPS_SCALE)
        .unwrap_or_else(|| overflow(env))
        .checked_div(platform_average_volume)
        .unwrap_or_else(|| overflow(env));

    raw_bps.min(MAX_VOLUME_FACTOR_BPS)
}

/// Compute the raw reputation-weighted limit for `stats`/`user_volume`
/// against `base_stake_limit`/`reputation_multiplier`, *before* the
/// `max_stake_per_user` ceiling is applied.
///
/// Returns `None` when `base_stake_limit <= 0`, meaning "the reputation
/// system is unconfigured / imposes no limit".
fn reputation_limit(
    env: &Env,
    base_stake_limit: i128,
    reputation_multiplier: u32,
    stats: &CreatorStats,
    user_volume: i128,
) -> Option<i128> {
    if base_stake_limit <= 0 {
        return None;
    }

    if stats.total_resolved < NEW_USER_RESOLVED_THRESHOLD {
        // New / unproven account: hard gate at exactly the baseline, ignoring
        // accuracy and volume entirely (see module doc comment).
        return Some(base_stake_limit);
    }

    let acc_bps = accuracy_bps(env, stats);
    let reputation_term = acc_bps
        .checked_mul(reputation_multiplier as i128)
        .unwrap_or_else(|| overflow(env))
        .checked_div(BPS_SCALE)
        .unwrap_or_else(|| overflow(env));
    let factor_bps = BPS_SCALE
        .checked_add(reputation_term)
        .unwrap_or_else(|| overflow(env));

    let vol_factor_bps = volume_factor_bps(env, user_volume);

    let after_accuracy = base_stake_limit
        .checked_mul(factor_bps)
        .unwrap_or_else(|| overflow(env))
        .checked_div(BPS_SCALE)
        .unwrap_or_else(|| overflow(env));

    let final_limit = after_accuracy
        .checked_mul(vol_factor_bps)
        .unwrap_or_else(|| overflow(env))
        .checked_div(BPS_SCALE)
        .unwrap_or_else(|| overflow(env));

    Some(final_limit)
}

/// Combine the reputation limit with the admin's absolute `max_stake_per_user`
/// ceiling (if configured) to get the limit actually enforced by
/// `stake_on_call`. Returns `i128::MAX` when neither cap applies.
pub fn effective_stake_limit(
    env: &Env,
    base_stake_limit: i128,
    reputation_multiplier: u32,
    max_stake_per_user: i128,
    stats: &CreatorStats,
    user_volume: i128,
) -> i128 {
    let rep_limit = reputation_limit(
        env,
        base_stake_limit,
        reputation_multiplier,
        stats,
        user_volume,
    )
    .unwrap_or(i128::MAX);

    if max_stake_per_user > 0 {
        rep_limit.min(max_stake_per_user)
    } else {
        rep_limit
    }
}

/// View: the reputation-weighted individual stake limit currently in effect
/// for `user`, combining `CreatorStats`, historical stake volume, and the
/// admin-configured parameters exactly as `stake_on_call` does. Returns
/// `i128::MAX` when no cap applies at all.
pub fn get_user_stake_limit(env: &Env, user: &Address) -> i128 {
    let config = get_config(env).expect("not initialized");
    let stats = get_creator_stats(env, user);
    let user_volume = get_user_total_stake_volume(env, user);
    effective_stake_limit(
        env,
        config.base_stake_limit,
        config.reputation_multiplier,
        config.max_stake_per_user,
        &stats,
        user_volume,
    )
}

/// Admin-only: set `base_stake_limit` and `reputation_multiplier`.
///
/// # Authorization
/// Current admin must sign.
///
/// # Panics
/// * Contract not initialized.
/// * `base_limit` is negative.
pub fn set_reputation_params(env: Env, base_limit: i128, multiplier: u32) {
    if base_limit < 0 {
        panic!("base_stake_limit cannot be negative");
    }

    let mut config = get_config(&env).expect("not initialized");
    config.admin.require_auth();

    let old_base = config.base_stake_limit;
    let old_multiplier = config.reputation_multiplier;
    config.base_stake_limit = base_limit;
    config.reputation_multiplier = multiplier;

    set_config(&env, &config);
    extend_storage_ttl(&env);

    emit_admin_params_changed_i128(
        &env,
        PARAM_BASE_STAKE_LIMIT,
        &config.admin,
        old_base,
        base_limit,
    );
    emit_admin_params_changed_u32(
        &env,
        PARAM_REPUTATION_MULTIPLIER,
        &config.admin,
        old_multiplier,
        multiplier,
    );
}

/// Recompute `user`'s effective stake limit before and after a `CreatorStats`
/// mutation (i.e. around a `resolve_call` that updates `total_resolved` /
/// `total_correct`) and emit `StakeLimitUpdated` iff the limit actually
/// changed. Called from `resolve_call` after the creator's stats are updated.
pub fn maybe_emit_stake_limit_updated(
    env: &Env,
    user: &Address,
    config: &ContractConfig,
    old_stats: &CreatorStats,
    new_stats: &CreatorStats,
    user_volume: i128,
) {
    let old_limit = effective_stake_limit(
        env,
        config.base_stake_limit,
        config.reputation_multiplier,
        config.max_stake_per_user,
        old_stats,
        user_volume,
    );
    let new_limit = effective_stake_limit(
        env,
        config.base_stake_limit,
        config.reputation_multiplier,
        config.max_stake_per_user,
        new_stats,
        user_volume,
    );

    if old_limit != new_limit {
        emit_stake_limit_updated(env, user, new_limit);
    }
}
