use rusqlite::Transaction;

use crate::db::DbCommandError;

#[derive(Debug, Clone)]
pub struct DraftJournalLine {
    pub account_id: i64,
    pub line_number: i32,
    pub description: Option<String>,
    pub debit_minor: i64,
    pub credit_minor: i64,
}

pub fn validate_balance(lines: &[DraftJournalLine]) -> Result<(), DbCommandError> {
    let mut dr: i64 = 0;
    let mut cr: i64 = 0;
    for l in lines {
        if l.debit_minor < 0 || l.credit_minor < 0 {
            return Err(DbCommandError::Validation {
                message: "debit/credit must be non-negative".into(),
            });
        }
        if l.debit_minor > 0 && l.credit_minor > 0 {
            return Err(DbCommandError::Validation {
                message: "a journal line cannot have both debit and credit".into(),
            });
        }
        if l.debit_minor == 0 && l.credit_minor == 0 {
            return Err(DbCommandError::Validation {
                message: "a journal line must have debit or credit".into(),
            });
        }
        dr = dr.saturating_add(l.debit_minor);
        cr = cr.saturating_add(l.credit_minor);
    }
    if dr != cr {
        return Err(DbCommandError::Invariant {
            message: format!("journal not balanced: debits {dr} credits {cr}"),
        });
    }
    Ok(())
}

pub fn insert_journal(
    tx: &Transaction<'_>,
    company_id: i64,
    entry_date: &str,
    memo: Option<&str>,
    source_kind: &str,
    source_id: i64,
    lines: &[DraftJournalLine],
) -> Result<i64, DbCommandError> {
    validate_balance(lines)?;
    tx.execute(
        "INSERT INTO journal (company_id, entry_date, memo, source_kind, source_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![company_id, entry_date, memo, source_kind, source_id],
    )?;
    let jid = tx.last_insert_rowid();
    for l in lines {
        tx.execute(
            "INSERT INTO journal_line (journal_id, account_id, line_number, description, debit_minor, credit_minor) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                jid,
                l.account_id,
                l.line_number,
                l.description,
                l.debit_minor,
                l.credit_minor
            ],
        )?;
    }
    Ok(jid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_balance_ok() {
        let lines = vec![
            DraftJournalLine {
                account_id: 1,
                line_number: 1,
                description: None,
                debit_minor: 100,
                credit_minor: 0,
            },
            DraftJournalLine {
                account_id: 2,
                line_number: 2,
                description: None,
                debit_minor: 0,
                credit_minor: 100,
            },
        ];
        assert!(validate_balance(&lines).is_ok());
    }

    #[test]
    fn validate_balance_rejects_mismatch() {
        let lines = vec![
            DraftJournalLine {
                account_id: 1,
                line_number: 1,
                description: None,
                debit_minor: 100,
                credit_minor: 0,
            },
            DraftJournalLine {
                account_id: 2,
                line_number: 2,
                description: None,
                debit_minor: 0,
                credit_minor: 99,
            },
        ];
        assert!(validate_balance(&lines).is_err());
    }

    #[test]
    fn validate_balance_rejects_both_debit_and_credit() {
        let lines = vec![DraftJournalLine {
            account_id: 1,
            line_number: 1,
            description: None,
            debit_minor: 10,
            credit_minor: 10,
        }];
        assert!(validate_balance(&lines).is_err());
    }

    #[test]
    fn validate_balance_rejects_negative_debit() {
        let lines = vec![
            DraftJournalLine {
                account_id: 1,
                line_number: 1,
                description: None,
                debit_minor: -1,
                credit_minor: 0,
            },
            DraftJournalLine {
                account_id: 2,
                line_number: 2,
                description: None,
                debit_minor: 0,
                credit_minor: 1,
            },
        ];
        assert!(validate_balance(&lines).is_err());
    }

    #[test]
    fn validate_balance_rejects_zero_line() {
        let lines = vec![DraftJournalLine {
            account_id: 1,
            line_number: 1,
            description: None,
            debit_minor: 0,
            credit_minor: 0,
        }];
        assert!(validate_balance(&lines).is_err());
    }
}
