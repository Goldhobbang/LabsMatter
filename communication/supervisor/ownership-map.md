# Ownership Map — Consolidated

- Owner: A0
- Updated: 2026-11-04
- Status: Active

| Agent | Role | Owned paths | Notes |
|-------|------|-------------|-------|
| A0 | Supervision and coordination | `/communication/**` (coordination only) | No feature code |
| A2 | Architecture, Core, World, Persistence | `docs/**`, `Cargo.toml` (workspace), `.cargo/**`, `rust-toolchain.toml`, `crates/realmatter-core/**`, `crates/realmatter-world/**`, `crates/realmatter-persist/**` | Former A1 and A7 responsibilities merged |
| A3 | Physics and Chemistry | `crates/realmatter-physics/**`, `crates/realmatter-chemistry/**` | Former A4 responsibilities merged |
| A5 | Renderer and UI | `crates/realmatter-renderer/**`, `crates/realmatter-ui/**`, `web/**` | Former A6 responsibilities merged |
| A8 | Verification and integration audit | `tests/**`, `benchmarks/**` (read-only elsewhere) | No feature implementation |

## Retired/merged
- A1: Architecture/contracts work merged into A2
- A4: Chemistry merged into A3
- A6: UI crate merged into A5 (web shell already under A5)
- A7: Persistence merged into A2

Historical reports and decisions preserved in `/communication/agents/*/reports/` and `/communication/supervisor/`.
