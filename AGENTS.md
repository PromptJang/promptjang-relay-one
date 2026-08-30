# AGENTS.md — promptjang-relay

Self-hosted webhook delivery on PostgreSQL. One Rust binary (Axum + SQLx), Vue UI, Apache-2.0.

## Workflow rules

- **Never commit to `main`.** Always: `git checkout -b <branch>` → commit → push → PR → merge. Verify the branch is correct with `git branch --show-current` before committing.
- One logical change per PR. Squash-merge and delete the branch.
- Land green only: `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cd web && npm run build` before pushing.
- No production `unwrap`/`expect`/`panic!` (a unit test enforces this for the worker).
- Schema changes: add a new SQLx migration; never edit an applied one (the single-baseline window closed when the GHCR package became public).
- Run external-state changes (image publishes, GHCR visibility) only with action-time owner confirmation, per the workspace `AGENT_HANDOFF.md`.

## Commands

```bash
cargo test --workspace                               # Rust unit tests (relay + mcp)
cargo fmt && cargo clippy --all-targets -- -D warnings
cd web && npm run build                               # vue-tsc + vite build
docker compose up -d --build                          # local stack on :8080
```

## Layout

```
src/domain/    pure policy: validation, secrets, delivery rules, errors
src/store/     PostgreSQL persistence (auth, destinations, events, keys)
src/api/       axum edge: state, error mapping, per-resource handlers
src/worker/    delivery run loop + policy
mcp/           MCP stdio server for the agent mailbox (reuses the store layer)
migrations/    SQLx migrations (auto-applied at startup)
docs/          operator documentation
web/           Vue operational UI
```

## Deployment

Releases are tag-driven (`v*`): CI verifies, builds the signed multi-arch image, and publishes to `ghcr.io/promptjang/promptjang-relay` with SBOM and provenance.
