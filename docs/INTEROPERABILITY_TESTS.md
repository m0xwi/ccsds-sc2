# Interoperability tests

This document explains how `ccsds-sc2/tests/interoperability.rs` maps to the CCSDS 235.1-W-0.4 reference-implementation functional requirements and the interoperability testing overview.

## How to run

From the crate directory:

```powershell
cargo test --test interoperability
```

## Conventions

- **Byte order**: big-endian for all multi-byte fields.
- **Hex format (canonical)**: lowercase, no separators (e.g. `c00504d2`).
- **Hex parser** accepts: `0x` prefixes, whitespace, and `_` separators (for copy/paste interoperability).

## Reference vector vs encoder output

Each SPDU test names a **reference vector** as a `const VECTOR_HEX` (or `ARTIFACT_*_HEX`) and compares it to `spdu.to_bytes()` using `assert_spdu_bytes_match_vector` in `tests/interoperability.rs`. On failure, the panic lists **both** the reference hex/bytes and the encoder output so you can see exactly what diverged from the frozen interoperability bytes.

The same comparison pattern is used in `src/spdu/mod.rs` unit tests for workshop / INTOP vectors.

## Test vectors (SPDUs)

### Fixed-length SPDU: Type F1 (16-bit PLCW)

Test: `interop_fixed_length_f1_known_vector_bytes`

Input fields:
- V(R) = 42
- expedited = 3
- PCID = 1
- retransmit = true

Expected wire bytes:
- hex: `b32a`
- bytes: `b3 2a`

### Fixed-length SPDU: Type F2 (32-bit PLCW)

Test: `interop_fixed_length_f2_known_vector_bytes`

Input fields:
- V(R) = 1234
- expedited = 5
- PCID = 0
- retransmit = false

Expected wire bytes:
- hex: `c00504d2`
- bytes: `c0 05 04 d2`

### Variable-length SPDU: Type 1 directive — SET V(R)

Test: `interop_variable_length_type1_set_vr_workshop_artifact`

This matches the workshop artifact:
- “Variable-Length SPDU Type 1 Directive, SET V(R) with SEQ_CTRL_FSN = 42”

Expected wire bytes:
- header: type_id=0 (Type 1), len=2 octets → `0x02`
- body: SET V(R) directive word → `0x602a` → `60 2a`
- hex: `02602a`

## Validation behavior

### Fixed-length size rejection

Test: `interop_spdu_validation_rejects_bad_fixed_length_size`

Rule:
- Fixed-length SPDUs must be exactly **2** (F1) or **4** (F2) octets.

## Hex dump interoperability

Test: `interop_hex_dump_roundtrip_accepts_separators_and_prefix`

Examples accepted by the parser:
- `0x02 60_2a`
- `02602a`

## Frame interoperability (payload preservation)

Test: `interop_frame_v3_pframe_contains_spdu_bytes`

Rule:
- The transfer frame **payload** is treated as a raw octet string. The bytes are preserved end-to-end across encode/decode.

