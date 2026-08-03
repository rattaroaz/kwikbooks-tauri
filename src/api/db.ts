import { invoke } from "./invoke";

export type DbInitResponse = {
  dbPath: string;
  migrationVersion: number;
};

export type MigrateResponse = {
  migrationVersionBefore: number;
  migrationVersionAfter: number;
};

export type HealthResponse = {
  ok: boolean;
  sqliteOk: boolean;
  migrationVersion: number;
  /** Host binary version (CARGO_PKG_VERSION). */
  appVersion: string;
  /** Effective host log level from KWIKBOOKS_LOG / RUST_LOG. */
  logLevel: string;
  /** Host slow-IPC threshold (KWIKBOOKS_SLOW_MS). */
  slowIpcMs: number;
};

export async function dbInit(): Promise<DbInitResponse> {
  return invoke<DbInitResponse>("db_init");
}

export async function dbMigrate(): Promise<MigrateResponse> {
  return invoke<MigrateResponse>("db_migrate");
}

export async function healthPing(): Promise<HealthResponse> {
  return invoke<HealthResponse>("health_ping");
}
