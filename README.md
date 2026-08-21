# LocalDock

Secure local-only launcher for loopback dev servers (Windows + Linux).

## Network policy

LocalDock itself opens **no** listen ports and makes **no** outbound connections.
Child processes you start may use the network for their own needs; by default they are steered to bind `127.0.0.1` only.

## Docs

- Design: `docs/superpowers/specs/2026-08-21-localdock-design.md`
- Plan: `docs/superpowers/plans/2026-08-21-localdock.md`

## Status

Workspace and `localdock-core` crate bootstrapped (v0.1.0).
