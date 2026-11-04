# A8 Verification Readiness Document

- Owner: A8 (Verification & Review)
- Date: 2026-11-04
- Protocol: REALMATTER Shared Agent Protocol v1.0.0
- M2 Plan: milestone-2-plan.md

## Baseline
- Last verified commit: 1ffdf3a (A3 physics/chemistry reports)
- Latest report: T-008-A8-20261104-0415.md
- Status: WAITING_FOR_CHANGES

## Verification Triggers (restart conditions)
1. **A2 reports Persistence recovery** (D-007) — A2 publishes completion report for persist fix
2. **A2 completes WASM boundary** — REQ-A5-001 resolved or WASM/JS contract published
3. **A3 changes Physics or Chemistry interfaces** — new report from A3 modifying traits/APIs
4. **A5 reports production renderer/browser integration completion** — new report from A5
5. **A0 requests milestone acceptance review** — directive/review from A0

## Supported Verification Commands
Only use commands that exist in the repo:

**Rust:**
- `cargo build -p realmatter-core` 
- `cargo build -p realmatter-physics`
- `cargo build -p realmatter-chemistry`
- `cargo build -p realmatter-persist`
- `cargo build --workspace`
- `cargo check --workspace`
- `cargo test -p <crate>` (if tests exist)
- `cargo clippy -p <crate>` (if clippy available)
- `rustc --version`, `cargo --version`

**Web:**
- `node --check web/src/**/*.js` (or check individual files)
- HTTP smoke test via python/http server as needed

**WASM (if tools available):**
- Check for wasm-pack/trunk in PATH; do not assume installed

## Evidence Requirements (MANDATORY)
For every verification:
- Exact commit SHA (from `git log -1 --oneline` or full SHA)
- Commands executed (full command line)
- Exit codes for each command
- Relevant compiler/runtime output (errors, warnings summary, test counts)
- Number of tests actually executed (0 if none; do NOT say "passed" if 0)
- Unverified behaviors
- Remaining blockers
- Changed modules compared to baseline

**Rules:**
- Successful build ≠ correct runtime behavior
- 0 executed tests = 0 coverage, NOT functional validation
- Do not create tests/mocks
- Do not repeat identical failing builds without relevant changes
- Only publish report if new evidence or actionable findings exist

## Current Blockers
- `realmatter-persist` API mismatch (D-007, A2 owned)
- WASM boundary contract pending (REQ-A5-001)

## Next Action
Remain in WAITING_FOR_CHANGES until triggered. Monitor for A2 completion reports.
