//! Cross-entity text search (LIKE) for navigation and discovery.

use rusqlite::Connection;

use crate::db::DbCommandError;
use crate::domain::constants::COMPANY_ID;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: String,
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub path: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSearchResponse {
    pub query: String,
    pub hits: Vec<SearchHit>,
}

/// Escape `%`, `_`, `\` for SQLite `LIKE ... ESCAPE '\'`.
pub fn sql_like_contains(query: &str) -> String {
    let mut out = String::from("%");
    for c in query.chars() {
        match c {
            '\\' | '%' | '_' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out.push('%');
    out
}

const MAX_HITS: usize = 200;

fn append_hits<P: rusqlite::Params>(
    conn: &Connection,
    hits: &mut Vec<SearchHit>,
    sql: &str,
    params: P,
) -> Result<(), rusqlite::Error> {
    if hits.len() >= MAX_HITS {
        return Ok(());
    }
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params, |row| {
        Ok(SearchHit {
            kind: row.get(0)?,
            id: row.get(1)?,
            title: row.get(2)?,
            subtitle: row.get(3)?,
            path: row.get(4)?,
        })
    })?;
    for r in rows {
        if hits.len() >= MAX_HITS {
            break;
        }
        hits.push(r?);
    }
    Ok(())
}

pub fn global_search(
    conn: &Connection,
    query: &str,
    limit_per_category: i64,
) -> Result<GlobalSearchResponse, DbCommandError> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(GlobalSearchResponse {
            query: query.into(),
            hits: Vec::new(),
        });
    }

    let lim = limit_per_category.clamp(1, 40);
    let pat = sql_like_contains(q);
    let esc = "\\";

    let mut hits: Vec<SearchHit> = Vec::new();

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'company', id,
             COALESCE(NULLIF(trim(legal_name), ''), name),
             CASE WHEN legal_name IS NOT NULL AND trim(legal_name) != '' THEN name ELSE NULL END,
             '/settings'
           FROM company WHERE id = ?1 AND (
             name LIKE ?2 ESCAPE ?3 OR (legal_name IS NOT NULL AND legal_name LIKE ?2 ESCAPE ?3)
           ) LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'account', id,
             code || ' · ' || name,
             account_type,
             '/accounts'
           FROM account WHERE company_id = ?1 AND is_active = 1 AND (
             code LIKE ?2 ESCAPE ?3 OR name LIKE ?2 ESCAPE ?3
           ) ORDER BY sort_order, code LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'customer', id, display_name,
             COALESCE(email, phone),
             '/customers'
           FROM customer WHERE company_id = ?1 AND is_active = 1 AND (
             display_name LIKE ?2 ESCAPE ?3 OR IFNULL(email,'') LIKE ?2 ESCAPE ?3
             OR IFNULL(phone,'') LIKE ?2 ESCAPE ?3 OR IFNULL(notes,'') LIKE ?2 ESCAPE ?3
           ) ORDER BY display_name LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'vendor', id, display_name,
             COALESCE(email, phone),
             '/vendors'
           FROM vendor WHERE company_id = ?1 AND is_active = 1 AND (
             display_name LIKE ?2 ESCAPE ?3 OR IFNULL(email,'') LIKE ?2 ESCAPE ?3
             OR IFNULL(phone,'') LIKE ?2 ESCAPE ?3 OR IFNULL(notes,'') LIKE ?2 ESCAPE ?3
           ) ORDER BY display_name LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'item', id, name,
             NULLIF(trim(COALESCE(sku,'')), ''),
             '/accounts'
           FROM item WHERE company_id = ?1 AND is_active = 1 AND (
             name LIKE ?2 ESCAPE ?3 OR IFNULL(sku,'') LIKE ?2 ESCAPE ?3
           ) ORDER BY name LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'invoice', i.id,
             i.number,
             trim(COALESCE(c.display_name,'') || ' · ' || COALESCE(i.issue_date,'')
               || CASE WHEN IFNULL(i.memo,'') != '' THEN ' · ' || substr(i.memo,1,80) ELSE '' END),
             '/invoices/' || i.id
           FROM invoice i
           JOIN customer c ON c.id = i.customer_id
           WHERE i.company_id = ?1 AND (
             i.number LIKE ?2 ESCAPE ?3 OR IFNULL(i.memo,'') LIKE ?2 ESCAPE ?3
             OR c.display_name LIKE ?2 ESCAPE ?3
             OR EXISTS (
               SELECT 1 FROM invoice_line il WHERE il.invoice_id = i.id
                 AND IFNULL(il.description,'') LIKE ?2 ESCAPE ?3
             )
           )
           ORDER BY i.issue_date DESC, i.id DESC LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'bill', b.id,
             b.number,
             trim(COALESCE(v.display_name, b.payee_name, '') || ' · ' || COALESCE(b.issue_date,'')
               || CASE WHEN IFNULL(b.memo,'') != '' THEN ' · ' || substr(b.memo,1,80) ELSE '' END),
             '/bills/' || b.id
           FROM bill b
           LEFT JOIN vendor v ON v.id = b.vendor_id
           WHERE b.company_id = ?1 AND (
             b.number LIKE ?2 ESCAPE ?3 OR IFNULL(b.memo,'') LIKE ?2 ESCAPE ?3
             OR IFNULL(b.payee_name,'') LIKE ?2 ESCAPE ?3
             OR IFNULL(v.display_name,'') LIKE ?2 ESCAPE ?3
             OR EXISTS (
               SELECT 1 FROM bill_line bl WHERE bl.bill_id = b.id
                 AND IFNULL(bl.description,'') LIKE ?2 ESCAPE ?3
             )
           )
           ORDER BY b.issue_date DESC, b.id DESC LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'journal', j.id,
             'Journal #' || j.id,
             trim(COALESCE(j.memo,'') || ' · ' || COALESCE(j.entry_date,'')),
             '/register'
           FROM journal j
           WHERE j.company_id = ?1 AND (
             IFNULL(j.memo,'') LIKE ?2 ESCAPE ?3 OR j.entry_date LIKE ?2 ESCAPE ?3
             OR CAST(j.id AS TEXT) LIKE ?2 ESCAPE ?3
             OR EXISTS (
               SELECT 1 FROM journal_line jl WHERE jl.journal_id = j.id
                 AND IFNULL(jl.description,'') LIKE ?2 ESCAPE ?3
             )
           )
           ORDER BY j.entry_date DESC, j.id DESC LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'payment_customer', cp.id,
             'Customer payment #' || cp.id,
             trim(COALESCE(c.display_name,'') || ' · ' || cp.payment_date
               || CASE WHEN IFNULL(cp.memo,'') != '' THEN ' · ' || substr(cp.memo,1,60) ELSE '' END),
             '/payments/receive'
           FROM customer_payment cp
           JOIN customer c ON c.id = cp.customer_id
           WHERE cp.company_id = ?1 AND (
             IFNULL(cp.memo,'') LIKE ?2 ESCAPE ?3 OR CAST(cp.amount_minor AS TEXT) LIKE ?2 ESCAPE ?3
             OR CAST(cp.id AS TEXT) LIKE ?2 ESCAPE ?3 OR c.display_name LIKE ?2 ESCAPE ?3
           )
           ORDER BY cp.payment_date DESC, cp.id DESC LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    append_hits(
        conn,
        &mut hits,
        r#"SELECT 'payment_vendor', vp.id,
             'Vendor payment #' || vp.id,
             trim(COALESCE(v.display_name,'') || ' · ' || vp.payment_date
               || CASE WHEN IFNULL(vp.memo,'') != '' THEN ' · ' || substr(vp.memo,1,60) ELSE '' END),
             CASE WHEN vp.payment_method = 'check'
               THEN '/checks/print/' || vp.id
               ELSE '/payments/pay'
             END
           FROM vendor_payment vp
           JOIN vendor v ON v.id = vp.vendor_id
           WHERE vp.company_id = ?1 AND (
             IFNULL(vp.memo,'') LIKE ?2 ESCAPE ?3 OR CAST(vp.amount_minor AS TEXT) LIKE ?2 ESCAPE ?3
             OR CAST(vp.id AS TEXT) LIKE ?2 ESCAPE ?3 OR v.display_name LIKE ?2 ESCAPE ?3
           )
           ORDER BY vp.payment_date DESC, vp.id DESC LIMIT ?4"#,
        rusqlite::params![COMPANY_ID, pat, esc, lim],
    )?;

    Ok(GlobalSearchResponse {
        query: q.to_string(),
        hits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_all;
    use tempfile::tempdir;

    #[test]
    fn global_search_finds_seed_account() {
        let dir = tempdir().expect("tmp");
        let db_path = dir.path().join("search.sqlite");
        run_all(&db_path).expect("migrate");
        let conn = rusqlite::Connection::open(&db_path).expect("open");
        let r = global_search(&conn, "Sales", 10).expect("search");
        assert!(
            r.hits.iter().any(|h| h.kind == "account" && h.title.contains("4000")),
            "{:?}",
            r.hits
        );
    }
}
