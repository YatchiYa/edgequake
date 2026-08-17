import { describe, expect, it } from 'vitest';
import { formatContextLabels, smartTruncate } from '../format-context-labels';

describe('smartTruncate', () => {
  it('leaves short strings intact', () => {
    expect(smartTruncate('Research')).toBe('Research');
  });

  it('keeps a distinctive suffix for long names', () => {
    const long = 'spec101-cap-ws-375 tenant 1785655737087';
    const short = smartTruncate(long, 22);
    expect(short.length).toBeLessThanOrEqual(22);
    expect(short).toContain('…');
    expect(short.startsWith('spec101')).toBe(true);
    expect(short.slice(-4)).toBe('7087');
  });
});

describe('formatContextLabels', () => {
  it('builds one-line Tenant — Workspace display', () => {
    const formatted = formatContextLabels({
      tenantName: 'Acme Org',
      workspaceName: 'Research',
    });
    expect(formatted.lineDisplay).toBe('Acme Org — Research');
    expect(formatted.title).toBe('Acme Org — Research');
    expect(formatted.ariaLabel).toBe('Tenant Acme Org, Workspace Research');
  });

  it('shortens each side for long names while title stays full', () => {
    const tenant = 'spec101-chips tenant 1785655729914';
    const workspace = 'spec101-chips ws 1785655729914';
    const formatted = formatContextLabels(
      { tenantName: tenant, workspaceName: workspace },
      { maxLen: 16 },
    );
    expect(formatted.title).toBe(`${tenant} — ${workspace}`);
    expect(formatted.lineDisplay).toContain('—');
    expect(formatted.lineDisplay.length).toBeLessThan(formatted.title.length);
    expect(formatted.tenantShort).toContain('…');
  });

  it('keeps workspace placeholder when workspace missing', () => {
    const formatted = formatContextLabels({ tenantName: 'Acme Org' });
    expect(formatted.hasWorkspace).toBe(false);
    expect(formatted.lineDisplay).toBe('Acme Org — Select workspace');
  });

  it('uses custom select placeholders', () => {
    const formatted = formatContextLabels(
      {},
      { selectTenant: 'Choisir org', selectWorkspace: 'Choisir espace' },
    );
    expect(formatted.tenantDisplay).toBe('Choisir org');
    expect(formatted.lineDisplay).toBe('Choisir org');
  });
});
