# Relay operations

## Health

`/health` is process liveness. `/ready` verifies PostgreSQL. OpenTelemetry collector availability is intentionally excluded from readiness.

## Backup and restore

Back up PostgreSQL with your provider's tooling and `PJ_ENCRYPTION_KEY` through a separate secret manager. Test restore into an isolated database before relying on it. A database restore without the matching key preserves history but cannot decrypt signing secrets or retrievable API keys.

## External PostgreSQL

Relay depends only on `DATABASE_URL`; the Compose database is an example. RDS, Cloud SQL, Neon, Supabase, and other compatible services work directly. Use a dedicated database role, cap `PJ_DB_MAX_CONNECTIONS` below the provider limit, and terminate public HTTP at a reverse proxy with TLS.

## Upgrades

1. Back up PostgreSQL and the encryption key.
2. Run the new image against a restored staging copy (migrations apply automatically at startup).
3. Verify `/ready`, destination decryption, delivery, history, and replay.
4. Roll out: start one new instance, wait for `/ready`, verify a test delivery, then replace the rest.

## Scale out

Multiple Relay instances can share PostgreSQL. Queue claims use `FOR UPDATE SKIP LOCKED`. Delivery is at-least-once: a receiver may see the same event ID twice if it accepted a request before Relay recorded success.

## Monitoring

Monitor queue depth, queue delay, delivery duration and outcome, retries, expired events, recovery, and cleanup through OTLP (see [observability](observability.md)). Structured stdout logs remain available with telemetry disabled.

## Troubleshooting

| Symptom | Check |
|---|---|
| `/health` fails | Process is unavailable — restart the container. |
| `/ready` fails | `DATABASE_URL`, network policy, TLS mode, credentials, pool capacity. |
| Events stay queued | System worker state, database connections, destination status, logs. |
| Repeated retries | Event timeline and response evidence, DNS, CIDR allowlist, custom CA, receiver verification. |
| Telemetry export fails | Collector reachability and OTLP headers — delivery and readiness are unaffected. |
