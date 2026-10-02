# PromptJang Relay One v0.4.0 — release candidate

- Optional Agent Message Envelope v1 validates task input and result output without rejecting legacy payloads.
- MCP tools advertise structured output schemas and provide structured content plus compatible text.
- Desktop updates require explicit confirmation and a trusted artifact signature.
- Recovery retains the previous application and checkpointed database, checks replacement readiness, and restores both on failed startup.
- Mailbox traffic is paused during update probation.

Publication gates: configure the trusted updater signing key, and complete actual
packaged upgrade/recovery UAT on macOS, Windows NSIS, and Linux AppImage.
v0.3 users install v0.4 manually once. No Cloud changes are included.

## Previous: v0.3.0

Relay One is easier to keep running, inspect, and connect to CLI agents.

- Closing the dashboard keeps Relay One available from the system tray; explicit Quit stops and checkpoints SQLite.
- Mailboxes can be searched by name. Retained messages can be searched by ID or payload and filtered by lifecycle state.
- MCP setup now distinguishes client configuration, verified adapter/API connectivity, and observed client activity.
- Guided client setup records activity without assigning a default mailbox.
- Existing mailbox, API, MCP, export/import, update prompt, and embedded documentation contracts remain intact.

Relay One stores work. It does not wake, run, or loop an agent.
