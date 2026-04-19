import { describe, expect, it } from 'vitest';
import { getAppVersion } from '../app-version';

describe('getAppVersion', () => {
  it('uses the current package version and always prefixes it with v', () => {
    expect(getAppVersion()).toBe('v0.10.7');
  });

  it('normalizes a provided version string without duplicating the prefix', () => {
    expect(getAppVersion('0.10.7')).toBe('v0.10.7');
    expect(getAppVersion('v0.10.7')).toBe('v0.10.7');
  });
});
