# PromptJang Relay One v0.3.0

Relay One is easier to keep running, inspect, and connect to CLI agents.

- Closing the dashboard keeps Relay One available from the system tray; explicit Quit stops and checkpoints SQLite.
- Mailboxes can be searched by name. Retained messages can be searched by ID or payload and filtered by lifecycle state.
- MCP setup now distinguishes client configuration, verified adapter/API connectivity, and observed client activity.
- Guided client setup records activity without assigning a default mailbox.
- Existing mailbox, API, MCP, export/import, update prompt, and embedded documentation contracts remain intact.

Relay One stores work. It does not wake, run, or loop an agent.
