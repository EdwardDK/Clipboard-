CREATE INDEX IF NOT EXISTS idx_clipboard_pinned_order ON clipboard_items(pinned, pin_order);
CREATE INDEX IF NOT EXISTS idx_group_rules_group_id ON group_rules(group_id);
INSERT OR IGNORE INTO settings(key, value, updated_at) VALUES
  ('max_history_size', '5000', CURRENT_TIMESTAMP),
  ('delete_unpinned_after_days', '90', CURRENT_TIMESTAMP),
  ('excluded_content_types', '[]', CURRENT_TIMESTAMP),
  ('excluded_applications', '[]', CURRENT_TIMESTAMP);
