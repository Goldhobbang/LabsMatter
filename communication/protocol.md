# REALMATTER — Shared Agent Protocol

This document is the single source of truth for inter-agent coordination.
All agents MUST read this file before modifying any code.

## 1. Project

REALMATTER is a high-performance, browser-based, offline physics and chemistry sandbox.
- Runtime: modern browsers (WebGPU preferred, WebGL2 fallback).
- Offline: no external APIs, AI services, or mandatory backend dependencies.
- Determinism: simulations must be deterministic given identical inputs and seed.
- Performance: 60 fps target for interactive scenes; budgets defined by A1 contracts.

## 2. Agent Roster

| ID | Role | Owns |
|----|------|------|
| A0 | Chief Supervisor | `/communication/**` (coordination only; no feature code) |
| A1 | Architecture & Contracts | `/src/core/**`, `/src/contracts/**`, shared types, build config |
| A2 | Physics Engine | `/src/physics/**` |
| A3 | Chemistry & Reactions | `/src/chemistry/**` |
| A4 | Rendering & Graphics | `/src/render/**` |
| A5 | UI & Interaction | `/src/ui/**` |
| A6 | Data, Persistence & Offline | `/src/data/**`, `/src/offline/**` |
| A7 | Performance & Optimization | `/src/perf/**`, benchmarks, profiling harness |
| A8 | Verification & Review | `/tests/**`, `/benchmarks/**` (read-only elsewhere) |

## 3. Mandatory Rules

1. Read `/communication/protocol.md`, your assignment, the latest supervisor directives, and relevant agent reports before modifying code.
2. Never modify files outside your explicitly assigned ownership paths.
3. Never overwrite, revert, reset, or delete another agent's work.
4. Never use destructive Git commands to resolve concurrent changes.
5. Use only approved interfaces and contracts.
6. If a dependency, interface, or file ownership conflict occurs, create a request in `/communication/requests/pending/` and stop the affected work.
7. Do not invent missing contracts or silently change public interfaces.
8. After each meaningful implementation milestone, run applicable verification and publish an immutable completion report.
9. Every report must include task ID, changed files, verification results, known limitations, blockers, and commit SHA if applicable.
10. Do not claim success without actual verification.
11. Do not create test cases or mocks yourself. Document recommended test scenarios and expected outcomes for the verification agent (A8) or human operator.
12. Continue independent, non-blocked tasks while waiting for dependencies.
13. Prioritize correctness, deterministic behavior, numerical stability, and performance.
14. Maintain offline operation. No external APIs, AI services, or mandatory backend dependencies.
15. Never commit or merge changes outside your owned files.

## 4. Communication Layout

```
/communication
  /protocol.md
  /agents/{A0..A8}
    /status.md
    /reports/{report-id}.md
  /supervisor
    /directives/{directive-id}.md
    /reviews/{review-id}.md
  /requests
    /pending/{request-id}.md
    /resolved/{request-id}.md
```

## 5. Report Format (immutable, append-only)

Filename: `{TASK_ID}-{AGENT_ID}-{YYYYMMDD-HHMM}.md`

Required sections:
- Task ID
- Agent ID
- Summary
- Changed files
- Verification results
- Known limitations
- Blockers
- Commit SHA (if applicable)
- Recommended test scenarios (for A8)

## 6. Status File

Each agent maintains `/communication/agents/{ID}/status.md` with:
- Current task ID
- State: IDLE | WORKING | BLOCKED | DONE
- Last updated timestamp
- Notes

## 7. Directives & Reviews

- Supervisor directives are authoritative. Agents must acknowledge by updating status.
- A8 performs project-wide review after every completed implementation milestone.
- Review outcomes: APPROVED | CHANGES_REQUESTED | BLOCKED | REJECTED.

## 8. Ownership Conflicts

- Create a request file in `/communication/requests/pending/`.
- Stop affected work until resolved by A0.
- Never silently modify another agent's files.

## 9. Quality Gates (integration approval)

All must pass:
- Type check / lint clean on changed files
- Build passes
- Tests pass (A8)
- Performance budgets met
- Offline constraint verified
- Determinism verified for physics/chemistry

## 10. Versioning

Protocol version: 1.0.0
Last updated: 2026-11-04
Owner: A0
