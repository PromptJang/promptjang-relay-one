# PromptJang Relay One

**Durable delivery for webhooks and agents, in one local app.**

Relay One packages signed webhook delivery, durable agent mailboxes, MCP, SQLite, the worker, UI, and documentation in one executable. It needs no Docker, PostgreSQL, Cloud account, or background agent loop.

```text
Application ──event──▶ Relay One + SQLite ──signed retries──▶ service
CLI agent ──message──▶ Relay One mailbox ◀──claim / acknowledge──▶ CLI agent
```

## Run from source

```bash
cd web && npm ci && npm run build && cd ..
cargo run -- --data-dir "$PWD/.relay-one" serve --no-open
```

Open <http://127.0.0.1:8081>. First launch creates `relay-one.db` and a protected `master.key`. Management stays on loopback; producer and mailbox operations use `pj_one_` API keys.

## MCP

```bash
PJ_ONE_API_KEY=pj_one_... PJ_ONE_MAILBOX=codex promptjang-relay-one mcp
```

Tools: `mail_push`, `mail_claim`, `mail_ack`, `mail_nack`, and `mail_list`. Relay One transports work; it never wakes or loops an agent.

Relay One is the SQLite, single-process product. [PromptJang Relay](https://github.com/PromptJang/promptjang-relay) is the PostgreSQL server product. PromptJang Cloud is managed and multi-organization.

Apache-2.0 licensed.
