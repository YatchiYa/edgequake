//! Weighted virtual-runtime foundations for SPEC-120 P2.
//!
//! Claim SQL in Postgres optionally ranks by `tenant_vruntime` (ascending).
//! These pure helpers define the charging rule the ledger adapter persists.

/// Charge completed service to a tenant lane.
///
/// A lane with weight `2.0` accrues half the virtual runtime of a lane with
/// weight `1.0` for the same work. Invalid/non-positive values are ignored so
/// an operational accounting defect cannot poison the ledger with NaN/∞.
pub fn charge_vruntime(current: f64, service_units: f64, weight: f64) -> f64 {
    if !current.is_finite()
        || !service_units.is_finite()
        || service_units <= 0.0
        || !weight.is_finite()
        || weight <= 0.0
    {
        return current;
    }
    current + service_units / weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charge_is_inverse_to_lane_weight() {
        assert_eq!(charge_vruntime(10.0, 4.0, 1.0), 14.0);
        assert_eq!(charge_vruntime(10.0, 4.0, 2.0), 12.0);
    }

    #[test]
    fn invalid_charge_inputs_do_not_corrupt_ledger() {
        assert_eq!(charge_vruntime(7.0, 1.0, 0.0), 7.0);
        assert_eq!(charge_vruntime(7.0, -1.0, 1.0), 7.0);
        assert_eq!(charge_vruntime(7.0, f64::NAN, 1.0), 7.0);
    }
}
