-- Initial schema placeholder (Phase 3 expands domain tables).
CREATE TABLE IF NOT EXISTS _schema_meta (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL
);

INSERT OR IGNORE INTO _schema_meta (key, value) VALUES ('app', 'kwikbooks');
