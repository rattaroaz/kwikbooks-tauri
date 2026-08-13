use crate::db::DbCommandError;

/// Require a calendar date as `YYYY-MM-DD` (strict zero-padded).
pub fn require_iso_date(label: &str, value: &str) -> Result<(), DbCommandError> {
    let t = value.trim();
    let b = t.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return Err(DbCommandError::Validation {
            message: format!("{label} must be a valid date (YYYY-MM-DD)"),
        });
    }
    let Ok(y) = t[0..4].parse::<i32>() else {
        return Err(invalid_date(label));
    };
    let Ok(m) = t[5..7].parse::<u32>() else {
        return Err(invalid_date(label));
    };
    let Ok(d) = t[8..10].parse::<u32>() else {
        return Err(invalid_date(label));
    };
    if m == 0 || m > 12 || d == 0 || d > days_in_month(y, m) {
        return Err(invalid_date(label));
    }
    Ok(())
}

fn invalid_date(label: &str) -> DbCommandError {
    DbCommandError::Validation {
        message: format!("{label} must be a valid date (YYYY-MM-DD)"),
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_iso_dates() {
        require_iso_date("d", "2026-01-05").unwrap();
        require_iso_date("d", "2024-02-29").unwrap();
    }

    #[test]
    fn rejects_unpadded_and_invalid() {
        assert!(require_iso_date("d", "2026-1-5").is_err());
        assert!(require_iso_date("d", "2025-02-29").is_err());
        assert!(require_iso_date("d", "not-a-date").is_err());
    }
}
