/**
 * SPEC-096 LAW-L6 — entity type catalog unit tests.
 */
import { describe, expect, it } from 'vitest';
import { ENTITY_PRESETS } from '@/constants/entity-presets';
import {
  detectCanonicalPreset,
  getPresetTypes,
  localizeType,
  localizeTypes,
  remapPresetTypes,
} from '@/constants/entity-type-catalog';

describe('entity-type-catalog (SPEC-096 LAW-L6)', () => {
  it('spec096_entity_type_catalog_french_general: General → French tokens', () => {
    const french = getPresetTypes('general', 'French');
    expect(french).toContain('PERSONNE');
    expect(french).toContain('ORGANISATION');
    expect(french).toContain('LIEU');
    expect(french).toContain('EVENEMENT');
    expect(french).toHaveLength(ENTITY_PRESETS.general.types.length);
    expect(detectCanonicalPreset(french)).toBe('general');
  });

  it('round-trips English ↔ French for General preset', () => {
    const english = getPresetTypes('general', 'English');
    const french = remapPresetTypes(english, 'English', 'French');
    expect(french).not.toBeNull();
    expect(french).toEqual(getPresetTypes('general', 'French'));

    const back = remapPresetTypes(french!, 'French', 'English');
    expect(back).toEqual(english);
  });

  it('spec096_entity_type_catalog_custom_no_remap: custom list unchanged', () => {
    const custom = ['PERSON', 'MY_CUSTOM_TYPE', 'WIDGET'];
    expect(detectCanonicalPreset(custom)).toBe('custom');
    expect(remapPresetTypes(custom, 'English', 'French')).toBeNull();
  });

  it('localizeType maps known tokens and passes through unknowns', () => {
    expect(localizeType('PERSON', 'French')).toBe('PERSONNE');
    expect(localizeType('personne', 'English')).toBe('PERSON');
    expect(localizeType('WIDGET_X', 'French')).toBe('WIDGET_X');
  });

  it('localizeTypes preserves order', () => {
    expect(localizeTypes(['PERSON', 'LOCATION'], 'French')).toEqual([
      'PERSONNE',
      'LIEU',
    ]);
  });

  it('detects manufacturing preset in French', () => {
    const fr = getPresetTypes('manufacturing', 'French');
    expect(detectCanonicalPreset(fr)).toBe('manufacturing');
    expect(fr).toContain('COMPOSANT');
    expect(fr).toContain('DEFAUT');
  });

  it('server default / null language resolves to English', () => {
    expect(getPresetTypes('general', null)).toEqual(ENTITY_PRESETS.general.types);
    expect(getPresetTypes('general', '__server_default__')).toEqual(
      ENTITY_PRESETS.general.types,
    );
  });
});
