INSERT OR IGNORE INTO groups(id, name, kind, created_at) VALUES
  ('system-links', 'Links', 'automatic', CURRENT_TIMESTAMP),
  ('system-code', 'Programming', 'automatic', CURRENT_TIMESTAMP),
  ('system-images', 'Images', 'automatic', CURRENT_TIMESTAMP);
