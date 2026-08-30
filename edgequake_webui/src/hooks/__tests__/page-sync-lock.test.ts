/**
 * SPEC-143 — Controller lock unit tests (U-143-02).
 */

import { describe, expect, it } from 'bun:test';

/**
 * Pure lock helper mirroring usePageSyncController.apply logic for unit tests
 * without mounting React.
 */
function shouldAcceptUpdate(args: {
  now: number;
  lockUntil: number;
  currentDriver: 'none' | 'pdf' | 'md' | 'external';
  source: 'none' | 'pdf' | 'md' | 'external';
}): boolean {
  const locked = args.now < args.lockUntil;
  if (locked && args.currentDriver !== 'none' && args.currentDriver !== args.source) {
    return false;
  }
  return true;
}

describe('page sync lock (U-143-02)', () => {
  it('accepts same-driver updates while locked', () => {
    expect(
      shouldAcceptUpdate({
        now: 100,
        lockUntil: 300,
        currentDriver: 'pdf',
        source: 'pdf',
      }),
    ).toBe(true);
  });

  it('rejects cross-driver updates while locked', () => {
    expect(
      shouldAcceptUpdate({
        now: 100,
        lockUntil: 300,
        currentDriver: 'pdf',
        source: 'md',
      }),
    ).toBe(false);
  });

  it('accepts cross-driver after settle', () => {
    expect(
      shouldAcceptUpdate({
        now: 400,
        lockUntil: 300,
        currentDriver: 'pdf',
        source: 'md',
      }),
    ).toBe(true);
  });
});
