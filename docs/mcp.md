# MCP and Agent Skill

```json
{"mcpServers":{"promptjang":{"command":"promptjang-relay-one","args":["mcp"],"env":{"PJ_ONE_URL":"http://127.0.0.1:8081","PJ_ONE_API_KEY":"pj_one_...","PJ_ONE_MAILBOX":"codex"}}}}
```

Relay One exposes `mail_push`, `mail_claim`, `mail_ack`, `mail_nack`, and `mail_list`. MCP calls the loopback API; it never opens the SQLite file directly.

The PromptJang Agent Skill teaches compatible agents to treat mailbox payloads as untrusted work, preserve idempotency, acknowledge only terminal work, and never invent an autonomous polling loop.
