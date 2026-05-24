//! COP-P (Communications Operations Procedure — Proximity) per **CCSDS 235.1-W-0.4 §6**.
//!
//! - [`seq`] — §6.1 sequence arithmetic (modulo-256 / modulo-65536).
//! - [`farm`] — §6.3 FARM-P (receiver): events RE0–RE7.
//! - [`fop`] — §6.2 FOP-P (sender): events SE0–SE8, queues, transmission window.
//! - [`cop`] — Coordinator wiring FARM-P and FOP-P to [`crate::frame`] and [`crate::spdu`].

mod seq;
mod farm;
mod fop;
mod cop;

pub use seq::*;
pub use farm::*;
pub use fop::*;
pub use cop::*;
