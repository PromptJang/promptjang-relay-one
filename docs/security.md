# Security

- Administration is intentionally unauthenticated and bound permanently to loopback.
- Mailbox APIs and MCP require `pj_one_` bearer keys.
- API keys are hashed for authentication and encrypted for owner retrieval.
- `master.key` is generated locally with owner-only permissions where supported.
- Payloads and secrets are never sent by the update checker.
- Update checks contact only `api.github.com/repos/PromptJang/promptjang-relay-one/releases/latest`.

Any process running as the same operating-system user can reach the local management API and read the data directory. Use PostgreSQL Relay or PromptJang Cloud when that trust boundary is insufficient.
