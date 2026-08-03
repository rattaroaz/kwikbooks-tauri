//! Integer-safe monetary helpers (minor units / cents).

/// Quantity scale: 6 decimal places (micro-units).
const QTY_SCALE: i128 = 1_000_000;

/// Line total in minor units from quantity × unit price.
///
/// Quantity is quantized to 6 decimal places, then multiplied in integer
/// arithmetic and rounded half-away-from-zero. Negative or non-finite results
/// clamp to 0 (same contract as the previous f64 helper).
pub fn line_total_minor(qty: f64, unit_minor: i64) -> i64 {
    if !qty.is_finite() {
        return 0;
    }
    let qty_scaled = (qty * QTY_SCALE as f64).round() as i128;
    let product = qty_scaled.saturating_mul(i128::from(unit_minor));
    let half = QTY_SCALE / 2;
    let rounded = if product >= 0 {
        (product + half) / QTY_SCALE
    } else {
        (product - half) / QTY_SCALE
    };
    if rounded <= 0 {
        0
    } else if rounded > i128::from(i64::MAX) {
        i64::MAX
    } else {
        rounded as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_quantity_exact() {
        assert_eq!(line_total_minor(2.0, 1250), 2500);
    }

    #[test]
    fn fractional_quantity_half_up() {
        // 0.1 × 33 → 3.3 → 3
        assert_eq!(line_total_minor(0.1, 33), 3);
        // 1.5 × 100 → 150
        assert_eq!(line_total_minor(1.5, 100), 150);
    }

    #[test]
    fn clamps_negative_and_non_finite() {
        assert_eq!(line_total_minor(-1.0, 100), 0);
        assert_eq!(line_total_minor(f64::NAN, 100), 0);
        assert_eq!(line_total_minor(1.0, -50), 0);
    }
}
