//! Acc@N regression floors (SPEC-083 X-35).
//!
//! Publishes the honest Acc@N curve and fails when a measured Acc@40 fixture
//! drops below the agreed regression floor. This is **not** a live LLM bench —
//! it gates the published/fixture curve so marketing Acc@5 cannot silently
//! replace Acc@40.

use serde::Deserialize;

/// One point on the Acc@N / F1@N curve.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AccAtNPoint {
    pub n_docs: usize,
    pub acc: f64,
    #[serde(default)]
    pub f1: f64,
}

/// Published Acc@N floors fixture.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AccAtNFloors {
    pub source: String,
    pub disclaimer: String,
    pub curve: Vec<AccAtNPoint>,
    pub regression_floor_acc_at_40: f64,
    pub measured_fixture_acc_at_40: f64,
}

/// Load embedded Acc@N floors JSON.
pub fn load_acc_at_n_floors() -> AccAtNFloors {
    serde_json::from_str(include_str!("../../tests/fixtures/acc_at_n_floors.json"))
        .expect("acc_at_n_floors.json must parse")
}

impl AccAtNFloors {
    /// Acc at a given document count (exact n_docs match).
    pub fn acc_at(&self, n_docs: usize) -> Option<f64> {
        self.curve
            .iter()
            .find(|p| p.n_docs == n_docs)
            .map(|p| p.acc)
    }

    /// True when Acc@N is non-increasing as N grows (honest degradation curve).
    pub fn is_monotone_non_increasing(&self) -> bool {
        let mut prev = f64::INFINITY;
        for p in &self.curve {
            if p.acc > prev + 1e-9 {
                return false;
            }
            prev = p.acc;
        }
        true
    }
}

/// Gate: measured Acc@40 fixture must stay ≥ regression floor; curve documented.
pub fn evaluate_acc_at_n_regression(floors: &AccAtNFloors) -> Result<(), String> {
    let acc40 = floors
        .acc_at(40)
        .ok_or_else(|| "curve missing Acc@40 point".to_string())?;
    if acc40 + 1e-9 < floors.regression_floor_acc_at_40 {
        return Err(format!(
            "published Acc@40 {acc40} below floor {}",
            floors.regression_floor_acc_at_40
        ));
    }
    if floors.measured_fixture_acc_at_40 + 1e-9 < floors.regression_floor_acc_at_40 {
        return Err(format!(
            "measured fixture Acc@40 {} below floor {}",
            floors.measured_fixture_acc_at_40, floors.regression_floor_acc_at_40
        ));
    }
    let acc5 = floors
        .acc_at(5)
        .ok_or_else(|| "curve missing Acc@5 point".to_string())?;
    if acc5 + 1e-9 < acc40 {
        return Err("Acc@5 < Acc@40 — curve fixture inverted".to_string());
    }
    if !floors.disclaimer.to_lowercase().contains("acc@5") {
        return Err("disclaimer must warn against quoting Acc@5".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// X-35: Acc@N regression gate against published floors JSON.
    #[test]
    fn bench_acc_at_n_regression_gate() {
        let floors = load_acc_at_n_floors();
        assert!(
            floors.is_monotone_non_increasing(),
            "published Acc@N curve must be monotone non-increasing"
        );
        evaluate_acc_at_n_regression(&floors).expect("Acc@N regression gate");
        assert!(
            floors.measured_fixture_acc_at_40 >= floors.regression_floor_acc_at_40,
            "measured Acc@40 fixture must respect floor"
        );
    }
}
