//! **Communications Operations Procedure — Proximity (COP-P)** per **CCSDS 235.1-W-0.4 §6**.
//!
//! COP-P provides reliable **Sequence Controlled** delivery using sequence numbers and **PLCWs**,
//! and a separate **Expedited** path. It splits into:
//!
//! - **FOP-P** ([`FopP`]) — **Frame Operation Procedure** at the sender (**§6.2**): `V(S)`, transmit
//!   window, retransmission, PLCW processing.
//! - **FARM-P** ([`FarmP`]) — **Frame Acceptance and Reporting Mechanism** at the receiver
//!   (**§6.3**): `V(R)`, gap detection, PLCW report generation.
//!
//! # Persistence
//!
//! **§4** defines the *persistence activity process* (timers such as waiting period, response,
//! lifetime). That MAC-layer behavior is not implemented in this module; FOP-P/FARM-P here cover
//! the sequence-control core of **§6** only.
//!
//! # See also
//!
//! - [`crate::spdu`] for PLCW **Type F1/F2** encoding carried in P-frames.

mod farm;
mod fop;
mod shared;

pub use farm::*;
pub use fop::*;
pub use shared::*;
