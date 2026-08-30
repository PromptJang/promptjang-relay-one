# Quick start

Relay is one application plus PostgreSQL. Start with webhook delivery, then add an agent mailbox only if you need pull-based work.

```text
Start Relay ──▶ Create API key ──▶ Send webhook or mailbox message ──▶ Inspect
```

## 1. Start

```bash
cp .env.example .env   # set PJ_ENCRYPTION_KEY (base64 of 32 random bytes)
docker compose up -d
```

Open http://localhost:8080 and sign in with `PJ_ADMIN_USERNAME` / `PJ_ADMIN_PASSWORD` (defaults: `admin` / the password in your `.env`).

## 2. Create a destination and key

In the UI: **Destinations → create** (any public HTTPS URL — webhook.site works for testing), then **API keys → create**. Relay encrypts the full key at rest so the owner can copy it again later.

## 3. Send your first event

```bash
curl -X POST "http://localhost:8080/v1/destinations/$DESTINATION_ID/events" \
  -H "Authorization: Bearer pj_relay_YOUR_KEY" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: first-event" \
  -H "X-Event-Type: hello" \
  -d '{"hello":"relay"}'
```

`202` means committed. Watch **Events** for `QUEUED → DELIVERED` with the receiver's response stored as evidence. See [API and signing](api.md) for verification on the receiver side.

## 4. Optional: pull instead of push

```bash
curl -X POST "http://localhost:8080/v1/mail/agent-tasks/messages" \
  -H "Authorization: Bearer pj_relay_YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"task":"summarize"}'

curl -X POST "http://localhost:8080/v1/mail/agent-tasks/claim" \
  -H "Authorization: Bearer pj_relay_YOUR_KEY" \
  -H "Content-Type: application/json" -d '{"limit":10}'
```

Claim returns `claim_token`s; finish each message with `ack` (or `nack` to requeue). Details in [Agent mailbox](mailbox.md).

## 5. Optional: connect a local agent with MCP

Give Claude Code, opencode, or any MCP client mailbox access by running the bundled MCP server against the same database:

```json
{
  "mcpServers": {
    "promptjang-relay": {
      "command": "promptjang-relay-mcp",
      "env": {
        "DATABASE_URL": "postgres://relay:password@localhost:5432/relay",
        "RELAY_MAILBOX": "agent-tasks"
      }
    }
  }
}
```

Build the server once with `cargo install --path mcp`. The agent can then call `mail_push`, `mail_claim`, `mail_ack`, `mail_nack`, and `mail_list` as tools. Compose binds PostgreSQL to `127.0.0.1:5432` for exactly this use.

## Where next

- [Configuration reference](configuration.md) — every environment variable
- [Operations](operations.md) — backups, upgrades, scaling
- [Security](security.md) — private networks, secret handling
