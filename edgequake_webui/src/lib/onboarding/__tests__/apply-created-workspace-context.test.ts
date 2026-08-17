import { describe, expect, it } from 'vitest';
import { buildCreatedContextSearchParams } from '../apply-created-workspace-context';

describe('buildCreatedContextSearchParams', () => {
  it('sets workspace and tenant slugs from names when slug missing', () => {
    const q = buildCreatedContextSearchParams(
      'workspace=old&foo=1',
      { id: 'w1', name: 'My Workspace' },
      { id: 't1', name: 'Acme Org' },
    );
    const params = new URLSearchParams(q);
    expect(params.get('workspace')).toBe('my-workspace');
    expect(params.get('tenant')).toBe('acme-org');
    expect(params.get('foo')).toBe('1');
  });

  it('prefers explicit slugs', () => {
    const q = buildCreatedContextSearchParams(
      '',
      { id: 'w1', name: 'X', slug: 'ws-slug' },
      { id: 't1', name: 'Y', slug: 'tenant-slug' },
    );
    const params = new URLSearchParams(q);
    expect(params.get('workspace')).toBe('ws-slug');
    expect(params.get('tenant')).toBe('tenant-slug');
  });
});
