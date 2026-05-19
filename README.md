# Kwikbooks

Local QuickBooks-style accounting: **Tauri 2** desktop app, **React** UI, **SQLite** with double-entry posting. Single company, offline-first.

## Quick start

```bash
npm ci
npm start          # or: npm run tauri:dev
```

Do **not** use `npm run dev` alone for real work — that is Vite only; IPC and the database require the Tauri shell.

## Scripts

| Command | Purpose |
|---------|---------|
| `npm run typecheck` | TypeScript |
| `npm run lint` | ESLint |
| `npm run test` | Vitest (unit + property) |
| `npm run test:coverage` | Vitest with coverage thresholds |
| `npm run test:e2e` | Playwright (Chromium + Firefox) |
| `npm run test:e2e:ci` | E2E without visual snapshots (Linux CI) |
| `npm run format:check` | Prettier |
| `cd src-tauri && cargo test` | Rust unit/integration tests |
| `cd src-tauri && cargo clippy -- -D warnings` | Rust lints |

## Environment

Copy [`.env.example`](.env.example) to `.env` or `.env.local` for Vite overrides.

| Variable | Default | Meaning |
|----------|---------|---------|
| `VITE_VERBOSE_IPC` | on in dev | Log each IPC call (redacted args) |
| `VITE_LOG_CONSOLE_FORWARD` | on in dev | Forward `console.*` to host logs |
| `VITE_SLOW_IPC_MS` | `1500` | Warn when IPC exceeds this (ms) |
| `VITE_TELEMETRY` | off | Reserved; no remote telemetry in v1 |

Host (Rust) logging:

| Variable | Meaning |
|----------|---------|
| `KWIKBOOKS_LOG` / `RUST_LOG` | Log level (`debug`, `info`, …) |
| `KWIKBOOKS_LOG_JSON` | `1` for JSON lines |
| `KWIKBOOKS_SLOW_MS` | Slow invoke threshold on the host |

## Logs & backups

- Logs: OS log directory via `tauri-plugin-log` (`kwikbooks.log`, `webview.log`), plus terminal in dev.
- Database: app data dir (printed at startup). **Backup / restore** in Settings — backup files are full SQLite copies (all company data).

## Tests & CI

GitHub Actions runs typecheck, lint, coverage, build, E2E (Ubuntu), `cargo test` + clippy (Ubuntu with GTK/WebKit deps), and a Windows smoke build + visual snapshots.

Visual regression baselines are **Windows** (`tests/e2e/*-snapshots/*-win32.png`).

## Keyboard shortcuts

With focus in the app (not in a text field): **Alt+1** Dashboard … **Alt+9** Settings, **Alt+0** Getting started.
