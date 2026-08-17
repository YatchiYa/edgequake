//! SPEC-091 adaptive batch sizing (07-migration-engine.md §Efficiency).
//!
//! Probe phase: start small, double until a batch exceeds `target_ms` or the
//! ceiling. Steady state: AIMD-lite — grow 1.25× while under target, halve on
//! a slow batch (> `slow_ms`) or explicit throttle. Bounded `[min, max]`.

/// Controller for one running job's batch size. Pure logic — unit-tested.
#[derive(Debug, Clone)]
pub struct AdaptiveBatchSizer {
    size: u32,
    min: u32,
    max: u32,
    target_ms: u64,
    slow_ms: u64,
    probing: bool,
}

/// Why the last adjustment happened (maps to `throttle_reason` on the job row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adjustment {
    ProbeGrow,
    SteadyGrow,
    SlowBatchShrink,
    ThrottleShrink,
    Steady,
}

impl AdaptiveBatchSizer {
    pub fn new(min: u32, max: u32, target_ms: u64, slow_ms: u64) -> Self {
        debug_assert!(min >= 1 && min <= max);
        Self {
            size: min,
            min,
            max,
            target_ms,
            slow_ms,
            probing: true,
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn is_probing(&self) -> bool {
        self.probing
    }

    /// Record one committed batch. `throttled` = an external gate fired this round.
    pub fn record(&mut self, duration_ms: u64, throttled: bool) -> Adjustment {
        if throttled {
            self.probing = false;
            self.size = (self.size / 2).max(self.min);
            return Adjustment::ThrottleShrink;
        }
        if duration_ms > self.slow_ms {
            self.probing = false;
            self.size = (self.size / 2).max(self.min);
            return Adjustment::SlowBatchShrink;
        }
        if self.probing {
            // Probe: double while comfortably under target.
            if duration_ms <= self.target_ms && self.size < self.max {
                self.size = (self.size.saturating_mul(2)).min(self.max);
                return Adjustment::ProbeGrow;
            }
            self.probing = false;
            return Adjustment::Steady;
        }
        if duration_ms < self.target_ms / 2 && self.size < self.max {
            let next = self.size + (self.size / 4).max(1);
            self.size = next.min(self.max);
            return Adjustment::SteadyGrow;
        }
        Adjustment::Steady
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec091_probe_doubles_until_target_then_steady() {
        let mut s = AdaptiveBatchSizer::new(64, 32_000, 250, 500);
        assert_eq!(s.size(), 64);
        assert_eq!(s.record(50, false), Adjustment::ProbeGrow);
        assert_eq!(s.size(), 128);
        // 400ms exceeds target (250) → probe ends, steady state begins.
        assert_eq!(s.record(400, false), Adjustment::Steady);
        assert!(!s.is_probing());
        // 600ms exceeds slow threshold (500) → halve.
        assert_eq!(s.record(600, false), Adjustment::SlowBatchShrink);
        assert_eq!(s.size(), 64);
    }

    #[test]
    fn contract_spec091_slow_batch_halves_with_floor() {
        let mut s = AdaptiveBatchSizer::new(64, 32_000, 250, 500);
        s.record(10, false); // 128
        s.record(10, false); // 256
        s.record(10, false); // 512
        s.record(10, false); // exit probe (duration <= target but size*2... still grows)
        let before = s.size();
        assert_eq!(s.record(600, false), Adjustment::SlowBatchShrink);
        assert_eq!(s.size(), (before / 2).max(64));
        // Floor respected.
        let mut tiny = AdaptiveBatchSizer::new(64, 32_000, 250, 500);
        tiny.record(999, false);
        assert_eq!(tiny.size(), 64);
    }

    #[test]
    fn contract_spec091_throttle_shrinks_and_growth_capped() {
        let mut s = AdaptiveBatchSizer::new(64, 256, 250, 500);
        while s.size() < 256 {
            s.record(10, false);
        }
        assert_eq!(s.size(), 256);
        // At ceiling, fast batches stay at ceiling.
        s.record(10, false);
        assert!(s.size() <= 256);
        assert_eq!(s.record(10, true), Adjustment::ThrottleShrink);
        assert_eq!(s.size(), 128);
    }

    #[test]
    fn contract_spec091_steady_growth_quarter_increment() {
        let mut s = AdaptiveBatchSizer::new(100, 32_000, 250, 500);
        // Exit probe quickly by exceeding target on first batch.
        s.record(300, false); // > target → probe ends (slow? 300 < 500 slow_ms)
        assert!(!s.is_probing());
        let size = s.size();
        assert_eq!(s.record(50, false), Adjustment::SteadyGrow);
        assert_eq!(s.size(), size + (size / 4).max(1));
    }
}
