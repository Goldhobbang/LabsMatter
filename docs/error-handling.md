# REALMATTER — Error Handling

**Version:** 0.1.0
**Owner:** A1

## Error type

`RealmatterError` (thiserror) with variants:
- `InvalidDimensions(u32, u32)`
- `ChunkOutOfBounds(i32, i32)`
- `CellOutOfBounds(i32, i32)`
- `UnknownMaterial(u16)`
- `Serialization(String)`
- `Deserialization(String)`
- `VersionMismatch { expected, found }`
- `Io(String)`
- `Internal(String)`

Alias: `pub type Result<T> = std::result::Result<T, RealmatterError>;`

## Rules

- Core returns `Result<T, RealmatterError>`; no panics in release hot path.
- WASM boundary converts `RealmatterError` to `JsValue` (JS exception).
- Invalid config fails fast at `init` (no partial state).
- Out-of-bounds access returns error; never UB.
- `unsafe` blocks must be audited and justified; all `unsafe` requires A1 review.
- Empty catch blocks are forbidden.