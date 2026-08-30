# Mailbox tools

Relay and Relay One expose the same tools. Always pass an explicit `mailbox`, even when a client has a default configured.

| Operation | Tool | Inputs | Result |
|---|---|---|---|
| Discover | `mail_list` | none | Mailboxes and state counts |
| Send | `mail_push` | `mailbox`, `payload`; optional `idempotency_key` | Message ID and `UNREAD` |
| Claim | `mail_claim` | `mailbox`; optional `limit`, `lease_seconds` | Messages, claim tokens, and lease |
| Finish | `mail_ack` | `mailbox`, `id`, `claim_token` | `ACKNOWLEDGED` |
| Retry | `mail_nack` | `mailbox`, `id`, `claim_token` | `UNREAD` |

Defaults: claim limit 10 and lease 300 seconds. The skill uses `limit: 1` unless the user requests a batch. An expired or already-used claim token cannot acknowledge or nack.

An idempotency key returns the original message only when the payload is unchanged. Reusing the key with different content is a conflict.
