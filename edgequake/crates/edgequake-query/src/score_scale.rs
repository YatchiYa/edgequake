//! Typed retrieval scores (SPEC-083 D-37).
//!
//! Arm scores arrive on incompatible scales (cosine ∈ [-1,1], RRF ∈ (0, w/k],
//! PPR share, `ts_rank_cd`, min-max ∈ [0,1]). Comparing raw `f32` across arms
//! is meaningless — convert at the fusion boundary only.

use std::cmp::Ordering;

/// Scale tag for a retrieval score. Distinct variants must not be compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoreScale {
    /// Dense vector cosine similarity (typically after L2-normalize).
    Cosine,
    /// Personalized PageRank share / mass.
    PprShare,
    /// Reciprocal Rank Fusion contribution.
    Rrf,
    /// Min-max normalized to `[0, 1]` within one arm (fusion boundary).
    MinMax,
    /// PostgreSQL `ts_rank_cd` cover-density rank (not BM25).
    TsRankCd,
}

/// Score + scale pair. Cross-scale comparison is a hard error.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaledScore {
    value: f32,
    scale: ScoreScale,
}

/// Attempted comparison or arithmetic across mismatched [`ScoreScale`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleMismatch {
    pub left: ScoreScale,
    pub right: ScoreScale,
}

impl std::fmt::Display for ScaleMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot compare scores across scales ({:?} vs {:?})",
            self.left, self.right
        )
    }
}

impl std::error::Error for ScaleMismatch {}

impl ScaledScore {
    pub fn new(value: f32, scale: ScoreScale) -> Self {
        Self { value, scale }
    }

    pub fn value(self) -> f32 {
        self.value
    }

    pub fn scale(self) -> ScoreScale {
        self.scale
    }

    /// Same-scale partial compare only.
    pub fn partial_cmp_compatible(&self, other: &Self) -> Result<Option<Ordering>, ScaleMismatch> {
        if self.scale != other.scale {
            return Err(ScaleMismatch {
                left: self.scale,
                right: other.scale,
            });
        }
        Ok(self.value.partial_cmp(&other.value))
    }
}

/// Min-max normalize a single arm's scores into [`ScoreScale::MinMax`].
///
/// Call at the Mix fusion boundary before combining arms (D-35/D-37).
pub fn min_max_normalize_to_fusion_scale(raw: &[f32]) -> Vec<ScaledScore> {
    if raw.is_empty() {
        return Vec::new();
    }
    let (min, max) = raw
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(mn, mx), &v| {
            (mn.min(v), mx.max(v))
        });
    let range = max - min;
    raw.iter()
        .map(|&v| {
            let norm = if range <= 0.0 { 1.0 } else { (v - min) / range };
            ScaledScore::new(norm, ScoreScale::MinMax)
        })
        .collect()
}

/// Weight a MinMax score at the fusion boundary. Rejects non-MinMax input.
pub fn weighted_minmax_contribution(
    score: ScaledScore,
    weight: f32,
) -> Result<ScaledScore, ScaleMismatch> {
    if score.scale != ScoreScale::MinMax {
        return Err(ScaleMismatch {
            left: score.scale,
            right: ScoreScale::MinMax,
        });
    }
    Ok(ScaledScore::new(weight * score.value, ScoreScale::MinMax))
}

/// Max of two MinMax contributions (D-35 Mix semantics). Cross-scale → error.
pub fn max_minmax(a: ScaledScore, b: ScaledScore) -> Result<ScaledScore, ScaleMismatch> {
    if a.scale != ScoreScale::MinMax || b.scale != ScoreScale::MinMax {
        return Err(ScaleMismatch {
            left: a.scale,
            right: b.scale,
        });
    }
    Ok(ScaledScore::new(a.value.max(b.value), ScoreScale::MinMax))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_score_scale_no_cross_compare() {
        let cosine = ScaledScore::new(0.9, ScoreScale::Cosine);
        let rrf = ScaledScore::new(0.02, ScoreScale::Rrf);
        let err = cosine
            .partial_cmp_compatible(&rrf)
            .expect_err("cross-scale compare must fail");
        assert_eq!(err.left, ScoreScale::Cosine);
        assert_eq!(err.right, ScoreScale::Rrf);

        let a = ScaledScore::new(0.5, ScoreScale::MinMax);
        let b = ScaledScore::new(0.8, ScoreScale::MinMax);
        assert_eq!(a.partial_cmp_compatible(&b).unwrap(), Some(Ordering::Less));

        let norm = min_max_normalize_to_fusion_scale(&[0.1, 0.5, 0.9]);
        assert!(norm.iter().all(|s| s.scale() == ScoreScale::MinMax));
        assert!((norm[0].value() - 0.0).abs() < 1e-5);
        assert!((norm[2].value() - 1.0).abs() < 1e-5);

        assert!(weighted_minmax_contribution(cosine, 1.0).is_err());
        let contrib = weighted_minmax_contribution(a, 2.0).unwrap();
        assert!((contrib.value() - 1.0).abs() < 1e-5);

        assert!(max_minmax(cosine, b).is_err());
        let m = max_minmax(a, b).unwrap();
        assert!((m.value() - 0.8).abs() < 1e-5);
    }
}
