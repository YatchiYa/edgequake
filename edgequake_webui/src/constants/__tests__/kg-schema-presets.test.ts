import { ENTITY_PRESETS } from '@/constants/entity-presets';
import {
  EDGE_PRESETS,
  RELATION_PRESETS,
  RUST_DEFAULT_ENTITY_TYPES,
  detectKgSchemaPreset,
  filterEdgesForVocabulary,
  getKgSchemaPreset,
} from '@/constants/kg-schema-presets';
import { describe, expect, it } from 'bun:test';

describe('SPEC-114 / 114b kg-schema-presets', () => {
  it('general entities match Rust default_entity_types()', () => {
    expect(ENTITY_PRESETS.general.types).toEqual([...RUST_DEFAULT_ENTITY_TYPES]);
  });

  it('blank preset is an empty slate', () => {
    const schema = getKgSchemaPreset('blank');
    expect(schema.entityTypes).toEqual([]);
    expect(schema.relationTypes).toEqual([]);
    expect(schema.relationEdges).toEqual([]);
    expect(detectKgSchemaPreset([], [], [])).toBe('blank');
  });

  it('every non-blank domain preset has entity, relation, and edge lists', () => {
    for (const key of Object.keys(ENTITY_PRESETS) as Array<
      keyof typeof ENTITY_PRESETS
    >) {
      if (key === 'blank') continue;
      const schema = getKgSchemaPreset(key);
      expect(schema.entityTypes.length).toBeGreaterThan(0);
      expect(schema.relationTypes.length).toBeGreaterThan(0);
      expect(schema.relationEdges.length).toBeGreaterThan(0);
      expect(schema.relationTypes).toEqual(RELATION_PRESETS[key]);
      expect(schema.relationEdges).toEqual(EDGE_PRESETS[key]);
      // Edges must reference vocabulary tokens
      for (const edge of schema.relationEdges) {
        expect(schema.entityTypes).toContain(edge.source);
        expect(schema.entityTypes).toContain(edge.target);
        expect(schema.relationTypes).toContain(edge.relation);
      }
    }
  });

  it('detectKgSchemaPreset matches manufacturing including edges', () => {
    const schema = getKgSchemaPreset('manufacturing');
    expect(
      detectKgSchemaPreset(
        schema.entityTypes,
        schema.relationTypes,
        schema.relationEdges,
      ),
    ).toBe('manufacturing');
  });

  it('detectKgSchemaPreset returns custom when relations diverge', () => {
    const schema = getKgSchemaPreset('general');
    expect(
      detectKgSchemaPreset(schema.entityTypes, ['TOTALLY_DIFFERENT']),
    ).toBe('custom');
  });

  it('detectKgSchemaPreset returns custom when edges diverge', () => {
    const schema = getKgSchemaPreset('general');
    expect(
      detectKgSchemaPreset(schema.entityTypes, schema.relationTypes, [
        { source: 'PERSON', relation: 'WORKS_AT', target: 'EVENT' },
      ]),
    ).toBe('custom');
  });

  it('filterEdgesForVocabulary drops edges for removed entity types', () => {
    const schema = getKgSchemaPreset('general');
    const withoutPerson = schema.entityTypes.filter((t) => t !== 'PERSON');
    const filtered = filterEdgesForVocabulary(
      schema.relationEdges,
      withoutPerson,
      schema.relationTypes,
    );
    expect(filtered.every((e) => e.source !== 'PERSON' && e.target !== 'PERSON')).toBe(
      true,
    );
  });
});
