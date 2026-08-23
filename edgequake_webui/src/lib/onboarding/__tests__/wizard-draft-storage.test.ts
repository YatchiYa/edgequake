import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import {
  clearWizardDraft,
  draftStorageKey,
  hydrateWizardDraft,
  loadWizardDraft,
  saveWizardDraft,
} from '../wizard-draft-storage';
import { EMPTY_WIZARD_DRAFT, draftForStorage } from '../wizard-state';

function installMemorySessionStorage() {
  const store = new Map<string, string>();
  const memory = {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => {
      store.set(key, value);
    },
    removeItem: (key: string) => {
      store.delete(key);
    },
    clear: () => store.clear(),
  };
  Object.defineProperty(globalThis, 'sessionStorage', {
    value: memory,
    configurable: true,
    writable: true,
  });
  return store;
}

describe('wizard-draft-storage', () => {
  beforeEach(() => {
    installMemorySessionStorage();
  });

  afterEach(() => {
    clearWizardDraft('create-workspace');
    clearWizardDraft('create-tenant');
    clearWizardDraft('first-run');
    clearWizardDraft('reconfigure-workspace', 'ws-1');
  });

  it('round-trips non-secret fields and step index', () => {
    const draft = {
      ...EMPTY_WIZARD_DRAFT,
      tenantName: 'Acme',
      workspaceName: 'Main',
      adminPassword: 'super-secret',
      adminPasswordConfirm: 'super-secret',
    };
    saveWizardDraft('create-tenant', draft, 2);
    const loaded = loadWizardDraft('create-tenant');
    expect(loaded).not.toBeNull();
    expect(loaded!.stepIndex).toBe(2);
    expect(loaded!.draft.tenantName).toBe('Acme');
    expect(loaded!.draft.workspaceName).toBe('Main');
    expect(loaded!.draft).not.toHaveProperty('adminPassword');
    expect(loaded!.draft).not.toHaveProperty('adminPasswordConfirm');
    expect(sessionStorage.getItem(draftStorageKey('create-tenant'))).toContain('Acme');
    expect(sessionStorage.getItem(draftStorageKey('create-tenant'))).not.toContain(
      'super-secret',
    );
  });

  it('hydrate never restores passwords', () => {
    saveWizardDraft(
      'first-run',
      {
        ...EMPTY_WIZARD_DRAFT,
        adminUsername: 'root',
        adminPassword: 'should-not-persist',
        adminPasswordConfirm: 'should-not-persist',
        tenantName: 'Org',
      },
      0,
    );
    const stored = loadWizardDraft('first-run');
    const hydrated = hydrateWizardDraft(
      {
        ...EMPTY_WIZARD_DRAFT,
        adminPassword: 'typed-in-ui',
        adminPasswordConfirm: 'typed-in-ui',
      },
      stored,
    );
    expect(hydrated.adminUsername).toBe('root');
    expect(hydrated.tenantName).toBe('Org');
    expect(hydrated.adminPassword).toBe('');
    expect(hydrated.adminPasswordConfirm).toBe('');
  });

  it('draftForStorage strips secrets', () => {
    const safe = draftForStorage({
      ...EMPTY_WIZARD_DRAFT,
      adminPassword: 'x',
      adminPasswordConfirm: 'x',
    });
    expect(safe).not.toHaveProperty('adminPassword');
    expect(safe).not.toHaveProperty('adminPasswordConfirm');
  });

  it('clear removes the key', () => {
    saveWizardDraft('create-workspace', { ...EMPTY_WIZARD_DRAFT, workspaceName: 'W' }, 1);
    clearWizardDraft('create-workspace');
    expect(loadWizardDraft('create-workspace')).toBeNull();
  });

  it('scopes reconfigure drafts by workspace id (EC-101-22)', () => {
    saveWizardDraft(
      'reconfigure-workspace',
      { ...EMPTY_WIZARD_DRAFT, extractionLanguage: 'Chinese' },
      2,
      'ws-1',
    );
    expect(loadWizardDraft('reconfigure-workspace')).toBeNull();
    const loaded = loadWizardDraft('reconfigure-workspace', 'ws-1');
    expect(loaded?.draft.extractionLanguage).toBe('Chinese');
    expect(loaded?.stepIndex).toBe(2);
    expect(draftStorageKey('reconfigure-workspace', 'ws-1')).toContain('ws-1');
  });

  it('persists reconfigure model picks (v2)', () => {
    saveWizardDraft(
      'reconfigure-workspace',
      { ...EMPTY_WIZARD_DRAFT, useServerDefaults: false },
      1,
      'ws-2',
      {
        llm: { provider: 'mistral', model: 'mistral-small-latest' },
        embedding: { provider: 'mistral', model: 'mistral-embed', dimension: 1024 },
        advancedOpen: true,
      },
    );
    const loaded = loadWizardDraft('reconfigure-workspace', 'ws-2');
    expect(loaded?.version).toBe(2);
    expect(loaded?.modelPicks?.embedding?.provider).toBe('mistral');
    expect(loaded?.modelPicks?.embedding?.dimension).toBe(1024);
    expect(loaded?.modelPicks?.advancedOpen).toBe(true);
  });
});
