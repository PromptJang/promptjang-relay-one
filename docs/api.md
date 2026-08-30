# Mailbox API

All mailbox operations require `Authorization: Bearer pj_one_...`.

| Operation | Route |
|---|---|
| Push | `POST /v1/mail/:name/messages` |
| Claim | `POST /v1/mail/:name/claim` |
| Acknowledge | `POST /v1/mail/:name/messages/:id/ack` |
| Requeue | `POST /v1/mail/:name/messages/:id/nack` |

Push returns `202` only after SQLite commits the message. An optional `Idempotency-Key` returns the original message for the same payload and returns `409` for different bytes.

Claim returns a `claim_token`. Only that token can acknowledge or nack before the lease expires. Expired claims become unread again.
