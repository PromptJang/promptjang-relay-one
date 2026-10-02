# Operations

Closing the dashboard window hides it while the loopback mailbox stays available. Reopen it from the Relay One system-tray icon. Choose **Quit Relay One** from the tray before copying its data directory.

Back up both `relay-one.db` and `master.key`; API keys cannot be decrypted without the matching key.

Use `export --output mailbox.json` for a permission-restricted mailbox archive. Import requires an installation with no mailbox data:

```bash
promptjang-relay-one import --input mailbox.json
```

Desktop v0.4 updates require confirmation and a verified artifact signature; see
[verified updates](verified-updates.md) for restart recovery and release gates.
Browser/CLI users install manually. Quit before replacing the application;
the SQLite schema migrates on the next launch.
