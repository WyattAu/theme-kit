//! The gate (REQ-THM-003/004/005). This is the crate's reason to exist: a
//! theme that fails does not warn, it fails — with the offending pair, the
//! measured numbers, and nothing swallowed.

use crate::error::ThemeError;
use crate::graph::TokenGraph;
use crate::role::Role;
use color_kit::{contrast_ratio, lightness_contrast, simulate_cvd, CvdKind, Rgba};

/// Perceptual separation floor (Lc) a pair must keep after CVD simulation.
pub const LC_FLOOR: f64 = 15.0;

/// Oklab distance floor between status roles after CVD simulation — the
/// shroom lesson: red/green semantic pairs collide for a dichromat long
/// before they collide for a trichromat.
pub const ROLE_DELTA_FLOOR: f64 = 0.02;

/// What an audit measured.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Audit {
    /// Semantic edges walked.
    pub edges: usize,
    /// Weakest WCAG ratio observed.
    pub min_ratio: f64,
    /// Weakest post-CVD Lc observed (when a CVD audit ran).
    pub min_lc: f64,
    /// Weakest post-CVD Oklab separation between status roles.
    pub min_role_delta: f64,
}

/// Audit a theme. `cvd` runs the dichromat pass on top of the WCAG pass.
pub fn audit(graph: &TokenGraph, cvd: Option<CvdKind>) -> Result<Audit, ThemeError> {
    let mut min_ratio = f64::INFINITY;
    let mut edges = 0usize;

    for (fg_role, bg_role, normal_text) in Role::semantic_pairs() {
        let fg = graph.require(*fg_role)?;
        let bg = graph.require(*bg_role)?;
        let required = if *normal_text {
            color_kit::WCAG_AA_NORMAL
        } else {
            color_kit::WCAG_AA_LARGE
        };
        let ratio = contrast_ratio(fg, bg).ok_or(color_kit::ColorError::NonFinite {
            component: "contrast",
        })?;
        if ratio < required {
            return Err(ThemeError::ContrastViolation {
                fg: *fg_role,
                bg: *bg_role,
                ratio,
                required,
            });
        }
        if ratio < min_ratio {
            min_ratio = ratio;
        }
        edges += 1;
    }

    let mut min_lc = f64::INFINITY;
    let mut min_delta = f64::INFINITY;
    if let Some(kind) = cvd {
        // (1) Every text pair keeps its perceptual separation under simulation.
        for (fg_role, bg_role, _) in Role::semantic_pairs() {
            let fg = simulate_cvd(graph.require(*fg_role)?, kind);
            let bg = simulate_cvd(graph.require(*bg_role)?, kind);
            let lc = lightness_contrast(fg, bg).ok_or(color_kit::ColorError::NonFinite {
                component: "lightness",
            })?;
            if lc.abs() < LC_FLOOR {
                return Err(ThemeError::CvdCollision {
                    fg: *fg_role,
                    bg: *bg_role,
                    lc,
                    kind,
                });
            }
            if lc.abs() < min_lc {
                min_lc = lc.abs();
            }
        }
        // (2) Status roles stay mutually distinguishable — the collision the
        // dichromat test exists to catch.
        let statuses = [Role::Accent, Role::Warn, Role::Error, Role::Success];
        for (i, a) in statuses.iter().enumerate() {
            for b in statuses.iter().skip(i + 1) {
                let ca = simulate_cvd(graph.require(*a)?, kind);
                let cb = simulate_cvd(graph.require(*b)?, kind);
                let d = oklab_distance(ca, cb)?;
                if d < ROLE_DELTA_FLOOR {
                    return Err(ThemeError::CvdCollision {
                        fg: *a,
                        bg: *b,
                        lc: d * 100.0,
                        kind,
                    });
                }
                if d < min_delta {
                    min_delta = d;
                }
            }
        }
    }

    Ok(Audit {
        edges,
        min_ratio,
        min_lc: if min_lc.is_finite() { min_lc } else { 0.0 },
        min_role_delta: if min_delta.is_finite() {
            min_delta
        } else {
            0.0
        },
    })
}

fn oklab_distance(a: Rgba, b: Rgba) -> Result<f64, ThemeError> {
    Ok(a.delta_e_ok(b)?)
}
