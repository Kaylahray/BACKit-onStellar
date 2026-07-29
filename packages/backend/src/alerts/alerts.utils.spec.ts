import { isPriceNearTarget } from './alerts.utils';

describe('isPriceNearTarget', () => {
  it('returns true when price exactly matches target', () => {
    expect(isPriceNearTarget(1.0, 1.0)).toBe(true);
  });

  it('returns true when price is within 2% above target', () => {
    expect(isPriceNearTarget(1.019, 1.0)).toBe(true);
  });

  it('returns true when price is within 2% below target', () => {
    expect(isPriceNearTarget(0.981, 1.0)).toBe(true);
  });

  it('returns true exactly at the 2% boundary', () => {
    expect(isPriceNearTarget(1.02, 1.0)).toBe(true);
  });

  it('returns false just outside the 2% boundary', () => {
    expect(isPriceNearTarget(1.021, 1.0)).toBe(false);
  });

  it('returns false when price is far from target', () => {
    expect(isPriceNearTarget(2.0, 1.0)).toBe(false);
  });

  it('respects a custom threshold', () => {
    expect(isPriceNearTarget(1.04, 1.0, 5)).toBe(true);
    expect(isPriceNearTarget(1.06, 1.0, 5)).toBe(false);
  });

  it('returns false for a zero or negative target price', () => {
    expect(isPriceNearTarget(1.0, 0)).toBe(false);
    expect(isPriceNearTarget(1.0, -5)).toBe(false);
  });

  it('returns false for non-finite inputs', () => {
    expect(isPriceNearTarget(NaN, 1.0)).toBe(false);
    expect(isPriceNearTarget(1.0, NaN)).toBe(false);
    expect(isPriceNearTarget(Infinity, 1.0)).toBe(false);
  });
});
