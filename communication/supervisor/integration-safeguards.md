# REALMATTER — Integration Safeguards

- Owner: A0
- Updated: 2026-11-04

## 1. Exclusive file ownership

- Each agent modifies only files under its owned paths (see protocol.md roster).
- Any change to a shared file (Cargo.toml workspace, protocol.md, architecture.md)
  requires explicit A0 approval and a request in `/communication/requests/pending/`.

## 2. Git worktrees

- Where practical, each agent works in a separate Git worktree to avoid
  concurrent edits to the same working tree.
- Worktree naming: `wt-<agent-id>` (e.g., `wt-A2`).
- A0 coordinates worktree creation; agents must not create worktrees in
  another agent's path.

## 3. Shared-file changes

- Explicit approval required from A0 before modifying any file not in the
  agent's owned paths.
- Approval recorded as a directive in `/communication/supervisor/directives/`.

## 4. Destructive Git operations

- FORBIDDEN: `git reset --hard`, `git checkout -- .`, `git clean -fd`,
  `git push --force`, `git rebase` on shared branches, deleting branches
  with unmerged work.
- Allowed: `git status`, `git diff`, `git add <specific files>`, `git commit`,
  `git log`, `git stash` (with care), `git worktree`.

## 5. Build verification before integration

- `cargo build` (workspace) must succeed with no errors.
- `cargo clippy` clean (or documented exception).
- `cargo test` (when tests exist) must pass.
- A8 verifies and reports results.

## 6. Regression checks

- Use existing tests (none yet; A8 to establish baseline after contracts).
- Any change that breaks a previously passing test blocks integration.

## 7. Supervisor approval before merging

- No agent merges or integrates changes without A0 approval.
- A0 publishes APPROVED review in `/communication/supervisor/reviews/`.
- Integration only after all quality gates pass (see protocol.md §9).

## 8. Conflict resolution

- On ownership/interface conflict: create request in
  `/communication/requests/pending/`, stop affected work, await A0 resolution.
- Never silently overwrite another agent's work.
