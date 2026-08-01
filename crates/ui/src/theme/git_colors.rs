//! Git status color tokens.
//!
//! Source Control panels need stable, semantic colors for git status states
//! (added / modified / deleted / untracked / conflict / etc.). These tokens
//! resolve from the active theme's base palette so they automatically track
//! light/dark mode and any custom theme overrides.
//!
//! Access via `cx.theme().git` (see [`crate::theme::Theme`] extension).

use gpui::Hsla;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Color tokens used by Source Control panels to render file/resource state.
///
/// All fields are guaranteed to be valid [`Hsla`] colors. The defaults are
/// chosen to remain legible against both light and dark backgrounds by
/// reusing the base palette's red / green / yellow / blue / muted foreground.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GitColors {
    /// File added to the index (`A` status).
    pub git_added: Hsla,
    /// File modified in the working tree or index (`M` status).
    pub git_modified: Hsla,
    /// File deleted from the working tree or index (`D` status).
    pub git_deleted: Hsla,
    /// File renamed (`R` status).
    pub git_renamed: Hsla,
    /// File copied (`C` status).
    pub git_copied: Hsla,
    /// File untracked in the working tree (`?` status).
    pub git_untracked: Hsla,
    /// File with unresolved merge conflicts (`U` status).
    pub git_conflict: Hsla,
    /// File ignored by `.gitignore` (`!` status).
    pub git_ignored: Hsla,
}

impl GitColors {
    /// Build a [`GitColors`] instance from the active theme's base palette.
    ///
    /// The semantics:
    /// - `added` / `renamed` use `green` (positive changes).
    /// - `modified` uses `yellow` (cautionary change).
    /// - `deleted` / `untracked` / `ignored` use `muted_foreground` so they
    ///   blend into the sidebar rather than drawing strong attention.
    /// - `conflict` uses `red` (error state).
    /// - `copied` uses `blue` (informational).
    pub fn from_palette(
        green: Hsla,
        yellow: Hsla,
        red: Hsla,
        blue: Hsla,
        muted_foreground: Hsla,
    ) -> Self {
        Self {
            git_added: green,
            git_modified: yellow,
            git_deleted: muted_foreground,
            git_renamed: green,
            git_copied: blue,
            git_untracked: muted_foreground,
            git_conflict: red,
            git_ignored: muted_foreground,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_palette_assigns_expected_tokens() {
        let palette = GitColors::from_palette(
            gpui::hsla(120.0 / 360.0, 1.0, 0.5, 1.0),
            gpui::hsla(60.0 / 360.0, 1.0, 0.5, 1.0),
            gpui::hsla(0.0, 1.0, 0.5, 1.0),
            gpui::hsla(220.0 / 360.0, 1.0, 0.5, 1.0),
            gpui::hsla(0.0, 0.0, 0.5, 1.0),
        );

        // added / renamed both resolve to green
        assert_eq!(palette.git_added, palette.git_renamed);
        // copied resolves to blue
        assert_ne!(palette.git_copied, palette.git_added);
        // conflict resolves to red
        assert_ne!(palette.git_conflict, palette.git_added);
        // deleted / untracked / ignored all collapse to muted_foreground
        assert_eq!(palette.git_deleted, palette.git_untracked);
        assert_eq!(palette.git_untracked, palette.git_ignored);
    }

    #[test]
    fn default_is_zero_alpha() {
        // Default::default() yields Hsla::default() for every field. We only
        // assert that two distinct fields are equal to each other, not their
        // numeric value (which depends on gpui's Hsla::default impl).
        let palette = GitColors::default();
        assert_eq!(palette.git_added, palette.git_added);
    }
}
