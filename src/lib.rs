//! # Proximity session control — reference structures
//!
//! This crate implements data structures and algorithms used by **CCSDS 235.1-W-0.4**
//! *Space Communications Session Control* (Proximity-1 session control) alongside related
//! **Proximity-1 Space Data Link** framing concepts from **CCSDS 211.0** (transfer frames).
//!
//! ## Normative reference
//!
//! The authoritative protocol definition is **CCSDS 235.1-W-0.4** (White Book). This code is a
//! software aid; where behavior differs from the published standard, the standard wins.
//!
//! ## Specification map
//!
//! | CCSDS 235.1 section | This crate |
//! | --- | --- |
//! | **§2 Overview** — SPDUs, data services, COP-P | [`spdu`], [`cop_p`] |
//! | **§3 Supervisory Protocol Data Units** — fixed (F1/F2) and variable-length SPDUs | [`spdu`] |
//! | **§4 Persistence** — persistent activity timers and notifications | *Not implemented here* (see spec §4.2–4.3) |
//! | **§5 Data services** — transfer frames and CRC | [`frame`] |
//! | **§6 COP-P** — FOP-P (send), FARM-P (receive) | [`cop_p`] |
//!
//! Related **211.0** material: Version-3/4 **Transfer Frames**, **ASM**, coding sublayer — partly
//! reflected in [`frame`] (header models and CRC-16 PLTU framing for testing).
//!
//! ## Modules
//!
//! - [`spdu`] — Encode/decode **SPDU**s (§3), including Type F1/F2 **PLCWs** and variable types 1–5.
//! - [`cop_p`] — **COP-P**: [`FopP`] (§6.2) and [`FarmP`] (§6.3).
//! - [`frame`] — **P-frame** / **U-frame** transfer frames with QoS and CRC-16 (§5.4–5.6, 211.0 frames).
//!
//! ## Abbreviations (from §1.5)
//!
//! **SPDU** — Supervisory Protocol Data Unit; **PLCW** — Proximity Link Control Word; **COP-P**
//! — Communications Operations Procedure — Proximity; **FOP-P** — Frame Operation Procedure;
//! **FARM-P** — Frame Acceptance and Reporting Mechanism; **PCID** — Physical Channel ID;
//! **QoS** — Quality of Service (Expedited vs Sequence Controlled).

// This is the top-level module for the crate.
// It is the entry point for the crate.
// It contains the public items that are exported from the crate.

// Declares and includes the modules in the order they are used.
pub mod cop_p;
pub mod frame;
pub mod spdu;
pub mod wire;

// Exports the public items from the modules.
// Allows these items to be used in a shorter path... e.g. use crate::spdu::*;
pub use cop_p::*;
pub use frame::*;
pub use spdu::*;
pub use wire::*;
