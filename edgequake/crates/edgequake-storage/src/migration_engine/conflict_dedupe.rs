//! SPEC-139 LAW-139-1: last-write-wins collapse so `INSERT … ON CONFLICT DO
//! UPDATE` never proposes the same arbiter key twice in one statement.
//!
//! Postgres raises SQLSTATE 21000 ("cannot affect row a second time") when the
//! proposed set duplicates a conflict target
//! (<https://www.postgresql.org/docs/current/sql-insert.html>).

use std::collections::HashMap;
use std::hash::Hash;

/// Keep the **last** value for each key; first-seen key order is preserved.
///
/// Unique keys pass through in input order. Duplicates keep the last payload
/// at the first occurrence's position (last-write-wins, stable unique prefix).
pub fn dedupe_last_write_wins<K, T>(rows: Vec<(K, T)>) -> Vec<(K, T)>
where
    K: Eq + Hash + Clone,
{
    if rows.is_empty() {
        return Vec::new();
    }
    let mut last: HashMap<K, T> = HashMap::with_capacity(rows.len());
    let mut order: Vec<K> = Vec::with_capacity(rows.len());
    for (k, v) in rows {
        if last.insert(k.clone(), v).is_none() {
            order.push(k);
        }
    }
    order
        .into_iter()
        .map(|k| {
            let v = last.remove(&k).expect("key inserted");
            (k, v)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec139_dedupe_last_write_wins_and_order() {
        let out = dedupe_last_write_wins(vec![(1u8, "a"), (2u8, "b"), (1u8, "c"), (3u8, "d")]);
        assert_eq!(out, vec![(1u8, "c"), (2u8, "b"), (3u8, "d")]);
    }

    #[test]
    fn contract_spec139_dedupe_unique_passthrough() {
        let out = dedupe_last_write_wins(vec![(1u8, "a"), (2u8, "b")]);
        assert_eq!(out, vec![(1u8, "a"), (2u8, "b")]);
    }

    #[test]
    fn contract_spec139_dedupe_empty() {
        let out: Vec<(u8, i32)> = dedupe_last_write_wins::<u8, i32>(Vec::new());
        assert!(out.is_empty());
    }
}
