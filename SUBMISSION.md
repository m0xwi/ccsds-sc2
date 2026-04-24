# CCSDS 235.1-W-0.4 Reference Implementation — Submission Report

Replace all placeholder text in **[brackets]** with your team’s information.  
Delete any sections marked **(optional)** if not applicable.  
Submit this file as `**SUBMISSION.md`\*\* in the root of your repository.

## Team Information

| Field                     | Details                               |
| ------------------------- | ------------------------------------- |
| Team Name                 | [Your team name]                      |
| Members                   | [Name 1, Name 2, …]                   |
| Institution / Affiliation | [University, company, or independent] |
| Contact Email             | [Primary contact email]               |
| Repository URL            | [Link to your private repo]           |

## Implementation Overview

### Language & Technology Stack

| Gateway            | Status                                |
| ------------------ | ------------------------------------- |
| Primary Language   | Rust                                  |
| Build System       | Cargo                                 |
| Key Libraries      | [List any third-party libraries used] |
| Test Framework     | Rust built-in testing                 |
| Documentation Tool | Mermaid, rustdoc                      |
| Platform(s) Tested | Windows                               |

### Architecture Summary

[2–3 paragraphs describing your system architecture, layer structure, and key design decisions. Include a diagram if helpful.]

## Build & Run Instructions

### Prerequisites

[List all dependencies and how to install them.]

[package manager install commands, e.g.:]
[apt install ... / brew install ... / pip instal ...]

### Build

[exact build commands]

### Run Tests

[exact test commands]

### Run Examples

[exact commands to run caller/responder examples]

## Gateway Status

Mark each gateway as Complete, Partial, or Not Attempted. Provide a brief note for partial
implementations.

| Gateway                         | Status                                         |
| ------------------------------- | ---------------------------------------------- |
| 0 - Design & Architecture       | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 1 - SPDU Layer                  | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 2 - COP-P Layer (FOP-P, FARM-P) | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 3 - Frame Layer                 | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 4 - Physical Layer Abstraction  | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 5 - State Machine               | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 6 - Hailing & Session           | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 7 - Integration Testing         | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 8 - Conformance Testing         | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 9 - Interoperability Testing    | [ ] Complete / [ ] Partial / [ ] Not Attempted |
| 10 - Documentation & Examples   | [ ] Complete / [ ] Partial / [ ] Not Attempted |

## Validation Results

### Gateway 1: SPDU Layer

| Check                                                     | Pass/Fail            | Evidence   |
| --------------------------------------------------------- | -------------------- | ---------- |
| Type F1 PLCW encodes/decodes correctly                    | -------------------- | ---------- |
| Type F2 PLCW encodes/decodes correctly                    | -------------------- | ---------- |
| Variable-length SPDUs (Types 1-5) encode/decode correctly | -------------------- | ---------- |
| PLCWs generated from FARM-P state                         | -------------------- | ---------- |
| Directives decoded and processed                          | -------------------- | ---------- |
| Big-endian byte order verified                            | -------------------- | ---------- |
| Malformed SPDUs rejected gracefully                       | -------------------- | ---------- |
| Encoding/decoding < 1 ms per SPDU                         | -------------------- | ---------- |

### Gateway 2: COP-P Layer

| Check                                           | Pass/Fail            | Evidence   |
| ----------------------------------------------- | -------------------- | ---------- |
| FOP-P maintains V(S) correctly                  | -------------------- | ---------- |
| FOP-P transmit window management works          | -------------------- | ---------- |
| FOP-P retransmits on retransmit flag            | -------------------- | ---------- |
| FOP-P processes PLCWs correctly                 | -------------------- | ---------- |
| FARM-P maintains V(R) correctly                 | -------------------- | ---------- |
| FARM-P detects sequence gaps                    | -------------------- | ---------- |
| FARM-P sets retransmit flag on gap              | -------------------- | ---------- |
| Expedited service bypasses sequencing           | -------------------- | ---------- |
| Sequence Controlled service guarantees delivery | -------------------- | ---------- |
| Resynchronization (SET V(R)) works              | -------------------- | ---------- |
| 8-bit sequence numbers (modulo-256)             | -------------------- | ---------- |
| 16-bit sequence numbers (modulo-65536)          | -------------------- | ---------- |
| Persistence mechanism implemented               | -------------------- | ---------- |

### Gateway 3: Frame Layer

| Check                                | Pass/Fail            | Evidence   |
| ------------------------------------ | -------------------- | ---------- |
| P-frames transmitted correctly       | -------------------- | ---------- |
| U-frames transmitted correctly       | -------------------- | ---------- |
| Frames received and parsed correctly | -------------------- | ---------- |
| P-frames vs U-frames distinguished   | -------------------- | ---------- |
| SPDUs extracted from P-frames        | -------------------- | ---------- |
| QoS flags set correctly              | -------------------- | ---------- |
| Version-3 frames supported           | -------------------- | ---------- |
| Version-4 frames supported           | -------------------- | ---------- |
| CRC-16 validation works              | -------------------- | ---------- |
| Malformed frames rejected            | -------------------- | ---------- |
| Frame processing < 1 ms per frame    | -------------------- | ---------- |

### Gateway 4: Physical Layer Abstraction

| Check                                 | Pass/Fail            | Evidence   |
| ------------------------------------- | -------------------- | ---------- |
| Physical Channel interfaced defined   | -------------------- | ---------- |
| Mock channel works for unit testing   | -------------------- | ---------- |
| TCP channel enables ground testing    | -------------------- | ---------- |
| UDP channel enables loss testing      | -------------------- | ---------- |
| PCID support works (channels 0 and 1) | -------------------- | ---------- |
| Channel status reporting works        | -------------------- | ---------- |

### Gateway 5: State Machine

| Check                                                                  | Pass/Fail            | Evidence   |
| ---------------------------------------------------------------------- | -------------------- | ---------- |
| All states implemented (Init, Hailing, Data, Reconneting, Termination) | -------------------- | ---------- |
| Full Duplex state (Tables 5-1, 5-2)                                    | -------------------- | ---------- |
| Half Duplex state (Tables 5-1, 5-3)                                    | -------------------- | ---------- |
| Simplex (Tables 5-1, 5-4)                                              | -------------------- | ---------- |
| State transitions match CCSDS 235.1 tables exactly                     | -------------------- | ---------- |
| Invalid transitions rejected                                           | -------------------- | ---------- |
| State history logging implemented                                      | -------------------- | ---------- |

### Gateway 6: Hailing & Session Control

Check Pass/Fail Evidence
Hailing works (caller and responder)
Caller controller implemented
Responder controller implemented
Session lifecycle completes
Reconnect (rehailing) works
COMM_CHANGE support works
Session callbacks implemented
Thread safety verified

### Gateway 7: Integration Testing\*\*

Check Pass/Fail Evidence
Complete session lifecycle tests pass
Full Duplex integration tests pass
Half Duplex integration tests pass
Simplex integration tests pass
Reconnect integration tests pass
Resynchronization integration tests pass
Multi-channel tests pass
Performance benchmarks complete
Stress tests pass (no leaks)

### Gateway 8: Conformance Testing\*\*

Check Pass/Fail Evidence
All mandatory features implemented
All conformance tests pass
PICS document complete
Test vectors generated (binary + JSON)
Compliance report shows 100% conformance

### Gateway 9: Interoperability Testing

Check Pass/Fail Evidence
Session established with other implementation(s)
Data exchanged successfully
Test vectors compatible across implementations
Error recovery works across implementations
Interoperability report complete

### Gateway 10: Documentation & Examples\*\*

Check Pass/Fail Evidence
API documentation complete
User guide complete
Architecture documentation complete
Caller example works
Responder example works
Wire format specification documented

## Workshop Interoperability Artifacts

Provide hex dumps for the 5 required artifacts:

# Artifact Hex Dump Verified

1 Type F1 PLCW
(V(R)=127,
Retransmit=false,
PCID=0, Exp=3)

```
[hex bytes] [ ]
```

2 Type F2 PLCW
(V(R)=500,
Retransmit=true, PCID=1,
Exp=6)

```
[hex bytes] [ ]
```

3 Variable-Length SPDU
(Type 1, SET V(R),
SEQ_CTRL_FSN=42)

```
[hex bytes] [ ]
```

4 P-frame (Version-3,
containing artifact #1,
Expedited QoS)

```
[hex bytes] [ ]
```

5 U-frame (Version-3,
payload 0x00–0x09, Seq
Ctrl QoS, seq=7)

```
[hex bytes] [ ]
```

## Performance Benchmarks

Metric Your Result Target
SPDU encoding (per SPDU) [time] < 1 ms
SPDU decoding (per SPDU) [time] < 1 ms
Frame processing (per frame) [time] < 1 ms
Frame throughput [fps] > 100 fps
Session memory overhead [KB] < 50 KB
Concurrent sessions supported [count] >= 10
Sustained operation (duration) [time] No leaks
Test coverage [%] > 90%

```

[Describe your benchmarking methodology: hardware, OS, compiler flags, number of runs, etc.]

## Interoperability (optional)

Partner Team Test Performed Result

[Team name] [e.g. Session established, data
exchanged]

```

[Pass/Fail + notes]

```

[Team name] [e.g. Decoded their test vectors] [Pass/Fail + notes]

[Describe the interop testing process and any issues encountered.]

## Test Summary

```

Category Total Passing Failing Skipped
Unit tests
Integration tests
Conformance tests
Performance tests
Interoperability tests

```

[Paste or reference your test output here]

## PICS Summary

[Briefly summarise your Protocol Implementation Conformance Statement. Reference the full PICS
document location in your repository.]

```

Feature Area Mandatory Features Implemented Conformant
SPDU Layer [count] [count] [count]
COP-P Layer [count] [count] [count]
Frame Layer [count] [count] [count]
State Machine [count] [count] [count]
Session Control [count] [count] [count]

```

## Known Limitations

[List any known issues, incomplete features, or deviations from the specification.]

## Innovation & Extras (optional)

[Describe any additional features, optimisations, visualisations, or novel approaches your team im-
plemented beyond the base requirements. E.g. state machine visualiser, session timeline viewer,
etc.]

## File Manifest

List the key files and directories in your submission:

```

Path Description
README.md Project overview and quick start
SUBMISSION.md This submission report
src/ Source code
include/ Header files / public interfaces
tests/ Test suite
docs/ Documentation (API, architecture, user guide)
examples/ Caller and responder examples
test-vectors/ Generated test vectors (binary + JSON)
[other] [description]

```

## Self-Assessment

Rate your submission against the scoring criteria:

```

Category (Points) Self-Score Justification
Protocol Correctness (40) /
Testing & Conformance (25) /
Interoperability (15) /
Performance (10) /
Documentation & Usability (10) /
Total /

```

**Bonus Points Claimed (optional, up to 5)**

[Describe any exceptional work you believe merits bonus points: code quality, novel approaches,
specification contributions, visualisation tools, etc.]

## Additional Notes (optional)

[Any other information you’d like the judges to know.]
```
