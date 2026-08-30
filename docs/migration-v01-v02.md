# Upgrade from a pre-release build

Relay v0.2 establishes a fresh database baseline. If you ran the v0.1 technical preview or a pre-baseline v0.2 build from source, migrate by exporting what you need and re-importing into a new installation — automatic in-place upgrade is not supported for pre-release databases.

1. Back up any data you want to keep from the old installation.
2. Start the current image with an empty database and a fresh `PJ_ENCRYPTION_KEY`.
3. Recreate destinations, keys, and owner credentials; re-ingest events if needed.

For new installations, `docker compose up` is all you need — the schema installs automatically.
