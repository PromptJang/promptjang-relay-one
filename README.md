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

Relay One opens <http://127.0.0.1:8081>, creates its local database and encryption key, and stays in the foreground.

1. Create a `pj_one_` API key.
2. Open **Integrations**.
3. Select the key and click **Install MCP** for Claude Desktop, Claude Code, Codex, or OpenCode.

Relay One writes the client setup with its absolute executable path. No default mailbox is configured; every operation names the mailbox it uses.

Tools: `mail_push`, `mail_claim`, `mail_ack`, `mail_nack`, and `mail_list`.

Install the Relay and Relay One agent skill:

```bash
npx --yes skills add PromptJang/promptjang-relay-skill --skill promptjang -y
```

The release bundle also includes a copy in `skills/promptjang` for offline installation. MCP supplies the tools; the skill teaches the agent the mailbox workflow.

- [Quick start](docs/quickstart.md)
- [Mailbox API](docs/api.md)
- [Mailbox lifecycle](docs/mailbox.md)
- [MCP and Agent Skill](docs/mcp.md)
- [Operations](docs/operations.md)
- [Security](docs/security.md)

Relay One checks the official GitHub stable-release feed and asks before opening an update. It never downloads or installs silently.

Apache-2.0 licensed.
