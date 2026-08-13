use rusqlite::Connection;

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;

pub fn customers_list(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT id, display_name, email, phone, terms_days, is_active, created_at
           FROM customer WHERE company_id = ?1 ORDER BY display_name"#,
    )?;
    let rows = stmt.query_map([COMPANY_ID], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "displayName": row.get::<_, String>(1)?,
            "email": row.get::<_, Option<String>>(2)?,
            "phone": row.get::<_, Option<String>>(3)?,
            "termsDays": row.get::<_, i64>(4)?,
            "isActive": row.get::<_, i64>(5)? == 1,
            "createdAt": row.get::<_, String>(6)?,
        }))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn vendors_list(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT id, display_name, email, phone, is_active, created_at
           FROM vendor WHERE company_id = ?1 ORDER BY display_name"#,
    )?;
    let rows = stmt.query_map([COMPANY_ID], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "displayName": row.get::<_, String>(1)?,
            "email": row.get::<_, Option<String>>(2)?,
            "phone": row.get::<_, Option<String>>(3)?,
            "isActive": row.get::<_, i64>(4)? == 1,
            "createdAt": row.get::<_, String>(5)?,
        }))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn invoices_list(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT i.id, i.number, i.status, i.customer_id, c.display_name, i.issue_date, i.due_date,
                  i.subtotal_minor, i.tax_minor, i.total_minor, i.journal_id, i.memo
           FROM invoice i
           JOIN customer c ON c.id = i.customer_id
           WHERE i.company_id = ?1
           ORDER BY i.issue_date DESC, i.id DESC"#,
    )?;
    let rows = stmt.query_map([COMPANY_ID], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "number": row.get::<_, String>(1)?,
            "status": row.get::<_, String>(2)?,
            "customerId": row.get::<_, i64>(3)?,
            "customerName": row.get::<_, String>(4)?,
            "issueDate": row.get::<_, String>(5)?,
            "dueDate": row.get::<_, Option<String>>(6)?,
            "subtotalMinor": row.get::<_, i64>(7)?,
            "taxMinor": row.get::<_, i64>(8)?,
            "totalMinor": row.get::<_, i64>(9)?,
            "journalId": row.get::<_, Option<i64>>(10)?,
            "memo": row.get::<_, Option<String>>(11)?,
        }))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn bills_list(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT b.id, b.number, b.status, b.vendor_id, v.display_name, b.payee_name, b.issue_date, b.due_date,
                  b.total_minor, b.journal_id, b.memo
           FROM bill b
           LEFT JOIN vendor v ON v.id = b.vendor_id
           WHERE b.company_id = ?1
           ORDER BY b.issue_date DESC, b.id DESC"#,
    )?;
    let rows = stmt.query_map([COMPANY_ID], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "number": row.get::<_, String>(1)?,
            "status": row.get::<_, String>(2)?,
            "vendorId": row.get::<_, Option<i64>>(3)?,
            "vendorName": row.get::<_, Option<String>>(4)?,
            "payeeName": row.get::<_, Option<String>>(5)?,
            "issueDate": row.get::<_, String>(6)?,
            "dueDate": row.get::<_, Option<String>>(7)?,
            "totalMinor": row.get::<_, i64>(8)?,
            "journalId": row.get::<_, Option<i64>>(9)?,
            "memo": row.get::<_, Option<String>>(10)?,
        }))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn journals_list(conn: &Connection, limit: i64) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let lim = limit.clamp(1, 2000);
    let mut stmt = conn.prepare(
        r#"SELECT j.id, j.entry_date, j.memo, j.source_kind, j.source_id, j.created_at
           FROM journal j
           WHERE j.company_id = ?1
           ORDER BY j.entry_date DESC, j.id DESC
           LIMIT ?2"#,
    )?;
    let rows = stmt.query_map(rusqlite::params![COMPANY_ID, lim], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "entryDate": row.get::<_, String>(1)?,
            "memo": row.get::<_, Option<String>>(2)?,
            "sourceKind": row.get::<_, Option<String>>(3)?,
            "sourceId": row.get::<_, Option<i64>>(4)?,
            "createdAt": row.get::<_, String>(5)?,
        }))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn invoice_get(conn: &Connection, invoice_id: i64) -> Result<serde_json::Value, DbCommandError> {
    let header = conn.query_row(
        r#"SELECT i.id, i.number, i.status, i.customer_id, c.display_name, i.issue_date, i.due_date,
                  i.memo, i.subtotal_minor, i.tax_minor, i.total_minor, i.journal_id
           FROM invoice i JOIN customer c ON c.id = i.customer_id
           WHERE i.id = ?1 AND i.company_id = ?2"#,
        rusqlite::params![invoice_id, COMPANY_ID],
        |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "number": row.get::<_, String>(1)?,
                "status": row.get::<_, String>(2)?,
                "customerId": row.get::<_, i64>(3)?,
                "customerName": row.get::<_, String>(4)?,
                "issueDate": row.get::<_, String>(5)?,
                "dueDate": row.get::<_, Option<String>>(6)?,
                "memo": row.get::<_, Option<String>>(7)?,
                "subtotalMinor": row.get::<_, i64>(8)?,
                "taxMinor": row.get::<_, i64>(9)?,
                "totalMinor": row.get::<_, i64>(10)?,
                "journalId": row.get::<_, Option<i64>>(11)?,
            }))
        },
    );

    let header = header.map_err(|_| DbCommandError::NotFound {
        entity: "invoice".into(),
        id: invoice_id,
    })?;

    let mut stmt = conn.prepare(
        r#"SELECT line_number, item_id, description, quantity, unit_price_minor, line_total_minor, income_account_id
           FROM invoice_line WHERE invoice_id = ?1 ORDER BY line_number"#,
    )?;
    let rows = stmt.query_map([invoice_id], |row| {
        Ok(serde_json::json!({
            "lineNumber": row.get::<_, i32>(0)?,
            "itemId": row.get::<_, Option<i64>>(1)?,
            "description": row.get::<_, String>(2)?,
            "quantity": row.get::<_, f64>(3)?,
            "unitPriceMinor": row.get::<_, i64>(4)?,
            "lineTotalMinor": row.get::<_, i64>(5)?,
            "incomeAccountId": row.get::<_, Option<i64>>(6)?,
        }))
    })?;

    let mut lines = Vec::new();
    for r in rows {
        lines.push(r?);
    }

    Ok(serde_json::json!({
        "header": header,
        "lines": lines,
    }))
}

pub fn bill_get(conn: &Connection, bill_id: i64) -> Result<serde_json::Value, DbCommandError> {
    let header = conn
        .query_row(
            r#"SELECT b.id, b.number, b.status, b.vendor_id, v.display_name, b.payee_name, b.issue_date, b.due_date,
                      b.memo, b.total_minor, b.journal_id
               FROM bill b LEFT JOIN vendor v ON v.id = b.vendor_id
               WHERE b.id = ?1 AND b.company_id = ?2"#,
            rusqlite::params![bill_id, COMPANY_ID],
            |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "number": row.get::<_, String>(1)?,
                    "status": row.get::<_, String>(2)?,
                    "vendorId": row.get::<_, Option<i64>>(3)?,
                    "vendorName": row.get::<_, Option<String>>(4)?,
                    "payeeName": row.get::<_, Option<String>>(5)?,
                    "issueDate": row.get::<_, String>(6)?,
                    "dueDate": row.get::<_, Option<String>>(7)?,
                    "memo": row.get::<_, Option<String>>(8)?,
                    "totalMinor": row.get::<_, i64>(9)?,
                    "journalId": row.get::<_, Option<i64>>(10)?,
                }))
            },
        )
        .map_err(|_| DbCommandError::NotFound {
            entity: "bill".into(),
            id: bill_id,
        })?;

    let mut stmt = conn.prepare(
        r#"SELECT line_number, description, amount_minor, expense_account_id
           FROM bill_line WHERE bill_id = ?1 ORDER BY line_number"#,
    )?;
    let rows = stmt.query_map([bill_id], |row| {
        Ok(serde_json::json!({
            "lineNumber": row.get::<_, i32>(0)?,
            "description": row.get::<_, String>(1)?,
            "amountMinor": row.get::<_, i64>(2)?,
            "expenseAccountId": row.get::<_, i64>(3)?,
        }))
    })?;

    let mut lines = Vec::new();
    for r in rows {
        lines.push(r?);
    }

    Ok(serde_json::json!({
        "header": header,
        "lines": lines,
    }))
}

pub fn vendor_payments_list(conn: &Connection) -> Result<Vec<serde_json::Value>, DbCommandError> {
    let mut stmt = conn.prepare(
        r#"SELECT vp.id, vp.vendor_id, v.display_name, vp.bank_account_id, vp.payment_date,
                  vp.amount_minor, vp.memo, vp.bill_id, vp.journal_id,
                  vp.check_number, vp.payment_method, vp.payee_name, vp.check_printed_at
           FROM vendor_payment vp
           JOIN vendor v ON v.id = vp.vendor_id
           WHERE vp.company_id = ?1
           ORDER BY vp.payment_date DESC, vp.id DESC"#,
    )?;
    let rows = stmt.query_map([COMPANY_ID], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "vendorId": row.get::<_, i64>(1)?,
            "vendorName": row.get::<_, String>(2)?,
            "bankAccountId": row.get::<_, i64>(3)?,
            "paymentDate": row.get::<_, String>(4)?,
            "amountMinor": row.get::<_, i64>(5)?,
            "memo": row.get::<_, Option<String>>(6)?,
            "billId": row.get::<_, Option<i64>>(7)?,
            "journalId": row.get::<_, Option<i64>>(8)?,
            "checkNumber": row.get::<_, Option<String>>(9)?,
            "paymentMethod": row.get::<_, String>(10)?,
            "payeeName": row.get::<_, Option<String>>(11)?,
            "checkPrintedAt": row.get::<_, Option<String>>(12)?,
        }))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn vendor_payment_get(conn: &Connection, payment_id: i64) -> Result<serde_json::Value, DbCommandError> {
    conn.query_row(
        r#"SELECT vp.id, vp.vendor_id, v.display_name, vp.bank_account_id, a.code, a.name,
                  vp.payment_date, vp.amount_minor, vp.memo, vp.bill_id, vp.journal_id,
                  vp.check_number, vp.payment_method, vp.payee_name, vp.check_printed_at
           FROM vendor_payment vp
           JOIN vendor v ON v.id = vp.vendor_id
           JOIN account a ON a.id = vp.bank_account_id
           WHERE vp.id = ?1 AND vp.company_id = ?2"#,
        rusqlite::params![payment_id, COMPANY_ID],
        |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "vendorId": row.get::<_, i64>(1)?,
                "vendorName": row.get::<_, String>(2)?,
                "bankAccountId": row.get::<_, i64>(3)?,
                "bankAccountCode": row.get::<_, String>(4)?,
                "bankAccountName": row.get::<_, String>(5)?,
                "paymentDate": row.get::<_, String>(6)?,
                "amountMinor": row.get::<_, i64>(7)?,
                "memo": row.get::<_, Option<String>>(8)?,
                "billId": row.get::<_, Option<i64>>(9)?,
                "journalId": row.get::<_, Option<i64>>(10)?,
                "checkNumber": row.get::<_, Option<String>>(11)?,
                "paymentMethod": row.get::<_, String>(12)?,
                "payeeName": row.get::<_, Option<String>>(13)?,
                "checkPrintedAt": row.get::<_, Option<String>>(14)?,
            }))
        },
    )
    .map_err(|_| DbCommandError::NotFound {
        entity: "vendor_payment".into(),
        id: payment_id,
    })
}
