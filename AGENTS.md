# AGENTS.md — promptjang-relay-one

Relay One is a packaged, loopback-only SQLite mailbox for CLI agents. It has no webhook delivery, PostgreSQL, Docker, login, Cloud account, agent wake-up, or background agent loop.

## Workflow rules

- Never commit to `main`; use a branch and PR.
- Keep the route, MCP tool, mailbox-state, and API-key contracts consistent.
- Management APIs and the UI stay loopback-only. Producer and mailbox operations require `pj_one_` API keys.
- Full API keys are retrievable because Relay One encrypts them with the local `master.key`.
- Update checks may only contact the official GitHub release feed. Never download or install silently.
- Tagging `v*` publishes portable public-beta artifacts. Require action-time owner confirmation before creating a tag.

## Required gates

```bash
npm --prefix web ci
npm --prefix web run build
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

Also run a temporary-data-dir UAT covering public docs, key creation and reveal, push, idempotency, claim, nack, ack, restart persistence, export/import, and MCP stdio.

## Layout

```text
src/api/          loopback HTTP API and embedded UI/docs
src/store/        SQLite key and mailbox persistence
src/mcp.rs        stdio MCP adapter using the loopback API
src/migration.rs  mailbox export/import
migrations/       SQLite schema
skills/           portable PromptJang Agent Skill
docs/             embedded operator documentation
web/              Vue operational UI
```
