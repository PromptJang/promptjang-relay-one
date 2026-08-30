# Relay configuration

| Variable | Default | Purpose |
|---|---:|---|
| `DATABASE_URL` | required | PostgreSQL connection string |
| `PJ_ENCRYPTION_KEY` | required | Base64-encoded 32-byte AES-GCM key for signing secrets |
| `PJ_ADMIN_USERNAME` | `admin` | Owner username (re-applied at startup when changed) |
| `PJ_ADMIN_PASSWORD` | required | Owner password; ≥ 12 chars (≥ 1 with the weak flag). Changing it resets the owner and revokes sessions |
| `PJ_ALLOW_WEAK_PASSWORD` | `false` | Dev only: allow short passwords for local `admin/admin` |
| `PJ_MAX_PAYLOAD_BYTES` | `1048576` | Accepted body maximum |
| `PJ_RATE_LIMIT_PER_DESTINATION_PER_MINUTE` | `10000` | Per-destination safeguard; `0` disables |
| `PJ_EVENT_RETENTION_DAYS` | `30` | Terminal event retention; `0` retains forever |
| `PJ_WORKER_CONCURRENCY` | `8` | Concurrent delivery loops |
| `PJ_DELIVERY_TIMEOUT_SECONDS` | `15` | Outbound HTTP timeout |
| `PJ_RETRY_DELAYS_SECONDS` | `60,120,240,480,960` | Retry schedule |
| `PJ_STUCK_AFTER_SECONDS` | `300` | Interrupted-processing recovery threshold |
| `PJ_RESPONSE_BODY_BYTES` | `10240` | Stored response evidence maximum |
| `PJ_DB_MAX_CONNECTIONS` | `20` | PostgreSQL pool maximum |
| `PJ_SESSION_TTL_SECONDS` | `86400` | Owner-session lifetime |
| `PJ_DESTINATION_ALLOW_PRIVATE_CIDRS` | empty | Comma-separated private CIDR allowlist |
| `PJ_ALLOW_INSECURE_HTTP` | `false` | Allow HTTP only to allowlisted private addresses |
| `PJ_EXTRA_CA_CERT_PATH` | empty | PEM CA bundle for private TLS |
| `PJ_OTEL_ENABLED` | `false` | OpenTelemetry master gate — see [observability](observability.md) |

Deprecated: `PJ_ADMIN_EMAIL` is still read as a fallback for `PJ_ADMIN_USERNAME`.
