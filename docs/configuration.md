# Configuration

| Option | Default |
|---|---|
| `--data-dir` | Operating-system application data directory |
| `--port` | `8081` |
| `serve --no-open` | Do not open a browser |
| `PJ_ONE_URL` | `http://127.0.0.1:8081` for MCP |
| `PJ_ONE_API_KEY` | Required for MCP |
| `PJ_UPDATE_CHECK_ENABLED` | `true`; set to `false` for no update-check network request |

Relay One always binds to `127.0.0.1`. Remote administration is intentionally unsupported.

`PJ_ONE_URL` and `PJ_ONE_API_KEY` are normally written by **Integrations → Install MCP**. There is no default-mailbox setting. Each tool call names its mailbox explicitly.
