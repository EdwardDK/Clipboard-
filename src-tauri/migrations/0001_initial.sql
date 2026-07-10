PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS groups (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE, kind TEXT NOT NULL DEFAULT 'manual', created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS group_rules (id TEXT PRIMARY KEY, group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE, field TEXT NOT NULL, operator TEXT NOT NULL, value TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS clipboard_items (
  id TEXT PRIMARY KEY, content TEXT NOT NULL, content_type TEXT NOT NULL, source_application TEXT, window_title TEXT,
  created_at TEXT NOT NULL, last_copied_at TEXT NOT NULL, copy_count INTEGER NOT NULL DEFAULT 1,
  pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0,1)), pin_order INTEGER, label TEXT, group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
  sensitive INTEGER NOT NULL DEFAULT 0 CHECK(sensitive IN (0,1)), backup_eligible INTEGER NOT NULL DEFAULT 1 CHECK(backup_eligible IN (0,1)), content_hash TEXT NOT NULL UNIQUE
);
CREATE TABLE IF NOT EXISTS tags (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE IF NOT EXISTS clipboard_item_tags (clipboard_item_id TEXT NOT NULL REFERENCES clipboard_items(id) ON DELETE CASCADE, tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE, PRIMARY KEY(clipboard_item_id, tag_id));
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS sync_queue (id TEXT PRIMARY KEY, clipboard_item_id TEXT REFERENCES clipboard_items(id) ON DELETE CASCADE, operation TEXT NOT NULL, created_at TEXT NOT NULL, attempts INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS devices (id TEXT PRIMARY KEY, name TEXT NOT NULL, last_seen_at TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS idx_clipboard_created_at ON clipboard_items(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clipboard_last_copied_at ON clipboard_items(last_copied_at DESC);
CREATE INDEX IF NOT EXISTS idx_clipboard_source_application ON clipboard_items(source_application);
CREATE INDEX IF NOT EXISTS idx_clipboard_content_type ON clipboard_items(content_type);
CREATE INDEX IF NOT EXISTS idx_clipboard_pinned ON clipboard_items(pinned);
CREATE INDEX IF NOT EXISTS idx_clipboard_group_id ON clipboard_items(group_id);
CREATE INDEX IF NOT EXISTS idx_clipboard_sensitive ON clipboard_items(sensitive);
CREATE INDEX IF NOT EXISTS idx_clipboard_content_hash ON clipboard_items(content_hash);
CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_items_fts USING fts5(content, content_type UNINDEXED, source_application UNINDEXED, content='clipboard_items', content_rowid='rowid', tokenize='unicode61');
CREATE TRIGGER IF NOT EXISTS clipboard_items_ai AFTER INSERT ON clipboard_items BEGIN INSERT INTO clipboard_items_fts(rowid, content, content_type, source_application) VALUES (new.rowid, new.content, new.content_type, new.source_application); END;
CREATE TRIGGER IF NOT EXISTS clipboard_items_ad AFTER DELETE ON clipboard_items BEGIN INSERT INTO clipboard_items_fts(clipboard_items_fts, rowid, content, content_type, source_application) VALUES ('delete', old.rowid, old.content, old.content_type, old.source_application); END;
CREATE TRIGGER IF NOT EXISTS clipboard_items_au AFTER UPDATE OF content, content_type, source_application ON clipboard_items BEGIN INSERT INTO clipboard_items_fts(clipboard_items_fts, rowid, content, content_type, source_application) VALUES ('delete', old.rowid, old.content, old.content_type, old.source_application); INSERT INTO clipboard_items_fts(rowid, content, content_type, source_application) VALUES (new.rowid, new.content, new.content_type, new.source_application); END;
