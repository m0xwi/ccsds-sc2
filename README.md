# `ccsds-sc2`

Reference data structures and algorithms for **CCSDS 235.1-W-0.4** (Space Communications Session Control / Proximity-1 session control).

## Run tests

From the crate directory:

```powershell
cd C:\Users\wylie\CCSDS\ccsds-sc2
cargo test
```

Run only the interoperability suite:

```powershell
cargo test --test interoperability
```

## Interoperability focus (SPDU layer)

This crate includes an interoperability-oriented SPDU wire-format implementation:

- **Fixed-length SPDUs**: Type **F1** (16-bit PLCW) and **F2** (32-bit PLCW)
- **Variable-length SPDUs**: Types **1–5** (Type 1 directives implemented; others are parsed/serialized as defined in this crate)
- **Hex dump exchange**: parse/format raw octets for cross-implementation comparisons

See `docs/INTEROPERABILITY_TESTS.md` for the exact vectors and expected hex.

## Run examples

Print the workshop interoperability artifacts as hex:

```powershell
cargo run --example workshop_artifacts
```

Print a small SPDU/hex demo:

```powershell
cargo run --example spdu_hex_demo
```

