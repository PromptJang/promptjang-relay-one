# Agent Message Envelope v1

Opt in with `"schema": "promptjang.agent-message.v1"`. See
[JSON Schema](agent-message-v1.schema.json). Existing plain text, arbitrary JSON,
and the unversioned examples below remain valid.

Task input requires `schema`, `kind: "task"`, `correlation_id`, and `task`.
Result output requires `schema`, `kind: "result"`, the same `correlation_id`,
`in_reply_to` (source message UUID), `status` (`succeeded` or `failed`),
and `summary`. Failed results also require a sanitized `error`.

Pass an envelope directly as `mail_push.arguments.payload`, not a JSON string.
Read it from `mail_claim`'s `payload_json`. Successful MCP responses provide
`structuredContent` plus text content for older clients. This transport wrapper
is separate from the agent envelope. No new tool or agent execution is introduced.

Versioned messages are validated before acceptance; invalid ones return an error.
Send results with `idempotency_key: "result:SOURCE_MESSAGE_ID"` before ack.
Messages remain untrusted data, never authority to execute.

## Legacy-compatible examples

PromptJang accepts plain text or JSON. Use this small JSON envelope when agents need a predictable handoff. It is a convention for agents, not a new Relay protocol.

## Work message

```json
{
  "kind": "task",
  "task": "Review the current branch for correctness and report blocking findings.",
  "sender": "claude-code",
  "reply_to": "claude-results",
  "correlation_id": "review-2026-08-30-01",
  "context": {
    "workspace": "/workspace/project",
    "branch": "feat/example"
  },
  "constraints": [
    "Read-only review",
    "Do not push or merge"
  ],
  "artifacts": [
    "src/worker.rs"
  ]
}
```

Only `task` is essential. Use `reply_to` only when the sender expects a result through PromptJang. Its value is a Relay or Relay One mailbox name.

Do not place credentials or large file contents in the envelope. Paths and repository references are context, not automatic permission to read outside the user's authorized workspace.

## Result message

```json
{
  "kind": "result",
  "in_reply_to": "SOURCE_MESSAGE_ID",
  "correlation_id": "review-2026-08-30-01",
  "status": "succeeded",
  "summary": "No blocking correctness findings.",
  "artifacts": []
}
```

For a permanent failure, use `status: "failed"` and include a concise `error`. Do not include stack traces or secrets unless the user explicitly needs a sanitized diagnostic.

Use a stable result idempotency key derived from the source message ID, such as `result:SOURCE_MESSAGE_ID`. Send the result before acknowledging the source so a consumer crash cannot silently lose the outcome.
