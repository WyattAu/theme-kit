//! The token graph and its DTCG 2025.10 bridge (REQ-THM-001/002/009).

use crate::error::ThemeError;
use crate::role::Role;
use color_kit::{parse_hex, Rgba};
use serde_json::{Map, Value};

/// A resolved theme: one colour per [`Role`], deterministic by
/// construction (roles are stored in declaration order).
#[derive(Debug, Clone, PartialEq)]
pub struct TokenGraph {
    name: String,
    values: Vec<Option<Rgba>>,
}

fn role_index(role: Role) -> usize {
    Role::ALL
        .iter()
        .position(|r| *r == role)
        .unwrap_or(0)
}

impl TokenGraph {
    /// An empty graph with a name (the theme's identity for emitters).
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            values: vec![None; Role::ALL.len()],
        }
    }

    /// The theme name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set a role (chainable).
    pub fn set(&mut self, role: Role, value: Rgba) -> &mut Self {
        let idx = role_index(role);
        if let Some(slot) = self.values.get_mut(idx) {
            *slot = Some(value);
        }
        self
    }

    /// Set a role from sRGB bytes — the door hex literals come through.
    pub fn set_hex(&mut self, role: Role, hex: &str) -> Result<&mut Self, ThemeError> {
        let v = parse_hex(hex).ok_or_else(|| ThemeError::UnparseableValue {
            path: role.token_name().to_owned(),
        })?;
        Ok(self.set(role, v.to_linear()))
    }

    /// Resolve a role, if set.
    #[must_use]
    pub fn resolve(&self, role: Role) -> Option<Rgba> {
        self.values.get(role_index(role)).copied().flatten()
    }

    /// Resolve a role or fail with [`ThemeError::UnsetRole`] — the form the
    /// gate uses, because a hole is a gate failure, not a default.
    pub fn require(&self, role: Role) -> Result<Rgba, ThemeError> {
        self.resolve(role).ok_or(ThemeError::UnsetRole(role))
    }

    /// Every set role, in declaration order.
    pub fn roles(&self) -> impl Iterator<Item = (Role, Rgba)> + '_ {
        Role::ALL
            .iter()
            .copied()
            .filter_map(|r| self.resolve(r).map(|v| (r, v)))
    }

    /// Parse a DTCG 2025.10 token tree. Accepts hex `$value` strings and
    /// `{alias}` references; unknown paths are typed errors (REQ-THM-002) —
    /// authored colours may enter *only* through roles.
    pub fn from_dtcg_json(name: impl Into<String>, src: &str) -> Result<Self, ThemeError> {
        let root: Value =
            serde_json::from_str(src).map_err(|e| ThemeError::InvalidSource(e.to_string()))?;
        let mut raw: Vec<(String, String)> = Vec::new();
        collect_tokens(&root, "", &mut raw);
        // Every leaf must be a role; anything else is an author mistake we
        // refuse to drop silently.
        let mut pending: Vec<(Role, String)> = Vec::with_capacity(raw.len());
        for (path, value) in raw {
            let role =
                Role::from_token_path(&path).ok_or(ThemeError::UnknownRole { path })?;
            pending.push((role, value));
        }
        let mut graph = Self::new(name);
        // Aliases may reference roles declared later; resolve iteratively.
        let mut progress = true;
        while progress && !pending.is_empty() {
            progress = false;
            let mut next = Vec::new();
            for (role, value) in pending {
                if value.starts_with('{') && value.ends_with('}') {
                    let inner = value
                        .trim_start_matches('{')
                        .trim_end_matches('}')
                        .to_owned();
                    match Role::from_token_path(&inner) {
                        Some(target) => match graph.resolve(target) {
                            Some(resolved) => {
                                graph.set(role, resolved);
                                progress = true;
                            }
                            None => next.push((role, value)),
                        },
                        None => {
                            return Err(ThemeError::UnknownRole { path: inner });
                        }
                    }
                } else {
                    let v = parse_hex(&value).ok_or_else(|| ThemeError::UnparseableValue {
                        path: role.token_name().to_owned(),
                    })?;
                    graph.set(role, v.to_linear());
                    progress = true;
                }
            }
            pending = next;
        }
        if let Some((role, _)) = pending.first() {
            return Err(ThemeError::UnsetRole(*role));
        }
        Ok(graph)
    }

    /// Serialize to DTCG 2025.10. Deterministic: fixed role order, fixed hex
    /// formatting — the attestation surface (REQ-THM-009).
    pub fn to_dtcg_json(&self) -> Result<String, ThemeError> {
        let mut color = Map::new();
        for role in Role::ALL {
            let Some(value) = self.resolve(*role) else {
                continue;
            };
            let view = Role::as_view(value).ok_or(ThemeError::UnparseableValue {
                path: role.token_name().to_owned(),
            })?;
            let mut token = Map::new();
            token.insert(
                "$value".to_owned(),
                Value::String(view.to_hex()),
            );
            token.insert("$type".to_owned(), Value::String("color".to_owned()));
            color.insert(role.token_name().to_owned(), Value::Object(token));
        }
        let mut root = Map::new();
        root.insert(
            "$schema".to_owned(),
            Value::String("https://tr.designtokens.org/format/".to_owned()),
        );
        root.insert(
            "$description".to_owned(),
            Value::String(self.name.clone()),
        );
        root.insert("color".to_owned(), Value::Object(color));
        serde_json::to_string_pretty(&Value::Object(root))
            .map_err(|e| ThemeError::Emit(e.to_string()))
    }
}

/// Walk a DTCG tree collecting `path → raw $value` pairs. Unknown leaves
/// are *not* filtered here — `from_dtcg_json` rejects them (REQ-THM-002).
fn collect_tokens(value: &Value, prefix: &str, out: &mut Vec<(String, String)>) {
    let Some(obj) = value.as_object() else {
        return;
    };
    if let Some(Value::String(v)) = obj.get("$value") {
        let path = if prefix.is_empty() {
            "$root".to_owned()
        } else {
            prefix.to_owned()
        };
        out.push((path, v.clone()));
        return;
    }
    for (key, child) in obj {
        if key.starts_with('$') {
            continue;
        }
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        collect_tokens(child, &path, out);
    }
}
