# theme-kit

The typed design-token contract for the WyattAu estate (spec:
`engineering-standards/specs/theme-kit.md`, ADR-0004 §2). One graph, one gate,
data-only emitters.

```rust
use theme_kit::{audit, Role, TokenGraph};

let mut theme = TokenGraph::new("midnight");
theme.set_hex(Role::Surface, "#0d1117").unwrap();
theme.set_hex(Role::OnSurface, "#e6edf3").unwrap();
// …the rest of the 12-role vocabulary…
// A theme that fails the gate does not warn; audit() is the build failure:
let report = audit(&theme, None);
```

| REQ | Where |
|---|---|
| THM-001 DTCG 2025.10 round-trip (byte-stable) | `TokenGraph::from_dtcg_json` / `to_dtcg_json` |
| THM-002 closed role vocabulary | `Role`; unknown token paths are typed errors |
| THM-003 **build-failing** WCAG gate | `audit()` → `ThemeError::ContrastViolation` (names the pair + measured ratio) |
| THM-004/005 lightness ranks + CVD collisions | `audit(graph, Some(CvdKind::Deuteranopia))` |
| THM-006 variant derivation | `derive(&graph, Variant::{Light, HighContrast, Monochrome, Cvd(kind)})`: the seven shroom variants are derivation modes over one seed |
| THM-007 emitters | `CssVars`, `DtcgJson`, `VsCodeTheme`, `Ansi16`, or your own `Emitter` |

### Derivation is not gate-safe: re-gate after `derive`

`derive` does **not** promise that its output passes `audit`. Measured
2026-10-18 in `estate-integration`'s `ui_semantics` suite: a palette that
passes as authored fails after `derive(HighContrast)`, with `on-accent` on
`accent` at **2.88:1** against the 4.5 floor. The shipped seed palette
survives the same derivation, so the property is palette-dependent rather
than uniformly broken, which is exactly why it went unnoticed.

A caller must therefore treat `derive` as a *proposal* and re-run the gate on
its output:

```rust
let variant = theme_kit::derive(&graph, Variant::HighContrast)?;
theme_kit::audit(&variant, None)?; // required: derive makes no promise
```

Shipping a derived variant without that second `audit` is how an
inaccessible theme reaches users while the build stays green.

## Properties

- **L1 substrate**: the only cross-L2 coupling in the UI family; estate dep
  `color-kit` (L0) alone. No toolkit dependency in the core: egui, ratatui,
  Slint, and Leptos lower through separate one-file adapter crates.
- **Total**: theme files are user input; malformed JSON, unknown roles,
  unset roles, and gate failures are typed errors.
- **Deterministic**: fixed role order and hex formatting; serialization is
  byte-stable across runs (REQ-THM-009, the attestation surface).

Donor: `shroom-theme` (the colour science) and `pediment` (the token schema).
Licensed under MIT OR Apache-2.0.
