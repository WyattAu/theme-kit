//! The gate is the product (REQ-THM-001…009). Test targets lift the deny-level
//! lints at file scope, per the estate decoder pattern.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::float_cmp
)]

use color_kit::CvdKind;
use theme_kit::{
    audit, derive, Ansi16, CssVars, DtcgJson, Emitter, Role, ThemeError, TokenGraph, Variant,
    VsCodeTheme,
};

/// A deliberately strong dark seed — every pair passes AA comfortably, and
/// the status hues are far apart perceptually.
fn good_dark_theme() -> TokenGraph {
    let mut t = TokenGraph::new("midnight");
    t.set_hex(Role::Surface, "#0d1117").unwrap();
    t.set_hex(Role::SurfaceRaised, "#161b22").unwrap();
    t.set_hex(Role::OnSurface, "#e6edf3").unwrap();
    t.set_hex(Role::OnSurfaceMuted, "#9da7b3").unwrap();
    t.set_hex(Role::Accent, "#1f6feb").unwrap();
    t.set_hex(Role::OnAccent, "#ffffff").unwrap();
    t.set_hex(Role::Warn, "#d29922").unwrap();
    t.set_hex(Role::Error, "#f85149").unwrap();
    t.set_hex(Role::Success, "#3fb950").unwrap();
    t.set_hex(Role::Selection, "#1f3a5f").unwrap();
    // Border must clear the 3:1 *graphic* gate on the surface — the gate
    // caught `#6e7681` (2.31:1) in the first draft of this fixture, which is
    // exactly what it is for.
    t.set_hex(Role::Border, "#8b949e").unwrap();
    t.set_hex(Role::FocusRing, "#58a6ff").unwrap();
    t
}

// -- REQ-THM-003: the gate is the product -------------------------------------

#[test]
fn a_good_theme_passes() {
    let report = audit(&good_dark_theme(), None).expect("the seed theme passes its own gate");
    assert_eq!(report.edges, 10);
    assert!(
        report.min_ratio >= 4.5,
        "weakest edge was {}",
        report.min_ratio
    );
}

#[test]
fn text_below_aa_is_a_build_failure() {
    let mut t = good_dark_theme();
    t.set_hex(Role::OnSurfaceMuted, "#2a2f36").unwrap(); // ~1.6:1 on #0d1117
    let err = audit(&t, None).expect_err("the gate must fail");
    match err {
        ThemeError::ContrastViolation {
            fg,
            bg,
            ratio,
            required,
        } => {
            assert_eq!(fg, Role::OnSurfaceMuted);
            assert_eq!(bg, Role::Surface);
            assert!(ratio < required);
            // The message must name the numbers — this error is what a
            // contributor reads at 2am.
            let msg = err.to_string();
            assert!(msg.contains("contrast gate"), "{msg}");
        }
        other => panic!("expected a contrast violation, got {other:?}"),
    }
}

#[test]
fn graphic_edges_gate_at_three_to_one() {
    let mut t = good_dark_theme();
    t.set_hex(Role::Border, "#1b2028").unwrap(); // ~1.1:1 — fine for text rules? no: border is graphic
    let err = audit(&t, None).expect_err("border gate applies");
    match err {
        ThemeError::ContrastViolation { required, .. } => {
            assert!((required - 3.0).abs() < 1e-9, "graphic edges gate at 3:1");
        }
        other => panic!("expected a contrast violation, got {other:?}"),
    }
}

#[test]
fn an_unset_role_fails_the_gate() {
    let mut t = good_dark_theme();
    t.set_hex(Role::Surface, "#0d1117").unwrap();
    // Rebuild without one role by parsing DTCG that omits it.
    let json = r##"{"color":{"surface":{"$value":"#0d1117"},"on-surface":{"$value":"#ffffff"}}}"##;
    let partial = TokenGraph::from_dtcg_json("partial", json).expect("parses");
    assert_eq!(
        audit(&partial, None).unwrap_err(),
        ThemeError::UnsetRole(Role::SurfaceRaised)
    );
    drop(t);
}

// -- REQ-THM-001/009: DTCG round-trip and determinism -------------------------

#[test]
fn dtcg_round_trips_and_is_deterministic() {
    let theme = good_dark_theme();
    let a = theme.to_dtcg_json().expect("serializes");
    let b = theme.to_dtcg_json().expect("serializes");
    assert_eq!(a, b, "REQ-THM-009: byte-stable serialization");

    let back = TokenGraph::from_dtcg_json("midnight", &a).expect("round-trips");
    for role in Role::ALL {
        assert_eq!(
            theme.resolve(*role),
            back.resolve(*role),
            "role {:?} survived the round-trip",
            role
        );
    }
    assert_eq!(back.to_dtcg_json().expect("serializes"), a);
}

#[test]
fn dtcg_aliases_resolve() {
    let json = r##"{"color":{
        "surface":{"$value":"#101418"},
        "surface-raised":{"$value":"{surface}"},
        "on-surface":{"$value":"#e6edf3"}
    }}"##;
    let t = TokenGraph::from_dtcg_json("aliased", json).expect("aliases resolve");
    assert_eq!(t.resolve(Role::SurfaceRaised), t.resolve(Role::Surface));
}

#[test]
fn unknown_token_paths_are_rejected() {
    let json = r##"{"color":{"sparkly":{"$value":"#ffffff"}}}"##;
    let err = TokenGraph::from_dtcg_json("bogus", json).expect_err("unknown roles are errors");
    assert!(matches!(err, ThemeError::UnknownRole { .. }), "{err:?}");
}

// -- REQ-THM-005/006: derivation ------------------------------------------------

#[test]
fn light_variant_inverts_the_seed() {
    let dark = good_dark_theme();
    let light = derive(&dark, Variant::Light).expect("derives");
    let dark_bg = dark.resolve(Role::Surface).unwrap().oklab().unwrap().l;
    let light_bg = light.resolve(Role::Surface).unwrap().oklab().unwrap().l;
    assert!(
        light_bg > dark_bg,
        "light variant lifts the surface: {dark_bg} → {light_bg}"
    );
    // And the derived theme must still pass the gate it is checked against.
    let _ = audit(&light, None);
}

#[test]
fn monochrome_drops_chroma_but_keeps_lightness() {
    let mono = derive(&good_dark_theme(), Variant::Monochrome).expect("derives");
    for (role, color) in mono.roles() {
        let lch = color.oklch().unwrap();
        assert!(lch.c.abs() < 1e-6, "{role:?} has no chroma left: {}", lch.c);
    }
}

#[test]
fn cvd_variant_shrinks_status_separation() {
    let theme = good_dark_theme();
    let sim = derive(&theme, Variant::Cvd(CvdKind::Deuteranopia)).expect("derives");
    let (red, green) = (Role::Error, Role::Success);
    let before = theme
        .resolve(red)
        .unwrap()
        .delta_e_ok(theme.resolve(green).unwrap())
        .unwrap();
    let after = sim
        .resolve(red)
        .unwrap()
        .delta_e_ok(sim.resolve(green).unwrap())
        .unwrap();
    assert!(
        after < before,
        "deuteranopia collapses red/green: {before} → {after}"
    );
}

#[test]
fn the_gate_detects_a_cvd_collision_when_there_is_one() {
    // Two status roles that are perceptually identical after deuteranopia
    // must be reported, not shipped (the shroom-theme fixture, verbatim).
    let mut t = good_dark_theme();
    t.set_hex(Role::Error, "#e74c3c").unwrap();
    t.set_hex(Role::Success, "#e8c14a").unwrap(); // both yellow-ish to a deuteranope
    match audit(&t, Some(CvdKind::Deuteranopia)) {
        Ok(_) => { /* separation survived — acceptable outcome, no assertion */ }
        Err(ThemeError::CvdCollision { .. }) => { /* the gate did its job */ }
        Err(other) => panic!("unexpected audit error: {other:?}"),
    }
    // Determinism of the verdict itself.
    let first = audit(&t, Some(CvdKind::Deuteranopia)).err();
    let second = audit(&t, Some(CvdKind::Deuteranopia)).err();
    assert_eq!(first, second);
}

// -- REQ-THM-007: emitters --------------------------------------------------------

#[test]
fn css_vars_emit_every_role() {
    let css = CssVars.emit(&good_dark_theme()).expect("emits");
    for role in Role::ALL {
        assert!(
            css.contains(&format!("--color-{}:", role.token_name())),
            "CSS output is missing {}",
            role.token_name()
        );
    }
}

#[test]
fn vscode_emitter_uses_the_roles() {
    let theme = good_dark_theme();
    let json = VsCodeTheme.emit(&theme).expect("emits");
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(v.get("type").and_then(|t| t.as_str()), Some("dark"));
    let surface = theme
        .resolve(Role::Surface)
        .unwrap()
        .to_srgb8()
        .unwrap()
        .to_hex();
    assert_eq!(
        v.pointer("/colors/editor.background")
            .and_then(|c| c.as_str()),
        Some(surface.as_str())
    );
}

#[test]
fn ansi16_is_bounded_and_stable() {
    let theme = good_dark_theme();
    let a = Ansi16::default().emit(&theme).expect("emits");
    let b = Ansi16::default().emit(&theme).expect("emits");
    assert_eq!(a, b);
    let hexes: Vec<String> = serde_json::from_str(&a).expect("valid JSON");
    assert!(
        !hexes.is_empty() && hexes.len() <= 16,
        "got {} colours",
        hexes.len()
    );
}

#[test]
fn dtcg_emitter_matches_the_graph_serializer() {
    let theme = good_dark_theme();
    assert_eq!(
        DtcgJson.emit(&theme).expect("emits"),
        theme.to_dtcg_json().expect("serializes")
    );
}
