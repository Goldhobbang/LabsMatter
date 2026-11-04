# REALMATTER — Interface Change Policy

**Version:** 0.1.0
**Owner:** A1

## Scope

Applies to all public types, traits, and functions in `realmatter-core` and the WASM entrypoint.

## Process

1. Agent identifies need for interface change.
2. Agent creates a request in `/communication/requests/pending/` describing the change, rationale, and affected modules.
3. A1 reviews; if breaking, escalates to A0 for approval.
4. On approval, A1 updates contract docs and core crate; bumps version.
5. Dependent agents migrate.

## Versioning

- Contracts use semver (MAJOR.MINOR.PATCH).
- Breaking change ⇒ MAJOR bump.
- Additive change ⇒ MINOR bump.
- Doc/clarification ⇒ PATCH bump.

## Rules

- No silent interface changes.
- No agent modifies contracts it does not own.
- Breaking changes require migration notes.
- Deprecated items remain for at least one minor version.