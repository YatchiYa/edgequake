//! SPEC-061/062/063 — shared concurrent stress knobs (clients, mult, scale, pool).
#![allow(dead_code)]

use edgequake_storage::PostgresConfig;
use std::time::Duration;

/// Matrix scale: `default` (CI) / `prod` (50k) / `large` (capacity ladder).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfScale {
    Default,
    Prod,
    Large,
}

impl PerfScale {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Prod => "prod",
            Self::Large => "large",
        }
    }
}

/// Capacity ladder step for `EDGEQUAKE_PERF_SCALE=large` (SPEC-063).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityLadder {
    L1,
    L2,
    L3,
}

impl CapacityLadder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::L1 => "L1",
            Self::L2 => "L2",
            Self::L3 => "L3",
        }
    }

    pub fn vector_rows(self) -> usize {
        match self {
            Self::L1 => 100_000,
            Self::L2 => 500_000,
            Self::L3 => 1_000_000,
        }
    }
}

/// Read `EDGEQUAKE_PERF_SCALE` (`large` is distinct from `prod`).
pub fn perf_scale() -> PerfScale {
    match std::env::var("EDGEQUAKE_PERF_SCALE")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "large" => PerfScale::Large,
        "prod" => PerfScale::Prod,
        _ => PerfScale::Default,
    }
}

/// Read `EDGEQUAKE_CAPACITY_LADDER` (default L1).
pub fn capacity_ladder() -> CapacityLadder {
    match std::env::var("EDGEQUAKE_CAPACITY_LADDER")
        .unwrap_or_else(|_| "L1".into())
        .to_ascii_uppercase()
        .as_str()
    {
        "L2" => CapacityLadder::L2,
        "L3" => CapacityLadder::L3,
        _ => CapacityLadder::L1,
    }
}

/// Concurrent clients: pg17/18 → 16; pg16 / unset → 8.
pub fn stress_clients() -> usize {
    match std::env::var("EQ_POSTGRES_MAJOR").as_deref() {
        Ok("17") | Ok("18") => 16,
        _ => 8,
    }
}

/// Stress multiplier vs single-client: pg17/18 → 1.5; pg16 → 2.0.
pub fn stress_mult() -> f64 {
    match std::env::var("EQ_POSTGRES_MAJOR").as_deref() {
        Ok("17") | Ok("18") => 1.5,
        _ => 2.0,
    }
}

/// Pool size for concurrent gates: at least clients and prod default (32).
pub fn stress_pool_max(clients: usize) -> u32 {
    (clients as u32).max(32)
}

/// Apply stress pool sizing on a test config.
pub fn with_stress_pool(mut config: PostgresConfig, clients: usize) -> PostgresConfig {
    config.max_connections = stress_pool_max(clients);
    config
}

/// Intentional saturation: tiny pool under many clients (acquire wait = queue).
pub fn with_saturation_pool(mut config: PostgresConfig) -> PostgresConfig {
    config.max_connections = 5;
    // Allow queueing under oversubscription; hangs >30s indicate deadlock.
    config.connect_timeout = Duration::from_secs(30);
    config
}

pub struct AnnScale {
    pub rows: usize,
    pub dim: usize,
    pub queries_per_client: usize,
    pub batch_size: usize,
}

pub fn ann_scale(scale: PerfScale) -> AnnScale {
    match scale {
        PerfScale::Default => AnnScale {
            rows: 10_000,
            dim: 64,
            queries_per_client: 50,
            batch_size: 2000,
        },
        PerfScale::Prod => AnnScale {
            rows: 50_000,
            dim: 1536,
            queries_per_client: 40,
            batch_size: 1000,
        },
        PerfScale::Large => {
            let ladder = capacity_ladder();
            AnnScale {
                rows: ladder.vector_rows(),
                dim: 1536,
                queries_per_client: 30,
                batch_size: 1000,
            }
        }
    }
}

pub struct FtsScale {
    pub rows: usize,
    pub dim: usize,
    pub queries_per_client: usize,
    pub batch_size: usize,
}

pub fn fts_scale(scale: PerfScale) -> FtsScale {
    match scale {
        PerfScale::Default => FtsScale {
            rows: 10_000,
            dim: 8,
            queries_per_client: 30,
            batch_size: 500,
        },
        PerfScale::Prod => FtsScale {
            rows: 50_000,
            dim: 8,
            queries_per_client: 30,
            batch_size: 500,
        },
        PerfScale::Large => FtsScale {
            rows: capacity_ladder().vector_rows().min(100_000),
            dim: 8,
            queries_per_client: 20,
            batch_size: 500,
        },
    }
}

pub struct ExpandScale {
    pub hubs: usize,
    pub leaves: usize,
    pub queries_per_client: usize,
}

pub fn expand_scale(scale: PerfScale) -> ExpandScale {
    match scale {
        PerfScale::Default => ExpandScale {
            hubs: 100,
            leaves: 50,
            queries_per_client: 40,
        },
        PerfScale::Prod => ExpandScale {
            hubs: 200,
            leaves: 100,
            queries_per_client: 40,
        },
        PerfScale::Large => ExpandScale {
            hubs: 400,
            leaves: 100,
            queries_per_client: 30,
        },
    }
}

pub struct MixScale {
    pub seed_rows: usize,
    pub dim: usize,
    pub queries_per_client: usize,
}

pub fn mix_scale(scale: PerfScale) -> MixScale {
    match scale {
        PerfScale::Default => MixScale {
            seed_rows: 80,
            dim: 1536,
            queries_per_client: 10,
        },
        PerfScale::Prod => MixScale {
            seed_rows: 5_000,
            dim: 1536,
            queries_per_client: 10,
        },
        PerfScale::Large => MixScale {
            seed_rows: 10_000,
            dim: 1536,
            queries_per_client: 8,
        },
    }
}

/// Absolute Mix concurrent p95 cap (release tighter than debug).
pub fn mix_absolute_budget_ms() -> f64 {
    let release = std::env::var("EDGEQUAKE_PERF_RELEASE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if release {
        5_000.0
    } else {
        15_000.0
    }
}

/// FTS concurrent budget from Q-FTS=200ms × mult.
pub fn fts_budget_ms() -> f64 {
    200.0 * stress_mult()
}

/// Expand concurrent budget from Q2-expand=100ms × mult.
pub fn expand_budget_ms() -> f64 {
    100.0 * stress_mult()
}

/// Pool-saturation wall (queueing expected; fail only on hang-like latency).
pub const POOL_SATURATION_BUDGET_MS: f64 = 2_000.0;

/// SPEC-066 — Wave-2 ceiling corpus size.
///
/// Env precedence:
/// 1. `EDGEQUAKE_CEILING_ROWS` (explicit N)
/// 2. `EQ_CEILING_STEP=L2|L3|SEEK` → 500k / 1M / 250k
pub fn ceiling_corpus_rows() -> usize {
    if let Some(n) = std::env::var("EDGEQUAKE_CEILING_ROWS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
    {
        return n.max(1_000);
    }
    match std::env::var("EQ_CEILING_STEP")
        .unwrap_or_else(|_| "L2".into())
        .to_ascii_uppercase()
        .as_str()
    {
        "L3" => 1_000_000,
        "SEEK" => 250_000,
        _ => 500_000,
    }
}

/// Hang cliff for SPEC-066 (same physics as SPEC-063 L2/L3).
pub fn ceiling_hang_cliff_ms(rows: usize) -> f64 {
    if rows >= 1_000_000 {
        20_000.0
    } else if rows >= 500_000 {
        10_000.0
    } else {
        5_000.0
    }
}
