# Kwikbooks — security notes (v1 desktop)

Threat model assumptions:

- **Local-only**: data stays in SQLite under the OS app data directory. There is no server in v1.
- **Authenticated OS user**: protections assume a normal desktop user account; attackers with malware or disk access already outside the app’s trust boundary.

## Path handling

Backup and restore use absolute paths supplied by OS file dialogs. The backend rejects restores when source and destination resolve to the same file. Avoid constructing paths from remote or untrusted text without validation.

## SQL injection

All queries use parameterized SQL (`?1`, `rusqlite::params![…]`). Values are never concatenated into raw SQL fragments from user-controlled strings.

## Secrets

OAuth tokens / API keys are **out of scope** for v1. When added, prefer **platform keystore / Tauri secrets plugin**, not plaintext in SQLite.

## Logging

Do not publish full database file paths in shared logs or crash reports. Frontend may display paths for troubleshooting; scrub before posting anywhere public.

## Supply chain / release

Prefer signed installers when distributing. Updates (Tauri updater) remain optional until a release pipeline is configured.
