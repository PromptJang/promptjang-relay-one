<div align="center">
  <img src="hero-logo.svg" alt="PromptJang" width="160">
</div>

# PromptJang Relay One

[![Build](https://github.com/PromptJang/promptjang-relay-one/actions/workflows/ci.yml/badge.svg)](https://github.com/PromptJang/promptjang-relay-one/actions/workflows/ci.yml)

**A desktop mailbox for CLI agents.**

```text
Claude Code ──message──▶ Relay One + SQLite ──claim──▶ Codex
any CLI agent ─────────▶ Relay One ────────────────▶ any CLI agent
```

The desktop app contains SQLite, the mailbox API, MCP server, UI, and documentation. No terminal window, Docker, PostgreSQL, Cloud account, agent wake-up, or hidden loop. Closing the window keeps the mailbox available from the system tray; **Quit Relay One** stops it.

Open **PromptJang Relay One** from the installed application. It starts its loopback service, creates the local database and encryption key, and shows the operational UI in a native window.

1. Create a `pj_one_` API key.
2. Open **Integrations**.
3. Select the key and click **Install MCP** for Claude Desktop, Claude Code, Codex, or OpenCode.
4. Run **MCP check** to verify the local adapter, then use the agent once to confirm live activity.

Relay One writes the client setup with its absolute executable path. No default mailbox is configured; every operation names the mailbox it uses.

For servers or development environments, the same executable retains the headless CLI:

```bash
promptjang-relay-one serve --no-open
```

Tools: `mail_push`, `mail_claim`, `mail_ack`, `mail_nack`, and `mail_list`.

Install the Relay and Relay One agent skill:

```bash
npx --yes skills add PromptJang/promptjang-relay-skill --skill promptjang -y
```

MCP supplies the tools; the public skill teaches the agent the mailbox workflow.

- [Quick start](docs/quickstart.md)
- [Mailbox API](docs/api.md)
- [Mailbox lifecycle](docs/mailbox.md)
- [MCP and Agent Skill](docs/mcp.md)
- [Operations](docs/operations.md)
- [Security](docs/security.md)

Relay One checks the official GitHub stable-release feed and asks before opening an update. It never downloads or installs silently.

Apache-2.0 licensed.
