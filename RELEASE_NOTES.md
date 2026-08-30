# PromptJang Relay One v0.1.0

Public beta of a durable local mailbox for CLI agents.

- One executable with SQLite, mailbox API, MCP, UI, and public documentation.
- Mailbox claim leases, acknowledgement, nack, retention, and idempotency.
- Encrypted, owner-retrievable `pj_one_` API keys.
- Loopback-only management with no login or Cloud account.
- Explicit stable-release checks with a Get update / Not now prompt.
- Mailbox export/import for local backup and movement.
- Portable builds for macOS, Windows, and Linux on x86-64 and ARM64 where supported.

Relay One stores work. It does not wake, run, or loop an agent.
