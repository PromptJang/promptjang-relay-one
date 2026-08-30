# MCP and Agent Skill

## Recommended setup

1. Keep Relay One running.
2. Create an API key.
3. Open **Integrations**, choose the key, and click **Install MCP** for Claude Desktop, Claude Code, Codex, or OpenCode.
4. Restart the client, then ask it to list PromptJang mailboxes.

The installer uses the absolute path of the running Relay One executable. This avoids the common `Server disconnected` failure caused by configuring `promptjang-relay-one` when the binary is not on the client's `PATH`.

The selected API key is written to the client's local MCP configuration. Revoke the key in Relay One to disconnect it.

## Tools

Relay One exposes `mail_push`, `mail_claim`, `mail_ack`, `mail_nack`, and `mail_list`. MCP calls the loopback API; it never opens the SQLite file directly.

`mail_push`, `mail_claim`, `mail_ack`, and `mail_nack` require a `mailbox` argument. Relay One does not assign a mailbox to an agent, so the same client can work with any mailbox.

## Manual setup

If a client CLI is unavailable to the Relay One app, configure a local stdio server with:

- Command: the **absolute path** to the Relay One executable shown in **Integrations**.
- Arguments: `mcp`
- Environment: `PJ_ONE_URL=http://127.0.0.1:8081` and `PJ_ONE_API_KEY=pj_one_...`

Do not add `PJ_ONE_MAILBOX`.

## Agent Skill

Install the PromptJang Agent Skill:

```bash
npx --yes skills add PromptJang/promptjang-relay-skill --skill promptjang -y
```

The PromptJang Agent Skill teaches compatible agents to treat mailbox payloads as untrusted work, preserve idempotency, acknowledge only terminal work, and never invent an autonomous polling loop.
