# MCP and Agent Skill

## Recommended setup

1. Keep Relay One running.
2. Create an API key.
3. Open **Integrations**, choose the key, and click **Install MCP** for Claude Desktop, Claude Code, Codex, or OpenCode.
4. Click **MCP check** to verify the adapter and API key.
5. Restart the client, then ask it to list PromptJang mailboxes.

The installer uses the absolute path of the running Relay One executable. This avoids the common `Server disconnected` failure caused by configuring `promptjang-relay-one` when the binary is not on the client's `PATH`.

The selected API key is written to the client's local MCP configuration. Revoke the key in Relay One to disconnect it.

The Integrations screen separates three facts:

- **Configured** means Relay One wrote the selected client configuration.
- **Adapter verified** means Relay One started that adapter, completed an MCP handshake, found the five expected tools, and authenticated with the local API.
- **Agent activity detected** means the configured client made a mailbox request. The check cannot prove that the client loaded its configuration; only real client activity can.

## Tools

Relay One exposes `mail_push`, `mail_claim`, `mail_ack`, `mail_nack`, and `mail_list`. MCP calls the loopback API; it never opens the SQLite file directly.

`mail_push`, `mail_claim`, `mail_ack`, and `mail_nack` require a `mailbox` argument. Relay One does not assign a mailbox to an agent, so the same client can work with any mailbox.

## Manual setup

If a client CLI is unavailable to the Relay One app, configure a local stdio server with:

- Command: the **absolute path** to the Relay One executable shown in **Integrations**.
- Arguments: `mcp`
- Environment: `PJ_ONE_URL=http://127.0.0.1:8081` and `PJ_ONE_API_KEY=pj_one_...`. Generated setup also adds `PJ_ONE_CLIENT` so Relay One can report activity for that client.

Do not add `PJ_ONE_MAILBOX`.

## Agent Skill

Install the PromptJang Agent Skill:

```bash
npx --yes skills add PromptJang/promptjang-relay-skill --skill promptjang -y
```

The PromptJang Agent Skill teaches compatible agents to treat mailbox payloads as untrusted work, preserve idempotency, acknowledge only terminal work, and never invent an autonomous polling loop.
