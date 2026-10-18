//! theme-kit — the typed token contract (specs/theme-kit.md, ADR-0004 §2).
//!
//! One graph, one gate, data-only emitters. Authored colours enter through
//! the closed [`Role`] vocabulary; component and state tokens derive from
//! them; every semantic pair is WCAG-gated at build time; the variants
//! (light, high-contrast, monochrome, dichromat-adapted) are *derived* from
//! one seed, never hand-authored.
//!
//! | REQ | Where |
//! |---|---|
//! | THM-001 DTCG 2025.10 round-trip | [`TokenGraph::from_dtcg_json`] / [`to_dtcg_json`] |
//! | THM-002 closed roles | [`Role`] — unknown token paths are typed errors |
//! | THM-003 contrast gate | [`audit`] — `ContrastViolation` *is* the build failure |
//! | THM-004 lightness ranks | [`audit`] with a CVD pass; [`audit::LC_FLOOR`] |
//! | THM-005 CVD rank preservation | [`audit`] — including the status-role collision check |
//! | THM-006 variant derivation | [`derive`] + [`Variant`] |
//! | THM-007 emitters | [`Emitter`], [`CssVars`], [`DtcgJson`], [`VsCodeTheme`], [`Ansi16`] |
//! | THM-008 toolkit adapters | one-file L2 crates (not here, by design) |
//! | THM-009 determinism | fixed role order, fixed hex formatting |
//!
//! ```
//! use theme_kit::{audit, TokenGraph, Role};
//!
//! let mut theme = TokenGraph::new("midnight");
//! theme.set_hex(Role::Surface, "#0d1117").unwrap();
//! theme.set_hex(Role::OnSurface, "#e6edf3").unwrap();
//! // …the rest of the vocabulary…
//! # let report = audit(&theme, None).ok();
//! # assert!(report.is_none_or(|a| a.edges > 0)); // incomplete themes fail loudly
//! ```
//!
//! # Layer
//!
//! L1 — substrate; the only cross-L2 coupling in the UI family. Estate dep:
//! `color-kit` (L0) alone. No toolkit dependency in the core — egui,
//! ratatui, Slint, and Leptos lower through separate adapter crates.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod audit;
mod emit;
mod error;
mod graph;
mod role;
mod variant;

pub use audit::{audit, Audit, LC_FLOOR, ROLE_DELTA_FLOOR};
pub use emit::{Ansi16, CssVars, DtcgJson, Emitter, VsCodeTheme};
pub use error::ThemeError;
pub use graph::TokenGraph;
pub use role::Role;
pub use variant::{derive, Variant};

/// The crate's version, for callers that report it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
