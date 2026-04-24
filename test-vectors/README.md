# Test vectors

This directory holds **frozen interoperability artifacts** (JSON metadata + optional binary exports).

For the high-level overview, see [`test-vectors.md`](./test-vectors.md).

## SPDU binary export (`*.bin`)

When present, `*.bin` files follow the interoperability brief's **SPDU Export Format**:

- 64-byte header
- followed by the raw SPDU bytes

Header layout (all big-endian for integers):

- Magic: `CCSDS\0\0\0` (8 bytes)
- Version: `1` (u32) (4 bytes)
- SPDU Type: ASCII padded to 4 bytes (4 bytes), e.g. `F1\0\0`, `F2\0\0`, `1\0\0\0`
- SPDU Length: N (u32) (4 bytes)
- Timestamp: Unix epoch seconds (u64) (8 bytes)
- Reserved: zeros (36 bytes)

# Interoperability test vectors

This folder contains **frozen, shareable test vectors** intended for cross-implementation interop testing
(Rust/C/C++/Python/…).

It follows the **"Standardized Test Vector Format"** described in the interoperability brief.

## Structure

- `spdus/` — SPDU-only artifacts
  - `type_f1/`
  - `type_f2/`
  - `type_1/`

## Files

Each vector has two files:

- `*.json` — human- and machine-readable metadata, including expected fields.
- `*.bin` — binary export format:
  - 64-byte header
  - followed by the raw SPDU bytes

### Binary header layout (64 bytes)

- Magic: `CCSDS\0\0\0` (8 bytes)
- Version: `1` (u32, big-endian) (4 bytes)
- SPDU Type: ASCII, NUL-padded to 4 bytes (4 bytes)
  - `F1\0\0`, `F2\0\0`, `1\0\0\0`, etc.
- SPDU Length: N (u32, big-endian) (4 bytes)
- Timestamp: Unix epoch seconds (u64, big-endian) (8 bytes)
- Reserved: zeros (36 bytes)

All **SPDU bytes** themselves are the crate's canonical on-wire encoding (big-endian for multi-byte fields).

