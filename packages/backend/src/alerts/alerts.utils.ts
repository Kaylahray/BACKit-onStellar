export const DEFAULT_TRIGGER_THRESHOLD_PERCENT = 2;

/**
 * Returns true when currentPrice is within `thresholdPercent` of targetPrice.
 *
 * Per acceptance criteria, the trigger condition is purely "within 2% of the
 * target price" - direction is stored on the alert as metadata (used for
 * display / deep-linking context) but does not gate whether the alert fires.
 * Invalid targets (<= 0) never trigger.
 */
export function isPriceNearTarget(
  currentPrice: number,
  targetPrice: number,
  thresholdPercent: number = DEFAULT_TRIGGER_THRESHOLD_PERCENT,
): boolean {
  if (!Number.isFinite(currentPrice) || !Number.isFinite(targetPrice))
    return false;
  if (targetPrice <= 0) return false;

  const percentDiff =
    (Math.abs(currentPrice - targetPrice) / targetPrice) * 100;
  // Small epsilon guards against floating-point rounding at the exact
  // threshold boundary (e.g. 1.02 vs 1.0 not evaluating to precisely 2%).
  const epsilon = 1e-9;
  return percentDiff <= thresholdPercent + epsilon;
}
