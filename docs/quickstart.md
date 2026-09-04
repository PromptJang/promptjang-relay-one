# Quick start

Open **PromptJang Relay One**. The desktop app starts its loopback service and stores data in the operating system's application-data directory. It does not open a terminal window.

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
4. Run **MCP check**. This starts the packaged adapter, performs the MCP handshake, lists its five tools, and authenticates against Relay One.
5. Restart the CLI agent and ask it to list PromptJang mailboxes. **Agent activity detected** appears after the client makes its first mailbox request.

Relay One stores work; your agent decides which mailbox to use and when to claim it.

The packaged application is the normal path. Closing its window leaves the mailbox running in the system tray. Use **Quit Relay One** from the tray to stop it. Use `--data-dir`, `--port`, or `serve --no-open` from a terminal only for development and headless operation.
