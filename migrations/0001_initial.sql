PRAGMA foreign_keys = ON;

CREATE TABLE destinations (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  url TEXT NOT NULL,
  signing_secret_ciphertext BLOB NOT NULL,
  previous_signing_secret_ciphertext BLOB,
  enabled INTEGER NOT NULL DEFAULT 1,
  deleted_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE api_keys (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  prefix TEXT NOT NULL,
  secret_hash TEXT NOT NULL UNIQUE,
  secret_ciphertext BLOB NOT NULL,
  unrestricted INTEGER NOT NULL DEFAULT 1,
  last_used_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE events (
  id TEXT PRIMARY KEY,
  destination_id TEXT NOT NULL REFERENCES destinations(id),
  status TEXT NOT NULL CHECK (status IN ('QUEUED','PROCESSING','DELIVERED','RETRYING','EXPIRED')),
  event_type TEXT,
  correlation_id TEXT,
  payload TEXT NOT NULL,
  payload_raw BLOB NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'application/json',
  payload_sha256 TEXT NOT NULL,
  idempotency_key_hash TEXT,
  traceparent TEXT,
  tracestate TEXT,
  retry_count INTEGER NOT NULL DEFAULT 0,
  max_retries INTEGER NOT NULL DEFAULT 5,
  is_replay INTEGER NOT NULL DEFAULT 0,
  source_event_id TEXT REFERENCES events(id) ON DELETE SET NULL,
  next_attempt_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX relay_one_events_idempotency_unique ON events(destination_id, idempotency_key_hash)
  WHERE idempotency_key_hash IS NOT NULL AND is_replay = 0;
CREATE INDEX relay_one_events_delivery_queue ON events(status, next_attempt_at, created_at);
CREATE INDEX relay_one_events_destination_created ON events(destination_id, created_at DESC);

CREATE TABLE delivery_attempts (
  id TEXT PRIMARY KEY,
  event_id TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  status_code INTEGER,
  response_body TEXT,
  duration_ms INTEGER NOT NULL,
  error TEXT,
  attempted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE api_key_destinations (
  api_key_id TEXT NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
  destination_id TEXT NOT NULL REFERENCES destinations(id) ON DELETE CASCADE,
  PRIMARY KEY (api_key_id, destination_id)
);

CREATE TABLE mailboxes (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE mailbox_messages (
  id TEXT PRIMARY KEY,
  mailbox_id TEXT NOT NULL REFERENCES mailboxes(id) ON DELETE CASCADE,
  status TEXT NOT NULL CHECK (status IN ('UNREAD','CLAIMED','ACKNOWLEDGED')),
  payload_raw BLOB NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'application/json',
  payload TEXT,
  payload_sha256 TEXT NOT NULL,
  idempotency_key_hash TEXT,
  traceparent TEXT,
  tracestate TEXT,
  claim_token TEXT,
  claimed_until TEXT,
  claim_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX relay_one_mail_idempotency_unique ON mailbox_messages(mailbox_id, idempotency_key_hash)
  WHERE idempotency_key_hash IS NOT NULL;
CREATE INDEX relay_one_mail_claim_queue ON mailbox_messages(mailbox_id, created_at) WHERE status = 'UNREAD';
CREATE INDEX relay_one_mail_lease_recovery ON mailbox_messages(mailbox_id, claimed_until) WHERE status = 'CLAIMED';

CREATE TABLE migration_history (
  id TEXT PRIMARY KEY,
  direction TEXT NOT NULL,
  source_version TEXT NOT NULL,
  migrated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
