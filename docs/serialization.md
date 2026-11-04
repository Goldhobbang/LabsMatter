# REALMATTER — Serialization & Compatibility

**Version:** 0.1.0
**Owner:** A1

## Format

World state is serialized as a versioned binary blob.

Header (little-endian):
- magic: `u32` = 0x524D5452 ("RMTR")
- format_version: `u32`
- width: `u32`
- height: `u32`
- chunk_size: `u32`
- cell_count: `u64`
- reserved: 16 bytes

Body: packed `Cell` array (width*height cells), row-major (y outer, x inner).

## Versioning

- `SerializationModule::version()` returns the format version produced.
- Reader must accept current version and N-1 (one version back).
- On version mismatch beyond tolerance: return `RealmatterError::VersionMismatch { expected, found }`.
- Material registry is serialized by id only; material definitions come from the bundled manifest.

## Compatibility policy

- Adding fields: bump minor version; reader fills defaults for missing fields.
- Removing/renaming fields: bump major version; migration required.
- No silent data loss. Unknown trailing bytes are ignored (forward compat).