# Kwikbooks

Local QuickBooks-style accounting: **Tauri 2** desktop app, **React** UI, **SQLite** with double-entry posting. Single company, offline-first.

## Quick start

```bash
npm ci
npm start          # or: npm run tauri:dev
```

Do **not** use `npm run dev` alone for real work — that is Vite only; IPC and the database require the Tauri shell.

## Scripts

| Command                                       | Purpose                                 |
| --------------------------------------------- | --------------------------------------- |
| `npm run typecheck`                           | TypeScript                              |
| `npm run lint`                                | ESLint                                  |
| `npm run test`                                | Vitest (unit + property)                |
| `npm run test:coverage`                       | Vitest with coverage thresholds         |
| `npm run test:e2e`                            | Playwright (Chromium + Firefox)         |
| `npm run test:e2e:ci`                         | E2E without visual snapshots (Linux CI) |
| `npm run format:check`                        | Prettier                                |
| `cd src-tauri && cargo test`                  | Rust unit/integration tests             |
| `cd src-tauri && cargo clippy -- -D warnings` | Rust lints                              |
| `npm run build:win`                           | Windows installer (unsigned, local dev) |
| `npm run build:win:signed`                    | Windows installer + updater signatures  |

## Environment

Copy [`.env.example`](.env.example) to `.env` or `.env.local` for Vite overrides.

| Variable                   | Default   | Meaning                                      |
| -------------------------- | --------- | -------------------------------------------- |
| `VITE_VERBOSE_IPC`         | on in dev | Log each IPC call (redacted args)            |
| `VITE_LOG_CONSOLE_FORWARD` | on in dev | Forward `console.*` to host logs             |
| `VITE_SLOW_IPC_MS`         | `1500`    | Warn when IPC exceeds this (ms)              |
| `VITE_DIAGNOSTICS`         | on in DEV | Attach breadcrumbs/meta to local captures    |

`VITE_TELEMETRY` is accepted as a legacy alias for `VITE_DIAGNOSTICS`. Observability is **offline-only** — exceptions and logs never leave the machine.

Host (Rust) logging:

| Variable                     | Meaning                           |
| ---------------------------- | --------------------------------- |
| `KWIKBOOKS_LOG` / `RUST_LOG` | Log level (`debug`, `info`, …)    |
| `KWIKBOOKS_LOG_JSON`         | `1` for JSON lines                |
| `KWIKBOOKS_SLOW_MS`          | Slow invoke threshold on the host |

## Logs & diagnostics

- Logs: OS log directory via `tauri-plugin-log` (`kwikbooks.log`, `webview.log`, rotated KeepAll), plus `panic.log` for Rust panics, and terminal in dev.
- Settings → **Diagnostics** → **View logs**: live tail, level filter, **Copy**, and **Export…** (redacted support bundle for sharing).
- IPC: request `rid` + host `seq`, slow-call warnings on both sides, breadcrumb ring for local exception context.
- Database: app data dir (printed at startup). **Backup / restore** in Settings — backup files are full SQLite copies (all company data).

## Tests & CI

GitHub Actions runs typecheck, lint, coverage, build, E2E (Ubuntu), `cargo test` + clippy (Ubuntu with GTK/WebKit deps), and a Windows smoke build + visual snapshots.

Visual regression baselines are **Windows** (`tests/e2e/*-snapshots/*-win32.png`).

## Application updates

Updates are **manual only** (Settings → **Check for updates**). The app fetches signed `latest.json` from GitHub Releases; it never checks on startup.

**Installed users:** Help yourself via Settings when a new release exists.

**Publish a release:**

1. Bump version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` (keep in sync).
2. Commit and push to `main`.
3. `git tag vX.Y.Z && git push origin vX.Y.Z`
4. GitHub Actions **Release** workflow builds **x64** and **ARM64** Windows installers (NSIS + MSI each), uploads `latest.json` and `.sig` files. Both matrix jobs must succeed — the workflow validates that `latest.json` contains `windows-aarch64`. Tag releases from a commit that includes the ARM64 workflow (`.github/workflows/release.yml` matrix).

**Repository secrets** (required for signed releases):

- `TAURI_SIGNING_PRIVATE_KEY` — contents of `scripts/tauri-signing.key` (never commit this file)
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — empty string if the key has no password

Endpoint: `https://github.com/rattaroaz/kwikbooks-tauri/releases/latest/download/latest.json`

Regenerate keys: `npm run tauri signer generate -- -w scripts/tauri-signing.key --ci --force` (then update `pubkey` in `tauri.conf.json`).

## Keyboard shortcuts

With focus in the app (not in a text field): **Alt+1** Dashboard … **Alt+9** Settings, **Alt+0** Getting started.
