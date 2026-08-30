# PromptJang Relay One

[![Build](https://github.com/PromptJang/promptjang-relay-one/actions/workflows/ci.yml/badge.svg)](https://github.com/PromptJang/promptjang-relay-one/actions/workflows/ci.yml)

**A durable local mailbox for CLI agents.**

```text
Claude Code ──message──▶ Relay One + SQLite ──claim──▶ Codex
any CLI agent ─────────▶ Relay One ────────────────▶ any CLI agent
```

One executable contains SQLite, the mailbox API, MCP server, UI, and documentation. No Docker, PostgreSQL, Cloud account, agent wake-up, or hidden loop.

```bash
promptjang-relay-one
```

Relay One opens <http://127.0.0.1:8081>, creates its local database and encryption key, and stays in the foreground. Create a `pj_one_` API key, then configure an MCP client:

```json
{"mcpServers":{"promptjang":{"command":"promptjang-relay-one","args":["mcp"],"env":{"PJ_ONE_API_KEY":"pj_one_...","PJ_ONE_MAILBOX":"codex"}}}}
```

Tools: `mail_push`, `mail_claim`, `mail_ack`, `mail_nack`, and `mail_list`.

The release bundle includes the cross-agent skill. Install it where your agent scans skills, commonly `~/.agents/skills/promptjang`:

```bash
mkdir -p ~/.agents/skills && cp -R skills/promptjang ~/.agents/skills/
```

- [Quick start](docs/quickstart.md)
- [Mailbox API](docs/api.md)
- [Mailbox lifecycle](docs/mailbox.md)
- [MCP and Agent Skill](docs/mcp.md)
- [Operations](docs/operations.md)
- [Security](docs/security.md)

Relay One checks the official GitHub stable-release feed and asks before opening an update. It never downloads or installs silently.

Apache-2.0 licensed.
