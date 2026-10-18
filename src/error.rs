//! Typed errors for theme construction and the gate.

use crate::role::Role;

/// Why a theme could not be built, or failed its audit.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ThemeError {
    /// The DTCG source was not valid JSON or not a token tree.
    #[error("invalid DTCG source: {0}")]
    InvalidSource(String),
    /// A token path that is not in the role vocabulary. Authored colours may
    /// only enter through roles (REQ-THM-002).
    #[error("unknown token `{path}` — authored colours enter through roles only")]
    UnknownRole {
        /// The offending path.
        path: String,
    },
    /// A role was never given a value; the gate cannot run on a hole.
    #[error("role `{0}` is unset")]
    UnsetRole(Role),
    /// A token's `$value` was not a hex string the parser accepts.
    #[error("token `{path}` has an unparseable colour value")]
    UnparseableValue {
        /// The offending path.
        path: String,
    },
    /// REQ-THM-003: a semantic pair below its WCAG gate. The error *is* the
    /// build failure — there is no warning mode.
    #[error("contrast gate: `{fg}` on `{bg}` is {ratio:.2}:1, needs {required}:1")]
    ContrastViolation {
        /// Foreground role.
        fg: Role,
        /// Background role.
        bg: Role,
        /// Measured WCAG 2.1 ratio.
        ratio: f64,
        /// The required ratio.
        required: f64,
    },
    /// REQ-THM-005: after CVD simulation the pair's perceptual separation
    /// fell under the floor — the collision the dichromat test exists to
    /// catch.
    #[error("CVD collision: `{fg}` on `{bg}` separates by only {lc:.1} Lc under {kind:?}")]
    CvdCollision {
        /// Foreground role.
        fg: Role,
        /// Background role.
        bg: Role,
        /// Lightness contrast after simulation.
        lc: f64,
        /// Which dichromacy was simulated.
        kind: color_kit::CvdKind,
    },
    /// A colour operation upstream failed (non-finite, etc.).
    #[error("colour operation failed: {0}")]
    Color(#[from] color_kit::ColorError),
    /// An emitter failed (serialization, unsupported target).
    #[error("emitter failed: {0}")]
    Emit(String),
}
