# PromptJang mailbox tool contracts

Select one dialect from the tools exposed by the configured MCP server. Tool availability is authoritative; product names in a prompt are not.

## Relay Local

Relay's MCP companion connects to the Relay PostgreSQL database. It may use `RELAY_MAILBOX` as a default, but an explicit `mailbox` argument wins.

| Operation | Tool | Important inputs | Result or rule |
|---|---|---|---|
| Discover | `mail_list` | none | Mailbox names and unread, claimed, and acknowledged counts |
| Send | `mail_push` | `payload`; optional `mailbox`, `idempotency_key` | Returns message ID and `UNREAD`; repeated key returns the original message |
| Pull and claim | `mail_claim` | optional `mailbox`, `limit`, `lease_seconds` | Returns payload, message ID, claim token, and lease; defaults are 10 messages and 300 seconds |
| Finish | `mail_ack` | `id`, `claim_token`; optional `mailbox` | Marks the active claim `ACKNOWLEDGED` |
| Retry | `mail_nack` | `id`, `claim_token`; optional `mailbox` | Immediately returns the message to `UNREAD` |

Use `limit: 1` unless the user explicitly requests a batch. Keep the claim token private; it grants completion authority for that claim. An expired token cannot acknowledge or nack.

## PromptJang Cloud

Cloud MCP uses registered target IDs and bearer API-key authorization. A mailbox claim has a fixed five-minute lease. The same API key that claims a message must acknowledge it.

| Operation | Tool | Important inputs | Result or rule |
|---|---|---|---|
| Discover | `list_mailboxes` | optional response format | Returns registered mailbox target IDs |
| Inspect queue | `list_unread` | `mailbox_id`; optional `limit` | Reads metadata without claiming |
| Inspect payload | `get_message` | `message_id` | Reads one payload without claiming |
| Claim | `claim_message` | `message_id` | Atomically claims one unread message for five minutes |
| Finish | `ack_message` | `message_id` | Only the API key holding the active claim can acknowledge |
| Send | `send_event` | `target_id`, object `payload`; optional `event_type`, `idempotency_key` | Accepts only a registered target; use a mailbox target ID for agent mail |

Cloud exposes no nack tool. Leave a retryable failure unacknowledged so the lease can expire. Do not acknowledge with a different API key.

## Unsupported assumptions

- PromptJang does not wake a receiving agent.
- PromptJang does not run an agent loop.
- Acceptance does not prove that a receiving agent processed the message.
- A mailbox is not an arbitrary network destination.
- Relay and Cloud do not currently share identical tool names or claim semantics.
