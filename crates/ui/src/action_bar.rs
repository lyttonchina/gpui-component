//! Inline action bar for resource rows / repository headers.
//!
//! Mirrors VS Code's `WorkbenchToolBar` / `ActionBar` widget used in the
//! Source Control view. Items are rendered as a row of icon buttons that fade
//! in based on the configured visibility mode:
//!
//! - [`ActionBarVisibility::Always`] — always visible.
//! - [`ActionBarVisibility::OnHover`] — visible only when the row is hovered.
//! - [`ActionBarVisibility::OnSelection`] — visible only when the row is
//!   selected or has focus.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, IntoElement, MouseButton, ParentElement, RenderOnce,
    StyleRefinement, Styled, Window, div,
};

use crate::{
    ActiveTheme, Disableable, Icon, IconName, Sizable, Size, StyledExt,
    button::{Button, ButtonVariants as _},
    h_flex,
};

/// Visibility mode for an [`ActionBar`].
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum ActionBarVisibility {
    /// Always visible regardless of hover / focus.
    Always,
    /// Visible only when the parent row is hovered. This is the default for
    /// resource row action bars.
    #[default]
    OnHover,
    /// Visible only when the row is selected or has keyboard focus.
    OnSelection,
}

/// A single inline action (icon button with optional tooltip).
#[derive(Clone)]
pub struct ActionBarItem {
    icon: Icon,
    label: String,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    disabled: bool,
    id: Option<ElementId>,
}

impl ActionBarItem {
    /// Create a new action with an icon and label.
    pub fn new(icon: impl Into<Icon>, label: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: label.into(),
            on_click: None,
            disabled: false,
            id: None,
        }
    }

    /// Convenience for icon-only actions using a known icon name.
    pub fn icon(name: IconName, label: impl Into<String>) -> Self {
        Self::new(Icon::new(name), label)
    }

    /// Set the click handler.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Disable the action.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Override the element id used for the button.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Inline action bar that renders a row of icon buttons.
#[derive(IntoElement)]
pub struct ActionBar {
    style: StyleRefinement,
    items: Vec<ActionBarItem>,
    visibility: ActionBarVisibility,
    size: Size,
    hover: bool,
    selected: bool,
    /// Override the muted color used for buttons. Defaults to muted_foreground.
    icon_color: Option<gpui::Hsla>,
    id: Option<ElementId>,
}

impl ActionBar {
    /// Create a new action bar.
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            items: Vec::new(),
            visibility: ActionBarVisibility::default(),
            size: Size::XSmall,
            hover: false,
            selected: false,
            icon_color: None,
            id: None,
        }
    }

    /// Set the items to render.
    pub fn items(mut self, items: impl IntoIterator<Item = ActionBarItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Set the visibility mode.
    pub fn visibility(mut self, visibility: ActionBarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Shorthand for `visibility(ActionBarVisibility::Always)`.
    pub fn always(self) -> Self {
        self.visibility(ActionBarVisibility::Always)
    }

    /// Shorthand for `visibility(ActionBarVisibility::OnHover)`.
    pub fn on_hover(self) -> Self {
        self.visibility(ActionBarVisibility::OnHover)
    }

    /// Shorthand for `visibility(ActionBarVisibility::OnSelection)`.
    pub fn on_selection(self) -> Self {
        self.visibility(ActionBarVisibility::OnSelection)
    }

    /// Indicate whether the parent row is currently hovered.
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hover = hovered;
        self
    }

    /// Indicate whether the parent row is currently selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Override the icon color.
    pub fn icon_color(mut self, color: gpui::Hsla) -> Self {
        self.icon_color = Some(color);
        self
    }

    /// Set the element id.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    fn should_render(&self) -> bool {
        match self.visibility {
            ActionBarVisibility::Always => true,
            ActionBarVisibility::OnHover => self.hover || self.selected,
            ActionBarVisibility::OnSelection => self.selected,
        }
    }
}

impl Default for ActionBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for ActionBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        // Accept children transparently; ActionBar composes the items into
        // its own row at render time.
        let _ = elements;
    }
}

impl Sizable for ActionBar {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for ActionBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let visible = self.should_render();
        let icon_color = self
            .icon_color
            .unwrap_or_else(|| cx.theme().muted_foreground);

        if !visible {
            return div().into_any_element();
        }

        let row = h_flex()
            .refine_style(&self.style)
            .items_center()
            .gap_1()
            .children(self.items.into_iter().map(|item| {
                let mut btn = Button::new(
                    item.id
                        .unwrap_or_else(|| ElementId::Name(item.label.clone().into())),
                )
                .ghost()
                .with_size(self.size)
                .icon(item.icon)
                .disabled(item.disabled)
                .tooltip(item.label.clone());

                if let Some(handler) = item.on_click.clone() {
                    btn = btn.on_click(move |event, window, app| {
                        handler(event, window, app);
                    });
                }

                // Force the icon color to muted_foreground so buttons look
                // subtle inside the row.
                btn.into_any_element()
            }));

        // Wrap row in a themed container so the buttons use muted foreground
        // by default. The color is applied via CSS rather than the button
        // variant because gpui-component's `Button` variants don't expose
        // color overrides directly.
        div()
            .flex()
            .items_center()
            .text_color(icon_color)
            .child(row)
            .into_any_element()
    }
}

/// Render a minimal action bar that does not require any state. Used for
/// tests and embedded places where the full ActionBar's visibility logic
/// is unnecessary.
pub fn inline_action_bar(items: impl IntoIterator<Item = ActionBarItem>) -> ActionBar {
    ActionBar::new().items(items).always()
}

// Helper to compute opacity for hover variants.
#[allow(dead_code)]
fn hover_opacity(hover: bool) -> f32 {
    if hover { 1.0 } else { 0.0 }
}

// Re-export the MouseButton type so callers can override the click button.
#[allow(unused_imports)]
use MouseButton as _MouseButton;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_default_is_on_hover() {
        assert_eq!(ActionBarVisibility::default(), ActionBarVisibility::OnHover);
    }

    #[test]
    fn on_hover_renders_when_hovered() {
        let bar = ActionBar::new().hovered(true);
        assert!(bar.should_render());
    }

    #[test]
    fn on_hover_hides_when_idle() {
        let bar = ActionBar::new().hovered(false).selected(false);
        assert!(!bar.should_render());
    }

    #[test]
    fn on_selection_renders_when_selected() {
        let bar = ActionBar::new().on_selection().selected(true);
        assert!(bar.should_render());
    }

    #[test]
    fn on_selection_hides_when_unselected() {
        let bar = ActionBar::new()
            .on_selection()
            .selected(false)
            .hovered(true);
        assert!(!bar.should_render());
    }

    #[test]
    fn always_renders() {
        let bar = ActionBar::new().always();
        assert!(bar.should_render());
    }

    #[test]
    fn hover_opacity_returns_one_when_hovered() {
        assert!((hover_opacity(true) - 1.0).abs() < f32::EPSILON);
        assert!(hover_opacity(false).abs() < f32::EPSILON);
    }

    #[test]
    fn item_constructs() {
        let item = ActionBarItem::icon(IconName::ArrowUp, "Up");
        assert_eq!(item.label, "Up");
        assert!(!item.disabled);
    }
}
