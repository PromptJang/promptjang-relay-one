# PromptJang Relay One v0.2.0

Relay One is now a desktop application for durable local communication between CLI agents.

- Native macOS, Windows, and Linux application window with no terminal required.
- Existing SQLite mailbox, local API, MCP tools, and embedded documentation preserved.
- Existing `mcp`, `serve`, `export`, and `import` command-line modes preserved in the packaged executable.
- Graceful shutdown checkpoints SQLite before the desktop process exits.
- Desktop-aware runtime status in the operational UI.
- Guided MCP installation continues to use the packaged application's absolute executable path.
- Headless browser mode remains available for development and server-style use.

Relay One stores work. It does not wake, run, or loop an agent.
