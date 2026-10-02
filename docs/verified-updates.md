# Verified desktop updates

Relay One v0.4 adds explicit-confirmation desktop updates. Browser/CLI users
continue installing manually. v0.3 cannot install itself; install v0.4 manually once.

The app rechecks the official manifest, requires the version the user confirmed,
and verifies the artifact against a public key pinned at build time. It refuses
unsigned artifacts, unexpected repository URLs, and builds without a trusted key.
Release artifacts also retain the existing Sigstore provenance and checksum files.
The in-app verifier uses Tauri's signature verifier, not the Sigstore CLI.
It also checks the manifest's repository/workflow/tag metadata and artifact SHA-256.
Artifact authentication comes from the pinned signing key; metadata alone is
not a cryptographic identity assertion.

After download verification, mailbox service stops and checkpoints SQLite.
A private recovery directory retains the previous application and database.
An independent old-version helper starts the replacement and checks the exact
version and database readiness. Mailbox and management operations return 503
during probation, so rollback cannot discard newly accepted messages.
Failed startup restores both executable/application bundle and database.
The master encryption key is never replaced.

One previous completed update is retained. Interrupted updates retain recovery
evidence rather than accepting traffic against an unverified database.
Do not delete `update-pending.json` to bypass a failed update.

## Release operator requirements

Configure the repository variable `PJ_ONE_UPDATER_PUBLIC_KEY` and secrets
`TAURI_SIGNING_PRIVATE_KEY`, optionally `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
Generate/store the private key outside the repository with Tauri's signer.
The public key is embedded in each release; retain its private counterpart safely.
CI refuses publishing unsigned updater builds and requires all five target artifacts.

Before publishing v0.4, test actual packaged upgrades and startup failure recovery
on macOS, Windows NSIS, and Linux AppImage. Never describe source/unit checks as
cross-platform installation UAT. Existing installed deb/rpm distributions update
through their package manager, not by replacing an extracted executable.
v0.4 Windows builds use NSIS; older MSI installations migrate manually once.
