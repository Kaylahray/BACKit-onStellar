#![cfg(test)]

fn compute_payout(staker_winning_stake: i128, total_winning_stake: i128, total_losing_stake: i128, fee_bps: u32) -> Option<i128> {
    if total_winning_stake <= 0 { return None; }
    let total_fee = total_losing_stake.checked_mul(fee_bps as i128)?.checked_div(10000)?;
    let net_losing = total_losing_stake.checked_sub(total_fee)?;
    let prize = staker_winning_stake.checked_mul(net_losing)?.checked_div(total_winning_stake)?;
    staker_winning_stake.checked_add(prize)
}

#[test]
fn fuzz_payout_never_exceeds_upper_bound() {
    let cases: &[(i128, i128, i128, u32)] = &[
        (100, 1000, 500, 100),
        (1, 1_000_000_000, 1_000_000_000, 500),
        (i128::MAX / 4, i128::MAX / 2, i128::MAX / 2, 0),
        (i128::MAX / 8, i128::MAX / 4, i128::MAX / 4, 1000),
        (500_000, 1_000_000, 0, 500),
    ];
    for &(s, tw, tl, fee) in cases {
        if let Some(p) = compute_payout(s, tw, tl, fee) {
            let upper = s + tl * s / tw;
            assert!(p <= upper, "payout {p} > upper bound {upper}");
        }
    }
}

#[test]
fn fuzz_sum_of_payouts_equals_pool_minus_fee() {
    let stakes: &[i128] = &[100, 200, 300, 400];
    let total_winning: i128 = stakes.iter().sum();
    let total_losing: i128 = 500;
    let fee_bps: u32 = 200;
    let fee = total_losing * fee_bps as i128 / 10000;
    let expected = total_winning + total_losing - fee;
    let sum: i128 = stakes.iter().filter_map(|&s| compute_payout(s, total_winning, total_losing, fee_bps)).sum();
    assert!((sum - expected).abs() <= stakes.len() as i128);
}

#[test]
fn fuzz_extreme_asymmetry() {
    let p = compute_payout(1, 1_000_000_000, 1_000_000_000, 100).unwrap();
    assert!(p >= 1);
}

#[test]
fn fuzz_fee_rates() {
    for &fee_bps in &[0u32, 100, 500, 1000, 5000] {
        let p = compute_payout(1_000, 10_000, 10_000, fee_bps).unwrap();
        assert!(p > 0);
    }
}

#[test]
fn fuzz_empty_winning_pool_returns_none() {
    assert!(compute_payout(100, 0, 500, 100).is_none());
}

#[test]
fn fuzz_no_overflow_large_inputs() {
    for &(s, tw, tl, fee) in &[
        (i128::MAX / 4, i128::MAX / 2, i128::MAX / 2, 0u32),
        (i128::MAX / 8, i128::MAX / 4, i128::MAX / 8, 500),
    ] {
        let _ = compute_payout(s, tw, tl, fee);
    }
}

#[test]
fn fuzz_single_winner_gets_all() {
    let fee = 1_000i128 * 200 / 10000;
    let expected = 500 + 1_000 - fee;
    assert_eq!(compute_payout(500, 500, 1_000, 200).unwrap(), expected);
}

#[test]
fn fuzz_concurrent_claims() {
    let stakes: &[i128] = &[100, 200, 300, 400];
    let tw: i128 = stakes.iter().sum();
    let tl: i128 = 2_000;
    let fee_bps = 300u32;
    let fee = tl * fee_bps as i128 / 10000;
    let expected = tw + tl - fee;
    let sum: i128 = stakes.iter().filter_map(|&s| compute_payout(s, tw, tl, fee_bps)).sum();
    assert!((sum - expected).abs() <= stakes.len() as i128);
}
