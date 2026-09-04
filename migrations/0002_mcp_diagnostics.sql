CREATE TABLE mcp_installations (
  client TEXT PRIMARY KEY CHECK (client IN ('claude-desktop','claude-code','codex','opencode')),
  key_id TEXT NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
  configured_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  adapter_verified_at TEXT,
  last_activity_at TEXT
);
