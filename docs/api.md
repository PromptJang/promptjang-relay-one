# Relay API and signing

The stable API uses `/api/v1` for administration and `/v1/destinations/:id/events` for ingestion. v0.1 `/api/*` and `/e/:id` routes remain deprecated aliases through v1.0 and send a `Deprecation` response header.

```text
Producer ──POST event──▶ Relay ──commit payload + queue──▶ PostgreSQL
Producer ◀────202────── Relay ──signed attempts + retries──▶ Receiver
```

`202` means Relay owns the item. It does not mean the receiver has processed it.

## Routes

| Area | Routes |
|---|---|
| Session | `POST /api/v1/session` · `DELETE /api/v1/session` |
| Destinations | `GET|POST /api/v1/destinations` · `GET|PATCH|DELETE /api/v1/destinations/:id` |
| Secret rotation | `POST …/:id/signing-secret/rotate` · `DELETE …/:id/signing-secret/previous` |
| Test delivery | `POST …/:id/test` |
| API keys | `GET|POST /api/v1/keys` · `GET /api/v1/keys/:id/secret` · `DELETE /api/v1/keys/:id` |
| Events | `GET /api/v1/events?cursor=&destination_id=&status=&event_type=&limit=` · `GET /api/v1/events/:id` · `POST /api/v1/events/:id/replay` |
| System | `GET /api/v1/system` |
| Ingestion | `POST /v1/destinations/:id/events` |

API keys are unrestricted when `destination_ids` is empty, or restricted to the listed destinations. The owner-only secret endpoint returns the full encrypted-at-rest key with `Cache-Control: no-store`. Keys created before encrypted retrieval cannot be recovered and must be replaced if their original value was lost.

## Ingest

```bash
curl -X POST "http://localhost:8080/v1/destinations/$DEST_ID/events" \
  -H "Authorization: Bearer pj_relay_YOUR_KEY" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: order-1042" \
  -H "X-Event-Type: order.created" \
  -d '{"order_id":"1042"}'
```

Returns `202` after PostgreSQL commits the payload and `QUEUED` state.

## Idempotency

`Idempotency-Key` is optional and scoped to one destination. Same key + exact payload bytes returns the original event; different bytes return `409`. Retries and replay never change this record.

## Standard Webhooks signature verification

```ts
import { Webhook } from "standardwebhooks"

const webhook = new Webhook(signingSecret)
webhook.verify(rawBody, {
  "webhook-id": request.headers.get("webhook-id"),
  "webhook-timestamp": request.headers.get("webhook-timestamp"),
  "webhook-signature": request.headers.get("webhook-signature"),
})
```

Relay signs the exact accepted bytes as `event_id.timestamp.payload`. The `webhook-id` stays stable across retries and changes for replay. Reject timestamps outside a five-minute tolerance and use `webhook-id` for receiver idempotency.

After rotation, `webhook-signature` contains current and previous space-separated `v1` signatures. Finish the rotation after the receiver accepts the new secret and in-flight retries have drained. `X-PromptJang-Event-Type` is optional delivery metadata and is not part of the Standard Webhooks signing headers.

## Retry and replay

Any non-2xx response or network failure schedules the next configured retry; a 2xx ends delivery. When the retry budget is exhausted the event becomes `EXPIRED`. Replay creates a new linked event with its own stable ID. Delivery is at-least-once — verify signatures and tolerate duplicates.
