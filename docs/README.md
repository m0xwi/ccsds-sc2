# Documentation (API, architecture, user guide)

The wire module is the "shared boundary" for bytes. It defines the smallest set of traits/helpers that any layer can use to agree on how to turn a typed value into on-wire octets and back:
- WireEncode::to_wire_bytes() and WireDecode::from_wire_bytes() are the generic interface
- HexDump + bytes_to_hex/hex_to_bytes are for inter

# Documentation index (`ccsds-sc2`)

| Document | Description |
|----------|-------------|
| [`INTEROPERABILITY_LAYERS.md`](./INTEROPERABILITY_LAYERS.md) | **How to interoperate** SPDU with frame, COP-P, and other implementations (bytes vs typed, APIs, pitfalls). |
| [`INTEROPERABILITY_TESTS.md`](./INTEROPERABILITY_TESTS.md) | **Test vectors**, expected hex, and how the `interoperability` test binary works. |

Project overview and commands: repository root [`README.md`](../README.md).
