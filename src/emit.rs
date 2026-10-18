//! Data-only emitters (REQ-THM-007). Emitters turn a resolved graph into
//! *text or data* — never a toolkit type. Toolkit lowering lives in
//! one-file adapter crates (THM-008) so this crate keeps no UI dependency.

use crate::error::ThemeError;
use crate::graph::TokenGraph;
use crate::role::Role;
use color_kit::quantize;

/// A named serialization target for a resolved theme.
pub trait Emitter {
    /// Serialize `graph` for this target.
    fn emit(&self, graph: &TokenGraph) -> Result<String, ThemeError>;
}

/// `:root { --color-<role>: #hex; … }` — the web/islands path
/// (`hosting-dashboard`, the leptos-* crates, pediment consumers).
#[derive(Debug, Clone, Copy, Default)]
pub struct CssVars;

impl Emitter for CssVars {
    fn emit(&self, graph: &TokenGraph) -> Result<String, ThemeError> {
        let mut out = String::from(":root {\n");
        for (role, color) in graph.roles() {
            let view = Role::as_view(color).ok_or(ThemeError::UnparseableValue {
                path: role.token_name().to_owned(),
            })?;
            out.push_str(&format!("  --color-{}: {};\n", role.token_name(), view.to_hex()));
        }
        out.push_str("}\n");
        Ok(out)
    }
}

/// DTCG 2025.10 JSON — the canonical interchange (round-trips back through
/// [`TokenGraph::from_dtcg_json`]).
#[derive(Debug, Clone, Copy, Default)]
pub struct DtcgJson;

impl Emitter for DtcgJson {
    fn emit(&self, graph: &TokenGraph) -> Result<String, ThemeError> {
        graph.to_dtcg_json()
    }
}

/// VS Code / TextMate theme — the shroom-theme pipeline's Rust emitter. The
/// full 1049-slot mapping is emitter work, not crate work; v0.1 carries the
/// load-bearing slots the editor chrome actually reads.
#[derive(Debug, Clone, Copy, Default)]
pub struct VsCodeTheme;

impl Emitter for VsCodeTheme {
    fn emit(&self, graph: &TokenGraph) -> Result<String, ThemeError> {
        use serde_json::{Map, Value};
        let hex = |role: Role| -> Result<String, ThemeError> {
            let c = graph.require(role)?;
            let view = Role::as_view(c).ok_or(ThemeError::UnparseableValue {
                path: role.token_name().to_owned(),
            })?;
            Ok(view.to_hex())
        };
        let mut colors = Map::new();
        colors.insert("editor.background".to_owned(), Value::String(hex(Role::Surface)?));
        colors.insert("editor.foreground".to_owned(), Value::String(hex(Role::OnSurface)?));
        colors.insert(
            "editorLineNumber.foreground".to_owned(),
            Value::String(hex(Role::OnSurfaceMuted)?),
        );
        colors.insert("editor.selectionBackground".to_owned(), Value::String(hex(Role::Selection)?));
        colors.insert("focusBorder".to_owned(), Value::String(hex(Role::FocusRing)?));
        colors.insert("editorIndentGuide.background".to_owned(), Value::String(hex(Role::Border)?));
        colors.insert("errorForeground".to_owned(), Value::String(hex(Role::Error)?));
        colors.insert("editorWarning.foreground".to_owned(), Value::String(hex(Role::Warn)?));

        let surface = graph.require(Role::Surface)?;
        let surface_l = surface.oklab()?.l;
        let mut root = Map::new();
        root.insert("name".to_owned(), Value::String(graph.name().to_owned()));
        root.insert(
            "type".to_owned(),
            Value::String(if surface_l < 0.5 { "dark" } else { "light" }.to_owned()),
        );
        root.insert("colors".to_owned(), Value::Object(colors));
        root.insert("tokenColors".to_owned(), Value::Array(vec![]));
        serde_json::to_string_pretty(&Value::Object(root))
            .map_err(|e| ThemeError::Emit(e.to_string()))
    }
}

/// A deterministic ANSI-16 palette: quantise the theme's roles to sixteen
/// colours (fixed seed — terminals get stable colours across builds).
#[derive(Debug, Clone, Copy)]
pub struct Ansi16 {
    /// Seed for the quantiser; part of the emitted bytes.
    pub seed: u64,
}

impl Default for Ansi16 {
    fn default() -> Self {
        Self { seed: 0xA11C_E5ED }
    }
}

impl Emitter for Ansi16 {
    fn emit(&self, graph: &TokenGraph) -> Result<String, ThemeError> {
        let palette: Vec<color_kit::Rgba> = graph.roles().map(|(_, c)| c).collect();
        let colors = quantize(&palette, 16, self.seed);
        let hexes: Vec<String> = colors
            .iter()
            .filter_map(|c| Role::as_view(*c).map(|v| v.to_hex()))
            .collect();
        serde_json::to_string_pretty(&hexes).map_err(|e| ThemeError::Emit(e.to_string()))
    }
}
