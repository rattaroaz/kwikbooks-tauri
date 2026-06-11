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
