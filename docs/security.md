# Relay security

Relay is designed for one trusted team. Put the UI and administration API behind TLS and network access controls. A reverse proxy may provide SSO; Relay still uses its local owner session.

**Secrets at rest**

- Signing secrets are AES-GCM encrypted with `PJ_ENCRYPTION_KEY`.
- API keys retain a SHA-256 hash for authentication and an AES-GCM encrypted copy for owner retrieval. Sessions are hash-only; owner passwords use Argon2id.
- Keep the encryption key, bootstrap password, and database password in a secret manager; back the key up separately — losing it makes destination secrets unrecoverable.

**Destination restrictions**

- Public HTTPS destinations work by default.
- Private addresses require explicit CIDR configuration and are revalidated before every delivery.
- HTTP is limited to allowlisted private destinations and requires `PJ_ALLOW_INSECURE_HTTP=true`.
- Redirects are disabled; embedded URL credentials are rejected.

**Telemetry**

Payload bodies, authorization headers, cookies, secrets, encryption keys, database credentials, and OTLP authorization headers are never recorded or exported. Receivers must verify signatures and tolerate duplicate delivery.
