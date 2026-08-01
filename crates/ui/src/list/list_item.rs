use crate::{ActiveTheme, Disableable, Icon, Selectable, Sizable as _, StyledExt, h_flex};
use gpui::{
    AnyElement, App, ClickEvent, DefiniteLength, Div, ElementId, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement, RenderOnce, Stateful,
    StatefulInteractiveElement as _, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use smallvec::SmallVec;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ListItemMode {
    #[default]
    Entry,
    Separator,
}

impl ListItemMode {
    #[inline]
    fn is_separator(&self) -> bool {
        matches!(self, ListItemMode::Separator)
    }
}

#[derive(IntoElement)]
pub struct ListItem {
    base: Stateful<Div>,
    mode: ListItemMode,
    style: StyleRefinement,
    disabled: bool,
    selected: bool,
    secondary_selected: bool,
    confirmed: bool,
    check_icon: Option<Icon>,
    /// Optional row height override used by the `h(...)` builder.
    ///
    /// PR-7b lets panels like the Git Panel source-control renderer
    /// pin a fixed 22 px row to match VS Code's `scm.css` `.scm-row`
    /// density without giving up the default padding-based row.
    height: Option<DefiniteLength>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_mouse_down:
        HashMap<MouseButton, Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>>,
    on_mouse_enter: Option<Box<dyn Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static>>,
    suffix: Option<Box<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>>,
    children: SmallVec<[AnyElement; 2]>,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id: ElementId = id.into();
        Self {
            mode: ListItemMode::Entry,
            base: h_flex().id(id),
            style: StyleRefinement::default(),
            disabled: false,
            selected: false,
            secondary_selected: false,
            confirmed: false,
            on_click: None,
            on_mouse_down: HashMap::new(),
            on_mouse_enter: None,
            check_icon: None,
            height: None,
            suffix: None,
            children: SmallVec::new(),
        }
    }

    /// Set this list item to as a separator, it not able to be selected.
    pub fn separator(mut self) -> Self {
        self.mode = ListItemMode::Separator;
        self
    }

    /// Set to show check icon, default is None.
    pub fn check_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.check_icon = Some(icon.into());
        self
    }

    /// Set ListItem as the selected item style.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set ListItem as the confirmed item style, it will show a check icon.
    pub fn confirmed(mut self, confirmed: bool) -> Self {
        self.confirmed = confirmed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the suffix element of the input field, for example a clear button.
    pub fn suffix<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> E + 'static,
        E: IntoElement,
    {
        self.suffix = Some(Box::new(move |window, cx| {
            builder(window, cx).into_any_element()
        }));
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_mouse_down(
        mut self,
        button: MouseButton,
        handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_mouse_down.insert(button, Box::new(handler));
        self
    }

    pub fn on_mouse_enter(
        mut self,
        handler: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_mouse_enter = Some(Box::new(handler));
        self
    }

    /// Pin the row to a fixed height.
    ///
    /// Defaults are derived from `px_3()` padding + content, so when the
    /// row contains icon + label + suffix the natural height lands near
    /// 22 px. For lists like the Source Control resource tree where
    /// VS Code pins `.scm-row` to 22 px explicitly, prefer [`Self::compact`].
    pub fn h(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Opt into a 22 px row, matching VS Code's source-control row
    /// density (`scm.css .monaco-list-row.scm-row { height: 22px }`).
    ///
    /// The Git Panel resource renderer uses this so the row height does
    /// not depend on the icon's intrinsic height when the panel is
    /// resized to a narrow column. PR-7b in
    /// `docs/superpowers/git-panel/08-execution-pr-plan.md`.
    pub fn compact(mut self) -> Self {
        self.height = Some(px(22.).into());
        self
    }
}

impl Disableable for ListItem {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for ListItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn secondary_selected(mut self, selected: bool) -> Self {
        self.secondary_selected = selected;
        self
    }
}

impl Styled for ListItem {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for ListItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_active = self.confirmed || self.selected || self.secondary_selected;

        let corner_radii = self.style.corner_radii.clone();

        let mut selected_style = StyleRefinement::default();
        selected_style.corner_radii = corner_radii;

        let is_selectable = !(self.disabled || self.mode.is_separator());

        self.base
            .relative()
            .gap_x_1()
            .py_1()
            .px_3()
            .text_base()
            .text_color(cx.theme().foreground)
            .relative()
            .items_center()
            .justify_between()
            .when_some(self.height, |this, height| this.h(height))
            .refine_style(&self.style)
            .when(is_selectable, |this| {
                this.when_some(self.on_click, |this, on_click| this.on_click(on_click))
                    .when_some(self.on_mouse_enter, |this, on_mouse_enter| {
                        this.on_mouse_move(move |ev, window, cx| (on_mouse_enter)(ev, window, cx))
                    })
                    .map(|this| {
                        self.on_mouse_down
                            .into_iter()
                            .fold(this, |this, (button, handler)| {
                                this.on_mouse_down(button, move |ev, window, cx| {
                                    handler(ev, window, cx)
                                })
                            })
                    })
                    .when(!is_active, |this| {
                        this.hover(|this| this.bg(cx.theme().tokens.list_hover))
                    })
            })
            .when(!is_selectable, |this| {
                this.text_color(cx.theme().muted_foreground)
            })
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .gap_x_1()
                    .child(div().w_full().children(self.children))
                    .when_some(self.check_icon, |this, icon| {
                        this.child(
                            div().w_5().items_center().justify_center().when(
                                self.confirmed,
                                |this| {
                                    this.child(icon.small().text_color(cx.theme().muted_foreground))
                                },
                            ),
                        )
                    }),
            )
            .when_some(self.suffix, |this, suffix| this.child(suffix(window, cx)))
            .map(|this| {
                if is_selectable && (self.selected || self.secondary_selected) {
                    let bg = if self.selected && cx.theme().list.active_highlight {
                        cx.theme().list_active
                    } else {
                        cx.theme().accent
                    };

                    this.when(!self.secondary_selected, |this| this.bg(bg))
                        .when(cx.theme().list.active_highlight, |this| {
                            this.child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .right_0()
                                    .bottom_0()
                                    .border_1()
                                    .border_color(cx.theme().list_active_border)
                                    .refine_style(&selected_style),
                            )
                        })
                } else {
                    this
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn default_list_item_has_no_height_override() {
        // Without calling `h(...)` or `compact()`, the builder keeps its
        // natural padding-derived height. The height field defaults to
        // `None`, which is what the render branches on.
        let item = ListItem::new("git-panel-row");
        assert!(item.height.is_none(), "no row-height override by default");
    }

    #[test]
    fn compact_pins_row_to_22_px() {
        // `compact()` is the Source Control resource-tree API surface
        // (PR-7b in `docs/superpowers/git-panel/08-execution-pr-plan.md`).
        // The accepted value is the fixed 22 px row VS Code uses in
        // `scm.css .monaco-list-row.scm-row`.
        let item = ListItem::new("git-panel-row").compact();
        match item.height {
            Some(DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(pixels))) => {
                assert_eq!(pixels, px(22.), "compact must be exactly 22 px");
            }
            other => panic!("expected absolute 22 px height, got {other:?}"),
        }
    }

    #[test]
    fn h_accepts_arbitrary_definite_length() {
        let item = ListItem::new("git-panel-row").h(px(36.));
        match item.height {
            Some(DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(pixels))) => {
                assert_eq!(pixels, px(36.), "h() must propagate the explicit value");
            }
            other => panic!("expected absolute 36 px height, got {other:?}"),
        }
    }
}
