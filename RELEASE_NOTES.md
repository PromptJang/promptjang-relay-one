# PromptJang Relay v0.4.0

Relay v0.4 makes the agent mailbox a first-class operational workflow while keeping reliable webhook delivery intact.

## Highlights

- Mailbox UI for inspecting messages and lifecycle state.
- Hardened mailbox idempotency, claim leases, acknowledgement, and requeue behavior.
- MCP companion fixes for accurate idempotent message status.
- Portable PromptJang Agent Skill for Relay and Cloud mailbox tools.
- Clearer README and built-in documentation for technical and non-technical readers.
- Copy controls for one-time API-key and signing-secret values.

Relay still does not run or wake agents. The user, CLI, or scheduler owns the agent loop.

Webhook signing remains Standard Webhooks v1. Review [the v0.2 to v0.3 migration guide](docs/migration-v02-v03.md) only when upgrading from the old signing contract.
