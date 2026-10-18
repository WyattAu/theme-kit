//! Variant derivation (REQ-THM-005/006): the seven shroom variants are
//! *derivation modes* over one seed palette, never hand-authored duplicates.

use crate::error::ThemeError;
use crate::graph::TokenGraph;
use crate::role::Role;
use color_kit::{simulate_cvd, CvdKind, Oklch, Rgba};

/// A derived variant of a seed graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    /// Light polarity (mirrored lightness, slightly calmer chroma).
    Light,
    /// Dark polarity (identity — the seed is expected to be dark).
    Dark,
    /// Maximum separation: foregrounds pushed to the polarity extreme.
    HighContrast,
    /// Channel projection to greyscale, lightness preserved.
    Monochrome,
    /// Dichromat-adapted (Brettel–Viénot simulation of every role).
    Cvd(CvdKind),
}

fn retint(c: Rgba, f: impl Fn(Oklch) -> Oklch) -> Result<Rgba, ThemeError> {
    let lch = c.oklch()?;
    let next = f(lch);
    Ok(Rgba::from_oklch(next)?)
}

/// Derive `variant` from `graph`. Deterministic (REQ-THM-006): the same
/// seed always yields the same variant bytes. The result is *not* assumed
/// to pass the gate — callers run [`crate::audit`] on it, which is how a bad
/// derivation becomes a build failure rather than a surprise in review.
pub fn derive(graph: &TokenGraph, variant: Variant) -> Result<TokenGraph, ThemeError> {
    let mut out = TokenGraph::new(format!("{}/{}", graph.name(), variant_label(variant)));
    let surface_l = graph
        .resolve(Role::Surface)
        .and_then(|c| c.oklab().ok())
        .map(|l| l.l)
        .unwrap_or(0.0);
    let dark_bg = surface_l < 0.5;

    for (role, color) in graph.roles() {
        let next = match variant {
            Variant::Dark => color,
            Variant::Light => retint(color, |lch| Oklch {
                l: 1.0 - lch.l * 0.9,
                c: lch.c * 0.95,
                h: lch.h,
            })?,
            Variant::Monochrome => retint(color, |lch| Oklch {
                l: lch.l,
                c: 0.0,
                h: lch.h,
            })?,
            Variant::HighContrast => {
                if role.is_background() {
                    color
                } else {
                    retint(color, |lch| Oklch {
                        l: if dark_bg { 0.99 } else { 0.01 },
                        c: lch.c,
                        h: lch.h,
                    })?
                }
            }
            Variant::Cvd(kind) => simulate_cvd(color, kind),
        };
        out.set(role, next);
    }
    Ok(out)
}

fn variant_label(v: Variant) -> &'static str {
    match v {
        Variant::Light => "light",
        Variant::Dark => "dark",
        Variant::HighContrast => "high-contrast",
        Variant::Monochrome => "monochrome",
        Variant::Cvd(CvdKind::Protanopia) => "protanopia",
        Variant::Cvd(CvdKind::Deuteranopia) => "deuteranopia",
        Variant::Cvd(CvdKind::Tritanopia) => "tritanopia",
    }
}
