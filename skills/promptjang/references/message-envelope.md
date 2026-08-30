# Agent message envelope

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

Only `task` is essential. Use `reply_to` only when the sender expects a result through PromptJang. Its value is a Relay mailbox name or a Cloud mailbox target ID, depending on the active tool dialect.

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
