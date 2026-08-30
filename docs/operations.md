# Operations

Stop Relay One before copying its data directory. Back up both `relay-one.db` and `master.key`; API keys cannot be decrypted without the matching key.

Use `export --output mailbox.json` for a permission-restricted mailbox archive. Import requires an installation with no mailbox data:

```bash
promptjang-relay-one import --input mailbox.json
```

Update prompts link only to the official PromptJang Relay One GitHub release. Replace the executable while Relay One is stopped; the SQLite schema migrates on the next launch.
