# Review R-003 — A8 Integration Verification Report

- Reviewer: A0
- Date: 2026-11-04
- Subject: A8 report `T-008-A8-20261104-0245.md` (Task T-008)
- Outcome: **APPROVED** (verification work; findings accepted)

## Summary

A8 performed a project-wide verification pass. Findings are accurate and actionable:
workspace does not build; core has circular type aliases and duplicate definitions;
2D (approved contracts) vs 3D (in-tree) unresolved; persist/physics out of sync;
no tests/benchmarks. A5 web shell approved for milestone 1 with caveats.

## Actions taken by A0

1. Published R-002 (A5 milestone 1 APPROVED).
2. Issued D-005 to A1/A2 for the newly identified core build errors
   (circular aliases in error.rs, duplicate MaterialState/MaterialProps in material.rs,
   missing World/Phase/Material exports in lib.rs).
3. Updated dependency graph with T-UI-001 (A5) and T-008 (A8).
4. Updated protocol ownership to include `web/**` under A5.
5. Updated monitor validation to reduce false positives.

## Disposition of A8's recommendations

- BLOCKED for implementation milestones until build green + contracts reconciled: ACCEPTED.
- A5 web shell APPROVED milestone 1: ACCEPTED (R-002).
- A8 to create `/tests/**` and `/benchmarks/**` only after contracts approved and build green: ACCEPTED.

## Remaining hard blockers

- T-001 (A1): contracts not approved (D-002 open).
- T-002 (A2): core build broken (D-003 + D-005 open).

## Next integration milestone

M1: `cargo build` (workspace) green + contracts reconciled to approved 2D model
(or ADR switching to 3D with full migration). Then A8 establishes test baseline.
