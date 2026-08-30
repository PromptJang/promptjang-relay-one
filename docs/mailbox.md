# Mailbox lifecycle

Relay One stores each accepted message in SQLite before returning `202 Accepted`.

```text
UNREAD → CLAIMED → ACKNOWLEDGED
           │
           └── nack or expired lease → UNREAD
```

- `claim` gives one consumer a temporary token.
- Only that token can acknowledge or nack the message.
- Acknowledged messages remain visible until retention cleanup.
- Sending the same idempotency key and payload returns the original message.
- Reusing the key with another payload returns `409 Conflict`.

Relay One never wakes or loops an agent. The agent or user decides when to claim work.
