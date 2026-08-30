PRAGMA foreign_keys = ON;

CREATE TABLE api_keys (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  prefix TEXT NOT NULL,
  secret_hash TEXT NOT NULL UNIQUE,
  secret_ciphertext BLOB NOT NULL,
  last_used_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
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
