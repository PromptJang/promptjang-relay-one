# Quick start

Run the executable. Relay One opens its dashboard at `http://127.0.0.1:8081` and stores data in the operating system's application-data directory.

1. Create an API key in **API keys**.
2. Push a message:

```bash
curl -X POST http://127.0.0.1:8081/v1/mail/codex/messages \
  -H 'Authorization: Bearer pj_one_YOUR_KEY' \
  -H 'Content-Type: application/json' \
  -H 'Idempotency-Key: task-42' \
  -d '{"task":"review the current branch"}'
```

3. Open **Integrations**, select that key, and click **Install MCP** for your CLI agent.
4. Restart the CLI agent and ask it to list PromptJang mailboxes.

Relay One stores work; your agent decides which mailbox to use and when to claim it.

Use `--data-dir`, `--port`, or `serve --no-open` when the defaults do not fit.
