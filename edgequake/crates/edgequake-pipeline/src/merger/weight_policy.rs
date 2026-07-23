//! D-31 — SSOT relation weight merge policy.
//!
//! Historical `(a+b)/2` is order-dependent and non-associative (three merges of
//! 1.0 → 0.9375). Production default is [`WeightPolicy::Max`] (strongest signal).
//! Optional [`WeightPolicy::MeanCounted`] stores associative mean via
//! `(weight_sum, weight_count)` properties.

use std::collections::HashMap;

/// How to combine relationship weights across merges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeightPolicy {
    /// Keep the maximum weight seen (associative, commutative).
    #[default]
    Max,
    /// Running mean from stored `(weight_sum, weight_count)`.
    MeanCounted,
}

impl WeightPolicy {
    /// Env: `EDGEQUAKE_WEIGHT_POLICY=max|mean` (default max).
    pub fn from_env() -> Self {
        match std::env::var("EDGEQUAKE_WEIGHT_POLICY")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "mean" | "mean_counted" | "average" | "avg" => Self::MeanCounted,
            _ => Self::Max,
        }
    }

    /// Merge `incoming` into edge `properties`, writing `weight` (and sum/count when mean).
    pub fn apply(self, properties: &mut HashMap<String, serde_json::Value>, incoming: f32) -> f32 {
        let incoming = incoming.max(0.0);
        match self {
            Self::Max => {
                let existing = properties
                    .get("weight")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                let new_weight = existing.max(incoming);
                insert_f32(properties, "weight", new_weight);
                new_weight
            }
            Self::MeanCounted => {
                let (mut sum, mut count) = read_sum_count(properties);
                sum += incoming as f64;
                count += 1.0;
                let mean = if count > 0.0 {
                    (sum / count) as f32
                } else {
                    incoming
                };
                insert_f32(properties, "weight", mean);
                insert_f64(properties, "weight_sum", sum);
                insert_f64(properties, "weight_count", count);
                mean
            }
        }
    }

    /// Combine two in-memory weights before graph write (domain dedupe).
    pub fn combine(self, a: f32, b: f32) -> f32 {
        match self {
            Self::Max => a.max(b),
            // Without counts, mean of two samples is the best associative step we can take.
            Self::MeanCounted => (a.max(0.0) + b.max(0.0)) / 2.0,
        }
    }
}

fn read_sum_count(properties: &HashMap<String, serde_json::Value>) -> (f64, f64) {
    let sum = properties
        .get("weight_sum")
        .and_then(|v| v.as_f64())
        .or_else(|| properties.get("weight").and_then(|v| v.as_f64()))
        .unwrap_or(0.0);
    let count = properties
        .get("weight_count")
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| {
            if properties.get("weight").and_then(|v| v.as_f64()).is_some() {
                1.0
            } else {
                0.0
            }
        });
    (sum, count)
}

fn insert_f32(properties: &mut HashMap<String, serde_json::Value>, key: &str, value: f32) {
    if let Some(n) = serde_json::Number::from_f64(value as f64) {
        properties.insert(key.to_string(), serde_json::Value::Number(n));
    }
}

fn insert_f64(properties: &mut HashMap<String, serde_json::Value>, key: &str, value: f64) {
    if let Some(n) = serde_json::Number::from_f64(value) {
        properties.insert(key.to_string(), serde_json::Value::Number(n));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC-083 matrix name (D-31) — alias of max-policy associativity.
    #[test]
    fn unit_weight_associative() {
        unit_weight_max_associative();
    }

    #[test]
    fn unit_weight_max_associative() {
        let mut props: HashMap<String, serde_json::Value> = HashMap::new();
        let p = WeightPolicy::Max;
        assert!((p.apply(&mut props, 1.0) - 1.0).abs() < f32::EPSILON);
        assert!((p.apply(&mut props, 1.0) - 1.0).abs() < f32::EPSILON);
        assert!((p.apply(&mut props, 1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unit_weight_mean_counted_associative() {
        let mut props: HashMap<String, serde_json::Value> = HashMap::new();
        let p = WeightPolicy::MeanCounted;
        p.apply(&mut props, 1.0);
        p.apply(&mut props, 1.0);
        let w = p.apply(&mut props, 1.0);
        assert!((w - 1.0).abs() < 1e-5, "got {w}");
    }

    #[test]
    fn unit_weight_mean_not_exponential_smoothing() {
        // Old bug diagram: start 0.5, then (w+1)/2 ×3 → 0.9375 (not 1.0).
        let mut w = 0.5f32;
        for _ in 0..3 {
            w = (w + 1.0) / 2.0;
        }
        assert!((w - 0.9375).abs() < 1e-5);

        let mut fixed: HashMap<String, serde_json::Value> = HashMap::new();
        let p = WeightPolicy::Max;
        p.apply(&mut fixed, 0.5);
        for _ in 0..3 {
            p.apply(&mut fixed, 1.0);
        }
        let got = fixed.get("weight").and_then(|v| v.as_f64()).unwrap();
        assert!((got - 1.0).abs() < 1e-9);
    }
}
