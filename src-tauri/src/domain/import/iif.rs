//! QuickBooks IIF (tab-separated) parsing for list sections.

use super::{ImportBatch, ParsedAccount, ParsedContact, ParsedItem};

pub(crate) fn parse_iif(content: &str) -> Result<(ImportBatch, usize, Vec<String>), String> {
    let mut batch = ImportBatch::default();
    let mut skipped = 0usize;
    let mut warnings: Vec<String> = Vec::new();

    #[derive(Clone)]
    enum Section {
        Accnt,
        Cust,
        Vend,
        Invitem,
        Skip,
    }

    let mut current: Option<(Section, Vec<String>)> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r').trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').map(|s| s.trim()).collect();
        if parts.is_empty() {
            continue;
        }

        let head = parts[0];

        if head.starts_with('!') {
            let sec_name = head.trim_start_matches('!').to_uppercase();
            let headers: Vec<String> = parts[1..].iter().map(|s| (*s).to_string()).collect();
            let sec = match sec_name.as_str() {
                "ACCNT" => Section::Accnt,
                "CUST" => Section::Cust,
                "VEND" => Section::Vend,
                "INVITEM" => Section::Invitem,
                _ => Section::Skip,
            };
            current = Some((sec, headers));
            continue;
        }

        let Some((ref sec, ref hdr)) = current else {
            if matches!(
                head.to_uppercase().as_str(),
                "TRNS" | "SPL" | "ENDTRNS" | "TIMEACT"
            ) {
                skipped += 1;
            }
            continue;
        };

        match sec {
            Section::Accnt if head.eq_ignore_ascii_case("ACCNT") => {
                if let Some(a) = parse_accnt_row(hdr, &parts) {
                    batch.accounts.push(a);
                } else {
                    skipped += 1;
                }
            }
            Section::Cust if head.eq_ignore_ascii_case("CUST") => {
                if let Some(c) = parse_cust_row(hdr, &parts) {
                    batch.customers.push(c);
                } else {
                    skipped += 1;
                }
            }
            Section::Vend if head.eq_ignore_ascii_case("VEND") => {
                if let Some(v) = parse_vend_row(hdr, &parts) {
                    batch.vendors.push(v);
                } else {
                    skipped += 1;
                }
            }
            Section::Invitem if head.eq_ignore_ascii_case("INVITEM") => {
                if let Some(it) = parse_invitem_row(hdr, &parts) {
                    batch.items.push(it);
                } else {
                    skipped += 1;
                }
            }
            Section::Skip => {}
            _ => {
                skipped += 1;
            }
        }
    }

    if skipped > 0 && warnings.is_empty() {
        warnings.push(format!(
            "{skipped} row(s) skipped (unsupported sections or malformed lines)."
        ));
    }

    Ok((batch, skipped, warnings))
}

fn header_index(headers: &[String], aliases: &[&str]) -> Option<usize> {
    for (i, h) in headers.iter().enumerate() {
        let n = normalize_header(h);
        for a in aliases {
            if n == normalize_header(a) {
                return Some(i);
            }
        }
    }
    None
}

fn normalize_header(s: &str) -> String {
    s.trim()
        .to_uppercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn parse_accnt_row(headers: &[String], parts: &[&str]) -> Option<ParsedAccount> {
    let ni = header_index(headers, &["NAME"])?;
    let ti = header_index(headers, &["ACCNTTYPE", "TYPE"])?;
    let name = parts.get(ni + 1)?.trim();
    if name.is_empty() {
        return None;
    }
    let accnt_type = parts.get(ti + 1).unwrap_or(&"").trim();
    let (account_type, is_bank) = map_qb_accnt_type(accnt_type);

    let code = header_index(headers, &["ACCNUM", "ACCNO"])
        .and_then(|i| parts.get(i + 1).map(|s| s.trim()))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| slug_code(name));

    Some(ParsedAccount {
        code,
        name: name.to_string(),
        account_type: account_type.into(),
        is_bank_cash: is_bank,
    })
}

fn parse_cust_row(headers: &[String], parts: &[&str]) -> Option<ParsedContact> {
    let ni = header_index(headers, &["NAME", "FULLNAME"])?;
    let name = parts.get(ni + 1)?.trim();
    if name.is_empty() {
        return None;
    }
    let email = header_index(headers, &["EMAIL"])
        .and_then(|i| parts.get(i + 1))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let phone = header_index(headers, &["PHONE", "PHONE1", "WORKPHONE"])
        .and_then(|i| parts.get(i + 1))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some(ParsedContact {
        display_name: name.to_string(),
        email,
        phone,
    })
}

fn parse_vend_row(headers: &[String], parts: &[&str]) -> Option<ParsedContact> {
    let ni = header_index(headers, &["NAME", "FULLNAME", "COMPANYNAME"])?;
    let name = parts.get(ni + 1)?.trim();
    if name.is_empty() {
        return None;
    }
    let email = header_index(headers, &["EMAIL"])
        .and_then(|i| parts.get(i + 1))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let phone = header_index(headers, &["PHONE", "PHONE1"])
        .and_then(|i| parts.get(i + 1))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some(ParsedContact {
        display_name: name.to_string(),
        email,
        phone,
    })
}

fn parse_invitem_row(headers: &[String], parts: &[&str]) -> Option<ParsedItem> {
    let ni = header_index(headers, &["NAME"])?;
    let name = parts.get(ni + 1)?.trim();
    if name.is_empty() {
        return None;
    }
    let price_s = header_index(headers, &["PRICE", "SALESPRICE"])
        .and_then(|i| parts.get(i + 1))
        .unwrap_or(&"");
    let unit_price_minor = parse_money_minor(price_s).unwrap_or(0);

    let income_account_name = header_index(headers, &["ACCNT", "INCACCNT", "INCOMEACCOUNT"])
        .and_then(|i| parts.get(i + 1))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Some(ParsedItem {
        name: name.to_string(),
        unit_price_minor,
        income_account_name,
    })
}

fn map_qb_accnt_type(t: &str) -> (&'static str, bool) {
    let u = t.trim().to_uppercase();
    let bank = matches!(
        u.as_str(),
        "BANK" | "UNDEPOSITEDFUNDS" | "UNDEPOSITED" | "SWIMMINGPOOLBANK"
    );
    let kind = match u.as_str() {
        "BANK" | "OCASSET" | "AR" | "INVASSET" | "FIXASSET" | "OASSET" | "OTHERASSET"
        | "UNDEPOSITEDFUNDS" | "UNDEPOSITED" => "asset",
        "AP" | "CREDITCARD" | "OCLIAB" | "LTLIAB" | "LONGTERMLIABILITY" => "liability",
        "EQUITY" => "equity",
        "INC" | "OINC" => "income",
        _ => "expense",
    };
    (kind, bank)
}

pub(crate) fn slug_code(name: &str) -> String {
    let mut s: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(16)
        .collect();
    if s.is_empty() {
        s = "QB".into();
    }
    format!("QB-{s}")
}

pub(crate) fn parse_money_minor(s: &str) -> Option<i64> {
    let mut t = s.trim().replace(['$', ',', ' '], "");
    let neg = t.starts_with('-');
    if neg {
        t = t.trim_start_matches('-').to_string();
    }
    if t.is_empty() {
        return None;
    }
    let parts: Vec<&str> = t.split('.').collect();
    let whole: i64 = parts.first()?.parse().ok()?;
    let frac = parts.get(1).map(|f| *f).unwrap_or("0");
    let frac_s: String = frac.chars().take(2).collect();
    let frac_s = format!("{frac_s:0<2}");
    let frac_n: i64 = frac_s.parse().ok()?;
    let v = whole * 100 + frac_n;
    Some(if neg { -v } else { v })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_iif_accnt_and_invitem() {
        let s = "!ACCNT\tNAME\tACCNTTYPE\tACCNUM\nACCNT\tChecking\tBANK\t101\n!INVITEM\tNAME\tPRICE\tACCNT\nINVITEM\tWidget\t12.50\tSales\n";
        let (b, _, _) = parse_iif(s).expect("parse");
        assert_eq!(b.accounts.len(), 1);
        assert_eq!(b.accounts[0].code, "101");
        assert_eq!(b.accounts[0].account_type, "asset");
        assert!(b.accounts[0].is_bank_cash);
        assert_eq!(b.items.len(), 1);
        assert_eq!(b.items[0].unit_price_minor, 1250);
    }
}
