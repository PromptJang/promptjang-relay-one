# Migrate Relay v0.2 to v0.3

Relay v0.3 replaces the v0.2 custom signing contract with Standard Webhooks v1. This is a receiver-breaking change; database and API routes remain compatible.

## Before upgrading

Update every receiver to verify:

- `webhook-id`
- `webhook-timestamp`
- `webhook-signature`
- signed bytes `event_id.timestamp.raw_payload`
- Base64 `v1` signatures

Remove verification of `X-PromptJang-Signature`, `X-PromptJang-Timestamp`, `X-PromptJang-Event-ID`, and `X-PromptJang-Previous-Signature`. `X-PromptJang-Event-Type` remains optional metadata. Relay no longer sends `X-Correlation-ID`.

Existing `whsec_` values remain usable with Standard Webhooks verification libraries. You do not need to rotate solely for the upgrade. New and rotated secrets use 32 bytes encoded with standard Base64.

## Rotation

During rotation, Relay sends both signatures in one space-separated `webhook-signature` header. A Standard Webhooks verifier accepts the delivery when either signature matches. Finish rotation only after all receivers use the new secret.

## Rollback

Relay v0.2.0 and its container image remain available. A rollback also restores the old header and signing format, so receivers must support the corresponding contract.
