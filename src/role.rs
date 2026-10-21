//! The closed role vocabulary (specs/theme-kit.md REQ-THM-002).
//!
//! Authored colours exist **only** here. Component and state tokens derive
//! from these roles; nothing else may introduce an ad-hoc colour, which is
//! what keeps the PDF editor, the mail client, the browser overlay, and the
//! TUIs resolving the same design system (ADR-0004 §2).

use color_kit::Rgba8;

/// The semantic role vocabulary. The list is deliberately small and closed:
/// adding a role is an API decision with cross-stack consequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    /// The base page/panel background.
    Surface,
    /// Raised background (cards, popovers, selected rows).
    SurfaceRaised,
    /// Primary text on any surface.
    OnSurface,
    /// Secondary text (captions, help, de-emphasised metadata).
    OnSurfaceMuted,
    /// The accent (selection, primary action, links).
    Accent,
    /// Text/icons drawn on top of [`Role::Accent`].
    OnAccent,
    /// Warning text/icons on a surface.
    Warn,
    /// Error text/icons on a surface.
    Error,
    /// Success text/icons on a surface.
    Success,
    /// Selection background (may equal a raised surface; kept separate so
    /// themes can differ).
    Selection,
    /// Border/divider colour (graphic, not text — 3:1 gate, not 4.5).
    Border,
    /// Focus ring colour (graphic; must be visible against its surface).
    FocusRing,
}

impl core::fmt::Display for Role {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.token_name())
    }
}

impl Role {
    /// Every role, in declaration order (stable — serialization order is a
    /// determinism guarantee, REQ-THM-009).
    pub const ALL: &'static [Role] = &[
        Role::Surface,
        Role::SurfaceRaised,
        Role::OnSurface,
        Role::OnSurfaceMuted,
        Role::Accent,
        Role::OnAccent,
        Role::Warn,
        Role::Error,
        Role::Success,
        Role::Selection,
        Role::Border,
        Role::FocusRing,
    ];

    /// The DTCG token name (also the `color.<name>` path prefix in DTCG
    /// sources).
    #[must_use]
    pub const fn token_name(self) -> &'static str {
        match self {
            Role::Surface => "surface",
            Role::SurfaceRaised => "surface-raised",
            Role::OnSurface => "on-surface",
            Role::OnSurfaceMuted => "on-surface-muted",
            Role::Accent => "accent",
            Role::OnAccent => "on-accent",
            Role::Warn => "warn",
            Role::Error => "error",
            Role::Success => "success",
            Role::Selection => "selection",
            Role::Border => "border",
            Role::FocusRing => "focus-ring",
        }
    }

    /// Look up a role by DTCG token name, tolerating the `color.` prefix.
    #[must_use]
    pub fn from_token_path(path: &str) -> Option<Role> {
        let tail = path.strip_prefix("color.").unwrap_or(path);
        Role::ALL.iter().copied().find(|r| r.token_name() == tail)
    }

    /// True for roles used as a background behind other roles.
    #[must_use]
    pub const fn is_background(self) -> bool {
        matches!(
            self,
            Role::Surface | Role::SurfaceRaised | Role::Accent | Role::Selection
        )
    }

    /// The (foreground, background, gate) edges the audit walks (REQ-THM-003).
    ///
    /// `true` means normal text (4.5:1), `false` means a graphic edge (3:1).
    #[must_use]
    pub const fn semantic_pairs() -> &'static [(Role, Role, bool)] {
        &[
            (Role::OnSurface, Role::Surface, true),
            (Role::OnSurface, Role::SurfaceRaised, true),
            (Role::OnSurfaceMuted, Role::Surface, true),
            (Role::OnAccent, Role::Accent, true),
            (Role::Warn, Role::Surface, true),
            (Role::Error, Role::Surface, true),
            (Role::Success, Role::Surface, true),
            (Role::OnSurface, Role::Selection, true),
            (Role::Border, Role::Surface, false),
            (Role::FocusRing, Role::Surface, false),
        ]
    }

    /// Compact 8-bit view of a resolved colour for emitters.
    #[must_use]
    pub fn as_view(c: color_kit::Rgba) -> Option<Rgba8> {
        c.to_srgb8().ok()
    }
}
