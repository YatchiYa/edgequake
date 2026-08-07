//! SPEC-111: forbid indexed-column `::text` casts in KV residue / migration 125.
#![cfg(test)]

#[test]
fn contract_spec111_residue_cast_direction() {
    let residue = include_str!("../src/migration_engine/advisor/residue.rs");
    let m125 = include_str!("../../../migrations/125_spec091_kv_drop.sql");
    for (label, src) in [("residue.rs", residue), ("125.sql", m125)] {
        assert!(
            !src.contains("id::text = substring") && !src.contains("document_id::text = substring"),
            "{label}: must not cast indexed uuid column to text (SPEC-111 #362)"
        );
        assert!(
            src.contains("NULLIF(substring") && src.contains(")::uuid"),
            "{label}: must cast extracted key to uuid (Index Cond–friendly)"
        );
    }
}

#[test]
fn contract_spec111_retirable_uses_uncovered_not_emptiness() {
    let types = include_str!("../src/migration_engine/advisor/types.rs");
    assert!(
        types.contains("uncovered_chunk_rows == 0"),
        "chunk_retirable must gate on uncovered_chunk_rows"
    );
    assert!(
        !types.contains("&& self.legacy_chunk_rows == 0"),
        "chunk_retirable must not require legacy emptiness (LAW-111-2)"
    );
}

#[test]
fn contract_spec111_fleet_mirror_uses_normalize_index() {
    let src = include_str!("../src/adapters/postgres/fleet_embedding_index.rs");
    assert!(
        src.contains("load_entity_name_index_pool") && src.contains("EntityNameIndex"),
        "mirror_legacy_batch must reuse coverage EntityNameIndex (SPEC-111 #363)"
    );
    assert!(
        !src.contains("es.name = $1 OR es.name = ($4::text || '::' || $1)"),
        "mirror must not use exact-name-only entity join antipattern"
    );
}
