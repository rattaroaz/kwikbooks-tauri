//! Flexible CSV parsing for QuickBooks-style exports (comma or tab).

use csv::ReaderBuilder;

use super::{ImportBatch, ParsedAccount, ParsedContact, ParsedItem};
use super::iif::{parse_money_minor, slug_code};

pub(crate) fn parse_csv(content: &str) -> Result<(ImportBatch, usize, Vec<String>), String> {
    let delim = sniff_delimiter(content);
    let mut rdr = ReaderBuilder::new()
        .delimiter(delim)
        .flexible(true)
        .trim(csv::Trim::Fields)
        .from_reader(content.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    if headers.is_empty() {
        return Err("CSV has no header row.".into());
    }

    let profile = detect_profile(&headers);
    let mut batch = ImportBatch::default();
    let mut skipped = 0usize;
    let warnings: Vec<String> = Vec::new();

    for rec in rdr.records() {
        let rec = rec.map_err(|e| e.to_string())?;
        match profile {
            CsvProfile::Customers => {
                if let Some(c) = row_customer(&headers, &rec) {
                    batch.customers.push(c);
                } else {
                    skipped += 1;
                }
            }
            CsvProfile::Vendors => {
                if let Some(v) = row_vendor(&headers, &rec) {
                    batch.vendors.push(v);
                } else {
                    skipped += 1;
                }
            }
            CsvProfile::Accounts => {
                if let Some(a) = row_account(&headers, &rec) {
                    batch.accounts.push(a);
                } else {
                    skipped += 1;
                }
            }
            CsvProfile::Items => {
                if let Some(it) = row_item(&headers, &rec) {
                    batch.items.push(it);
                } else {
                    skipped += 1;
                }
            }
            CsvProfile::MixedLists => {
                if let Some(c) = row_customer(&headers, &rec) {
                    batch.customers.push(c);
                } else if let Some(v) = row_vendor(&headers, &rec) {
                    batch.vendors.push(v);
                } else {
                    skipped += 1;
                }
            }
        }
    }

    Ok((batch, skipped, warnings))
}

#[derive(Clone, Copy)]
enum CsvProfile {
    Customers,
    Vendors,
    Accounts,
    Items,
    MixedLists,
}

fn sniff_delimiter(content: &str) -> u8 {
    let first = content.lines().next().unwrap_or("");
    let commas = first.matches(',').count();
    let tabs = first.matches('\t').count();
    if tabs > commas {
        b'\t'
    } else {
        b','
    }
}

fn norm_header(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn detect_profile(headers: &[String]) -> CsvProfile {
    let n: Vec<String> = headers.iter().map(|h| norm_header(h)).collect();

    let has_customer_col = n.iter().any(|h| {
        h == "customer"
            || h == "fullname"
            || h.ends_with("customername")
            || *h == "displayname" && n.iter().any(|x| x.contains("customer"))
    });
    let has_vendor_col = n.iter().any(|h| {
        h == "vendor"
            || h.ends_with("vendorname")
            || (*h == "companyname" && !has_customer_col)
    });
    let has_account =
        n.iter().any(|h| h.contains("account") && (h.contains("name") || h.contains("type")))
            && n.iter().any(|h| h.contains("account") && (h.contains("number") || h.contains("#")));
    let has_item = n.iter().any(|h| {
        (h.contains("item") && (h.contains("name") || h == "item"))
            || *h == "productservice"
            || *h == "product/service"
    });

    if has_account && !has_customer_col && !has_vendor_col {
        return CsvProfile::Accounts;
    }
    if has_item && !has_customer_col {
        return CsvProfile::Items;
    }
    if has_vendor_col && !has_customer_col {
        return CsvProfile::Vendors;
    }
    if has_customer_col {
        return CsvProfile::Customers;
    }
    if has_vendor_col {
        return CsvProfile::Vendors;
    }

    // Fallback: try generic name + email columns
    if n.iter().any(|h| h == "email" || h.contains("phone"))
        && n.iter().any(|h| h == "name" || h.contains("fullname"))
    {
        return CsvProfile::MixedLists;
    }

    CsvProfile::MixedLists
}

fn col_idx(headers: &[String], aliases: &[&str]) -> Option<usize> {
    for (i, h) in headers.iter().enumerate() {
        let nh = norm_header(h);
        for a in aliases {
            let na = norm_header(a);
            if nh == na {
                return Some(i);
            }
        }
    }
    None
}

fn get_field(rec: &csv::StringRecord, headers: &[String], aliases: &[&str]) -> Option<String> {
    let i = col_idx(headers, aliases)?;
    rec.get(i).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn row_customer(headers: &[String], rec: &csv::StringRecord) -> Option<ParsedContact> {
    let name = get_field(rec, headers, &[
        "Customer",
        "Full Name",
        "Display Name",
        "Name",
        "Company Name",
        "Customer Name",
    ])?;
    let email = get_field(rec, headers, &["Email", "E-mail", "E-Mail"]);
    let phone = get_field(rec, headers, &["Phone", "Mobile", "Main Phone", "Work Phone"]);
    Some(ParsedContact {
        display_name: name,
        email,
        phone,
    })
}

fn row_vendor(headers: &[String], rec: &csv::StringRecord) -> Option<ParsedContact> {
    let name = get_field(rec, headers, &[
        "Vendor",
        "Company Name",
        "Full Name",
        "Display Name",
        "Name",
    ])?;
    let email = get_field(rec, headers, &["Email", "E-mail"]);
    let phone = get_field(rec, headers, &["Phone", "Main Phone", "Work Phone"]);
    Some(ParsedContact {
        display_name: name,
        email,
        phone,
    })
}

fn row_account(headers: &[String], rec: &csv::StringRecord) -> Option<ParsedAccount> {
    let name = get_field(rec, headers, &[
        "Account Name",
        "Name",
        "Full Name",
        "Description",
    ])?;
    let code = get_field(rec, headers, &[
        "Account Number",
        "Acct No",
        "Number",
        "No.",
        "Account #",
    ])
    .unwrap_or_else(|| slug_code(&name));
    let type_str = get_field(
        rec,
        headers,
        &[
            "Account Type",
            "Type",
            "Detail Type",
            "Kind",
        ],
    )
    .unwrap_or_default();
    let (account_type, is_bank) = map_csv_account_type(&type_str);
    Some(ParsedAccount {
        code,
        name,
        account_type: account_type.into(),
        is_bank_cash: is_bank,
    })
}

fn row_item(headers: &[String], rec: &csv::StringRecord) -> Option<ParsedItem> {
    let name = get_field(rec, headers, &[
        "Item Name",
        "Name",
        "Product/Service Name",
        "Item",
    ])?;
    let price_s = get_field(rec, headers, &[
        "Sales Price",
        "Price",
        "Rate",
        "Amount",
    ])
    .unwrap_or_default();
    let unit_price_minor = parse_money_minor(&price_s).unwrap_or(0);
    let income_account_name =
        get_field(rec, headers, &["Income Account", "Income", "Sales Account"]);
    Some(ParsedItem {
        name,
        unit_price_minor,
        income_account_name,
    })
}

fn map_csv_account_type(s: &str) -> (&'static str, bool) {
    let u = s.trim().to_lowercase();
    let bank = u.contains("bank")
        || u.contains("checking")
        || u.contains("savings")
        || u.contains("cash");
    let kind = if u.contains("bank")
        || u.contains("accounts receivable")
        || u.contains("other current asset")
        || u.contains("inventory")
        || u.contains("fixed asset")
        || u.contains("asset")
    {
        "asset"
    } else if u.contains("credit card")
        || u.contains("accounts payable")
        || u.contains("liabilit")
        || u.contains("loan")
    {
        "liability"
    } else if u.contains("equity") {
        "equity"
    } else if u.contains("income") || u.contains("revenue") {
        "income"
    } else {
        "expense"
    };
    (kind, bank)
}

#[cfg(test)]
mod tests {
    use super::parse_csv;

    #[test]
    fn parse_customer_csv_comma_delimited() {
        let csv = "Customer Name,Email,Phone\nAcme Corp,acme@test.com,555-0100\n";
        let (batch, skipped, warnings) = parse_csv(csv).unwrap();
        assert_eq!(skipped, 0);
        assert!(warnings.is_empty());
        assert_eq!(batch.customers.len(), 1);
        assert_eq!(batch.customers[0].display_name, "Acme Corp");
        assert_eq!(batch.customers[0].email.as_deref(), Some("acme@test.com"));
    }

    #[test]
    fn parse_vendor_csv_tab_delimited() {
        let csv = "Vendor\tCompany Name\nOffice Mart\tOffice Mart LLC\n";
        let (batch, skipped, _) = parse_csv(csv).unwrap();
        assert_eq!(skipped, 0);
        assert_eq!(batch.vendors.len(), 1);
        assert_eq!(batch.vendors[0].display_name, "Office Mart");
    }

    #[test]
    fn parse_account_csv_maps_type() {
        let csv = "Account Number,Account Name,Account Type\n1000,Cash,Bank\n6000,Utilities,Expense\n";
        let (batch, skipped, _) = parse_csv(csv).unwrap();
        assert_eq!(skipped, 0);
        assert_eq!(batch.accounts.len(), 2);
        assert_eq!(batch.accounts[0].code, "1000");
        assert!(batch.accounts[0].is_bank_cash);
        assert_eq!(batch.accounts[1].account_type, "expense");
    }

    #[test]
    fn parse_credit_card_account_is_liability_not_bank_cash() {
        let csv = "Account Number,Account Name,Account Type\n2100,Visa,Credit Card\n";
        let (batch, skipped, _) = parse_csv(csv).unwrap();
        assert_eq!(skipped, 0);
        assert_eq!(batch.accounts.len(), 1);
        assert_eq!(batch.accounts[0].account_type, "liability");
        assert!(!batch.accounts[0].is_bank_cash);
    }

    #[test]
    fn parse_csv_rejects_empty_headers() {
        assert!(parse_csv("\n").is_err());
    }
}
