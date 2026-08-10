/**
 * @module kg-schema-presets
 * @description Domain KG schema presets: entity types + relation types + typed edges.
 *
 * Entity lists for domains live in `ENTITY_PRESETS` (language-aware via catalog).
 * Relation lists and edges are English tokens in v1 (no localized relation catalog).
 *
 * @implements SPEC-114 LAW-114-4
 * @implements SPEC-114b LAW-114-9…13
 */

import {
  ENTITY_PRESETS,
  type PresetKey,
  detectPreset,
  normalizeEntityType,
} from '@/constants/entity-presets';

/** Alias — max types per allow-list (entity or relation). */
export const MAX_RELATION_TYPES = 50;

/** Max typed edges per workspace (SPEC-114b). */
export const MAX_RELATION_EDGES = 100;

export type KgSchemaPresetKey = PresetKey;

/** Typed edge: source entity type — relation → target entity type. */
export interface RelationEdge {
  source: string;
  relation: string;
  target: string;
}

export interface KgSchemaPreset {
  entityTypes: string[];
  relationTypes: string[];
  relationEdges: RelationEdge[];
}

/** Curated relation allow-lists per domain (English, UPPER_SNAKE). */
export const RELATION_PRESETS: Record<Exclude<PresetKey, 'custom'>, string[]> = {
  blank: [],
  general: ['RELATED_TO', 'PART_OF', 'LOCATED_IN', 'WORKS_AT', 'CREATED_BY'],
  manufacturing: [
    'PART_OF',
    'PRODUCED_BY',
    'HAS_DEFECT',
    'MEASURED_BY',
    'LOCATED_IN',
    'RELATED_TO',
  ],
  healthcare: [
    'TREATS',
    'DIAGNOSED_WITH',
    'ADMINISTERED_BY',
    'LOCATED_IN',
    'RELATED_TO',
  ],
  legal: ['PARTY_TO', 'GOVERNED_BY', 'CITES', 'REPRESENTED_BY', 'RELATED_TO'],
  research: ['AUTHORED_BY', 'CITES', 'FUNDED_BY', 'PART_OF', 'RELATED_TO'],
  finance: [
    'OWNED_BY',
    'TRANSACTS_WITH',
    'REGULATED_BY',
    'PART_OF',
    'RELATED_TO',
  ],
};

/**
 * Curated typed edges per domain (SPEC-114b).
 * Endpoints must exist in the domain's entity list; relations in RELATION_PRESETS.
 */
export const EDGE_PRESETS: Record<Exclude<PresetKey, 'custom'>, RelationEdge[]> = {
  blank: [],
  general: [
    { source: 'PERSON', relation: 'WORKS_AT', target: 'ORGANIZATION' },
    { source: 'PERSON', relation: 'LOCATED_IN', target: 'LOCATION' },
    { source: 'ORGANIZATION', relation: 'LOCATED_IN', target: 'LOCATION' },
    { source: 'ARTIFACT', relation: 'CREATED_BY', target: 'PERSON' },
    { source: 'CONTENT', relation: 'CREATED_BY', target: 'PERSON' },
    { source: 'EVENT', relation: 'LOCATED_IN', target: 'LOCATION' },
    { source: 'CONCEPT', relation: 'RELATED_TO', target: 'CONCEPT' },
    { source: 'ORGANIZATION', relation: 'PART_OF', target: 'ORGANIZATION' },
  ],
  manufacturing: [
    { source: 'MACHINE', relation: 'HAS_DEFECT', target: 'DEFECT' },
    { source: 'COMPONENT', relation: 'PART_OF', target: 'PRODUCT' },
    { source: 'PRODUCT', relation: 'PRODUCED_BY', target: 'ORGANIZATION' },
    { source: 'MACHINE', relation: 'LOCATED_IN', target: 'LOCATION' },
    { source: 'DEFECT', relation: 'MEASURED_BY', target: 'MEASUREMENT' },
    { source: 'PRODUCT', relation: 'RELATED_TO', target: 'PROCESS' },
  ],
  healthcare: [
    { source: 'PERSON', relation: 'TREATS', target: 'PATIENT' },
    { source: 'PATIENT', relation: 'DIAGNOSED_WITH', target: 'CONDITION' },
    { source: 'DRUG', relation: 'ADMINISTERED_BY', target: 'PERSON' },
    { source: 'PATIENT', relation: 'LOCATED_IN', target: 'LOCATION' },
    { source: 'CONDITION', relation: 'RELATED_TO', target: 'DRUG' },
  ],
  legal: [
    { source: 'PARTY', relation: 'PARTY_TO', target: 'CONTRACT' },
    { source: 'CONTRACT', relation: 'GOVERNED_BY', target: 'REGULATION' },
    { source: 'CASE', relation: 'CITES', target: 'REGULATION' },
    { source: 'PARTY', relation: 'REPRESENTED_BY', target: 'ORGANIZATION' },
    { source: 'CLAUSE', relation: 'RELATED_TO', target: 'CONTRACT' },
  ],
  research: [
    { source: 'PAPER', relation: 'AUTHORED_BY', target: 'PERSON' },
    { source: 'PAPER', relation: 'CITES', target: 'PAPER' },
    { source: 'DATASET', relation: 'FUNDED_BY', target: 'ORGANIZATION' },
    { source: 'DATASET', relation: 'PART_OF', target: 'ORGANIZATION' },
    { source: 'METHOD', relation: 'RELATED_TO', target: 'CONCEPT' },
  ],
  finance: [
    { source: 'SECURITY', relation: 'OWNED_BY', target: 'ORGANIZATION' },
    { source: 'ORGANIZATION', relation: 'TRANSACTS_WITH', target: 'COUNTERPARTY' },
    { source: 'PRODUCT', relation: 'REGULATED_BY', target: 'REGULATION' },
    { source: 'FUND', relation: 'PART_OF', target: 'ORGANIZATION' },
    { source: 'RISK', relation: 'RELATED_TO', target: 'SECURITY' },
  ],
};

/** Full KG schema for a domain preset. */
export function getKgSchemaPreset(
  key: Exclude<PresetKey, 'custom'>,
): KgSchemaPreset {
  return {
    entityTypes: [...ENTITY_PRESETS[key].types],
    relationTypes: [...RELATION_PRESETS[key]],
    relationEdges: EDGE_PRESETS[key].map((e) => ({ ...e })),
  };
}

/** Normalize relation / edge token (same rules as entity). */
export function normalizeRelationType(raw: string): string {
  return normalizeEntityType(raw);
}

export function normalizeRelationEdge(edge: RelationEdge): RelationEdge | null {
  const source = normalizeRelationType(edge.source);
  const relation = normalizeRelationType(edge.relation);
  const target = normalizeRelationType(edge.target);
  if (!source || !relation || !target) return null;
  return { source, relation, target };
}

export function edgeKey(edge: RelationEdge): string {
  const n = normalizeRelationEdge(edge);
  if (!n) return '';
  return `${n.source}|${n.relation}|${n.target}`;
}

export function edgesEqual(a: RelationEdge[], b: RelationEdge[]): boolean {
  const ak = a
    .map(edgeKey)
    .filter(Boolean)
    .sort()
    .join(';');
  const bk = b
    .map(edgeKey)
    .filter(Boolean)
    .sort()
    .join(';');
  return ak === bk;
}

/**
 * Detect preset from entity + relation lists (+ optional edges).
 * Falls back to entity-only detect when relations empty; `custom` if either diverges.
 * When edges provided and non-empty, they must also match EDGE_PRESETS.
 */
export function detectKgSchemaPreset(
  entityTypes: string[],
  relationTypes: string[],
  relationEdges: RelationEdge[] = [],
): PresetKey {
  const entityKey = detectPreset(entityTypes);
  if (entityKey === 'custom') return 'custom';
  if (!relationTypes.length && !relationEdges.length) return entityKey;
  if (relationTypes.length) {
    const expected = RELATION_PRESETS[entityKey]
      .map(normalizeRelationType)
      .sort()
      .join(',');
    const actual = [...relationTypes]
      .map(normalizeRelationType)
      .filter(Boolean)
      .sort()
      .join(',');
    if (expected !== actual) return 'custom';
  }
  if (relationEdges.length && !edgesEqual(relationEdges, EDGE_PRESETS[entityKey])) {
    return 'custom';
  }
  return entityKey;
}

/** Drop edges that reference removed entity or relation types. */
export function filterEdgesForVocabulary(
  edges: RelationEdge[],
  entityTypes: string[],
  relationTypes: string[],
): RelationEdge[] {
  const ents = new Set(entityTypes.map(normalizeRelationType).filter(Boolean));
  const rels = new Set(relationTypes.map(normalizeRelationType).filter(Boolean));
  const seen = new Set<string>();
  const out: RelationEdge[] = [];
  for (const edge of edges) {
    const n = normalizeRelationEdge(edge);
    if (!n) continue;
    if (ents.size && (!ents.has(n.source) || !ents.has(n.target))) continue;
    if (rels.size && !rels.has(n.relation)) continue;
    const k = edgeKey(n);
    if (seen.has(k)) continue;
    seen.add(k);
    out.push(n);
    if (out.length >= MAX_RELATION_EDGES) break;
  }
  return out;
}

/** Rust `default_entity_types()` — used by unit tests for General parity. */
export const RUST_DEFAULT_ENTITY_TYPES = [
  'PERSON',
  'CREATURE',
  'ORGANIZATION',
  'LOCATION',
  'EVENT',
  'CONCEPT',
  'METHOD',
  'CONTENT',
  'DATA',
  'ARTIFACT',
  'NATURALOBJECT',
  'OTHER',
] as const;
