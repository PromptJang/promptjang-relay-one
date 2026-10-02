# Structured agent messages

Agent Message Envelope v1 is an optional task/result contract shared with Relay.
Use `schema: "promptjang.agent-message.v1"`; unversioned text/JSON stays valid.
Invalid opted-in envelopes are rejected before durable acceptance.

Task:

```json
{"schema":"promptjang.agent-message.v1","kind":"task","correlation_id":"review-001","task":"Review this branch","reply_to":"results","constraints":["Read only"]}
```

Result:

```json
{"schema":"promptjang.agent-message.v1","kind":"result","correlation_id":"review-001","in_reply_to":"550e8400-e29b-41d4-a716-446655440000","status":"succeeded","summary":"No blocking findings"}
```

Send the object as `mail_push.payload`; claim returns `payload_json`.
MCP advertises output schemas and returns structured content alongside compatible
text. A result is a message payload, not the MCP transport response.

Send results before ack, using `result:SOURCE_MESSAGE_ID` as the idempotency key.
Failed results require `status: "failed"`, `summary`, and a sanitized `error`.
The envelope never authorizes an agent to execute arbitrary received instructions.
