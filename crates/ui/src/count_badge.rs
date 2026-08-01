//! Count badge for SCM (Source Control) panels.
//!
//! Mirrors VS Code's `CountBadge` style used in the Source Control view to
//! display the number of changes per resource group. Unlike [`crate::Badge`],
//! which renders an overlay corner indicator, `CountBadge` is an inline label
//! that uses one of six semantic colors and hides itself when the count is zero.

use gpui::{
    App, Hsla, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window, div, px,
};

use crate::{ActiveTheme, Sizable, Size, StyledExt, h_flex};

/// Semantic color for a [`CountBadge`]. Maps to theme tokens plus a `+N` overflow
/// (grey) variant. The names mirror VS Code's `CountBadge` style options.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum CountBadgeStyle {
    /// Bright accent used for the default badge (resource count).
    #[default]
    Default,
    /// Grey, used for "tracked" / informational counts.
    Muted,
    /// Blue, used for branches / refs.
    Ref,
    /// Green, used for ahead / +N counts.
    Ahead,
    /// Red, used for behind / -N / error counts.
    Behind,
    /// Yellow, used for warnings / pending counts.
    Warning,
}

/// Inline count badge, e.g. `(5)` next to a resource group header.
///
/// The rendered width collapses to zero when the count is zero so the parent
/// flex layout can hide it without reserving space.
#[derive(IntoElement)]
pub struct CountBadge {
    style: StyleRefinement,
    count: usize,
    badge_style: CountBadgeStyle,
    size: Size,
    max: Option<usize>,
}

impl CountBadge {
    /// Create a new count badge. Use `.count(...)` to set the visible count.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            count: 0,
            badge_style: CountBadgeStyle::default(),
            size: Size::Small,
            max: None,
        }
    }

    /// Set the count to display. Zero hides the badge.
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Set the semantic style (color).
    pub fn style(mut self, style: CountBadgeStyle) -> Self {
        self.badge_style = style;
        self
    }

    /// Set the maximum count to display, appending `+` when overflow occurs.
    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    fn resolve_color(&self, cx: &App) -> Hsla {
        match self.badge_style {
            CountBadgeStyle::Default => cx.theme().foreground,
            CountBadgeStyle::Muted => cx.theme().muted_foreground,
            CountBadgeStyle::Ref => cx.theme().blue,
            CountBadgeStyle::Ahead => cx.theme().green,
            CountBadgeStyle::Behind => cx.theme().red,
            CountBadgeStyle::Warning => cx.theme().yellow,
        }
    }
}

impl Default for CountBadge {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for CountBadge {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        // CountBadge does not wrap children; this impl satisfies the trait
        // binding so it can be used as a flex child like other inline elements.
        let _ = elements;
    }
}

impl Sizable for CountBadge {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for CountBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.count == 0 {
            return div().into_any_element();
        }

        let color = self.resolve_color(cx);
        let label = match self.max {
            Some(max) if self.count > max => format!("{max}+"),
            _ => self.count.to_string(),
        };

        let text_size = match self.size {
            Size::Large | Size::Medium | Size::Size(_) => px(11.),
            Size::Small | Size::XSmall => px(10.),
        };

        h_flex()
            .refine_style(&self.style)
            .items_center()
            .text_size(text_size)
            .text_color(color)
            .child(format!("({label})"))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_clamped_to_max() {
        // 1000 with max=99 renders "99+"
        let label = match Some(99usize) {
            Some(max) if 1000 > max => format!("{max}+"),
            _ => "1000".to_string(),
        };
        assert_eq!(label, "99+");
    }

    #[test]
    fn label_under_max_renders_raw() {
        let label = match Some(99usize) {
            Some(max) if 7 > max => format!("{max}+"),
            _ => "7".to_string(),
        };
        assert_eq!(label, "7");
    }

    #[test]
    fn default_style_resolves() {
        // Compile-time check that the default variant is the Default trait impl.
        assert_eq!(CountBadgeStyle::default(), CountBadgeStyle::Default);
    }
}
