use std::ops::Range;

use crate::{
    IconName, Sizable, Size, StyledExt,
    group_box::GroupBoxVariant,
    input::{Input, InputState},
    resizable::{h_resizable, resizable_panel},
    setting::{SettingGroup, SettingPage},
    sidebar::{Sidebar, SidebarMenu, SidebarMenuItem},
};
use gpui::{
    App, AppContext as _, Axis, ElementId, Entity, IntoElement, ParentElement as _, Pixels,
    RenderOnce, SharedString, StyleRefinement, Styled, Window, container_query, div,
    prelude::FluentBuilder as _, px, relative,
};
use rust_i18n::t;

const STACKED_LAYOUT_MAX_WIDTH: Pixels = px(480.);

/// The settings structure containing multiple pages for app settings.
///
/// The hierarchy of settings is as follows:
///
/// ```ignore
/// Settings
///   SettingPage     <- The single active page displayed
///     SettingGroup
///       SettingItem
///         Label
///         SettingField (e.g., Switch, Dropdown, Input)
/// ```
#[derive(IntoElement)]
pub struct Settings {
    id: ElementId,
    pages: Vec<SettingPage>,
    group_variant: GroupBoxVariant,
    size: Size,
    sidebar_width: Pixels,
    sidebar_size_range: Range<Pixels>,
    sidebar_style: StyleRefinement,
    default_selected_index: SelectIndex,
    header_style: StyleRefinement,
    show_sidebar: bool,
    show_sidebar_search: bool,
    external_state: Option<Entity<SettingsState>>,
}

impl Settings {
    /// Create a new settings with the given ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            pages: vec![],
            group_variant: GroupBoxVariant::default(),
            size: Size::default(),
            sidebar_width: px(250.0),
            sidebar_size_range: px(160.0)..px(360.0),
            sidebar_style: StyleRefinement::default(),
            default_selected_index: SelectIndex::default(),
            header_style: StyleRefinement::default(),
            show_sidebar: true,
            show_sidebar_search: true,
            external_state: None,
        }
    }

    /// Set the width of the sidebar, default is `250px`.
    pub fn sidebar_width(mut self, width: impl Into<Pixels>) -> Self {
        self.sidebar_width = width.into();
        self
    }

    /// Set the resize range of the sidebar, default is `160px..360px`.
    pub fn sidebar_size_range(mut self, range: impl Into<Range<Pixels>>) -> Self {
        self.sidebar_size_range = range.into();
        self
    }

    /// Add a page to the settings.
    pub fn page(mut self, page: SettingPage) -> Self {
        self.pages.push(page);
        self
    }

    /// Add pages to the settings.
    pub fn pages(mut self, pages: impl IntoIterator<Item = SettingPage>) -> Self {
        self.pages.extend(pages);
        self
    }

    /// Set the default variant for all setting groups.
    ///
    /// All setting groups will use this variant unless overridden individually.
    pub fn with_group_variant(mut self, variant: GroupBoxVariant) -> Self {
        self.group_variant = variant;
        self
    }

    /// Set the style refinement for the sidebar.
    pub fn sidebar_style(mut self, style: &StyleRefinement) -> Self {
        self.sidebar_style = style.clone();
        self
    }

    /// Set the default index of the page to be selected.
    pub fn default_selected_index(mut self, index: SelectIndex) -> Self {
        self.default_selected_index = index;
        self
    }

    /// Set the style refinement for the header.
    pub fn header_style(mut self, style: &StyleRefinement) -> Self {
        self.header_style = style.clone();
        self
    }

    /// Show or hide the sidebar entirely. Default `true`.
    pub fn show_sidebar(mut self, visible: bool) -> Self {
        self.show_sidebar = visible;
        self
    }

    /// Show or hide the search input rendered inside the sidebar header. When
    /// the host application owns a separate search input, hiding this avoids
    /// two competing inputs. Default `true`.
    pub fn show_sidebar_search(mut self, visible: bool) -> Self {
        self.show_sidebar_search = visible;
        self
    }

    /// Share an externally managed [`SettingsState`] so the host can drive
    /// navigation from outside (`SettingsState::navigate`, `set_search`).
    pub fn state(mut self, state: Entity<SettingsState>) -> Self {
        self.external_state = Some(state);
        self
    }

    fn filtered_pages(
        &self,
        query: &str,
        cx: &mut App,
        state: &Entity<SettingsState>,
    ) -> Vec<SettingPage> {
        let selected_index = state.read(cx).selected_index;
        let selected_page = self.pages.get(selected_index.page_ix).cloned();
        let pages = self
            .pages
            .iter()
            .filter_map(|page| {
                let filtered_groups: Vec<SettingGroup> = page
                    .groups
                    .iter()
                    .filter_map(|group| {
                        let mut group = group.clone();
                        group.items = group
                            .items
                            .iter()
                            .filter(|item| item.is_match(&query, cx))
                            .cloned()
                            .collect();
                        if group.items.is_empty() {
                            None
                        } else {
                            Some(group)
                        }
                    })
                    .collect();
                let mut page = page.clone();
                page.groups = filtered_groups;
                if page.groups.is_empty() {
                    None
                } else {
                    Some(page)
                }
            })
            .collect::<Vec<_>>();
        // Always preserve the currently selected page even when filtering
        // would otherwise remove it, so deep links and active selections
        // remain stable as the user types in the search box.
        if let Some(active) = selected_page {
            if !pages.iter().any(|p| p.page_id() == active.page_id()) {
                let mut active = active;
                let q = query.to_string();
                active.groups = active
                    .groups
                    .into_iter()
                    .filter_map(|mut group| {
                        group.items = group
                            .items
                            .into_iter()
                            .filter(|item| item.is_match(&q, cx))
                            .collect();
                        if group.items.is_empty() {
                            None
                        } else {
                            Some(group)
                        }
                    })
                    .collect();
                if !active.groups.is_empty() {
                    let mut combined = Vec::with_capacity(pages.len() + 1);
                    combined.push(active);
                    combined.extend(pages);
                    return combined;
                }
            }
        }
        pages
    }

    /// Build a (page_id, group_id, item_id) -> (page_ix, group_ix, item_ix)
    /// lookup table from `pages`. The table is rebuilt every render so the
    /// host does not have to keep its own in sync with page additions.
    fn build_id_index(
        &self,
        pages: &[SettingPage],
    ) -> std::collections::HashMap<String, (usize, Option<usize>, Option<usize>)> {
        let mut index = std::collections::HashMap::new();
        for (page_ix, page) in pages.iter().enumerate() {
            if let Some(page_id) = page.page_id() {
                index.insert(page_id.to_string(), (page_ix, None, None));
            }
            for (group_ix, group) in page.groups.iter().enumerate() {
                if let Some(group_id) = group.group_id() {
                    let key = format!(
                        "{}#{}",
                        page.page_id().map(|s| s.to_string()).unwrap_or_default(),
                        group_id
                    );
                    index.insert(key, (page_ix, Some(group_ix), None));
                }
                for (item_ix, item) in group.items.iter().enumerate() {
                    if let Some(item_id) = item.item_id() {
                        let page_id = page.page_id().map(|s| s.to_string()).unwrap_or_default();
                        let key = format!("{page_id}#{item_id}");
                        index.insert(key, (page_ix, Some(group_ix), Some(item_ix)));
                    }
                }
            }
        }
        index
    }

    fn render_active_page(
        &self,
        state: &Entity<SettingsState>,
        pages: &Vec<SettingPage>,
        _id_index: &std::collections::HashMap<String, (usize, Option<usize>, Option<usize>)>,
        options: &RenderOptions,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let selected_index = state.read(cx).selected_index;

        for (ix, page) in pages.into_iter().enumerate() {
            if selected_index.page_ix == ix {
                return page
                    .render(ix, state, &options, window, cx)
                    .into_any_element();
            }
        }

        return div().into_any_element();
    }

    fn render_sidebar(
        &self,
        state: &Entity<SettingsState>,
        pages: &Vec<SettingPage>,
        _: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let selected_index = state.read(cx).selected_index;
        let search_input = state.read(cx).search_input.clone();

        Sidebar::new("settings-sidebar")
            .w(relative(1.))
            .border_0()
            .refine_style(&self.sidebar_style)
            .collapsible(false)
            .collapsed(false)
            .when(self.show_sidebar_search, |this| {
                this.header(
                    div()
                        .w_full()
                        .refine_style(&self.header_style)
                        .child(Input::new(&search_input).prefix(IconName::Search)),
                )
            })
            .child(
                SidebarMenu::new().children(pages.iter().enumerate().map(|(page_ix, page)| {
                    let is_page_active =
                        selected_index.page_ix == page_ix && selected_index.group_ix.is_none();
                    SidebarMenuItem::new(page.title.clone())
                        .click_to_open(true)
                        .when_some(page.icon.clone(), |this, icon| this.icon(icon))
                        .default_open(page.default_open)
                        .active(is_page_active)
                        .on_click({
                            let state = state.clone();
                            move |_, _, cx| {
                                state.update(cx, |state, cx| {
                                    state.selected_index = SelectIndex {
                                        page_ix,
                                        ..Default::default()
                                    };
                                    cx.notify();
                                })
                            }
                        })
                        .when(page.groups.len() > 1, |this| {
                            this.children(
                                page.groups
                                    .iter()
                                    .filter(|g| g.title.is_some())
                                    .enumerate()
                                    .map(|(group_ix, group)| {
                                        let is_active = selected_index.page_ix == page_ix
                                            && selected_index.group_ix == Some(group_ix);
                                        let title = group.title.clone().unwrap_or_default();

                                        SidebarMenuItem::new(title).active(is_active).on_click({
                                            let state = state.clone();
                                            move |_, _, cx| {
                                                state.update(cx, |state, cx| {
                                                    state.selected_index = SelectIndex {
                                                        page_ix,
                                                        group_ix: Some(group_ix),
                                                    };
                                                    state.deferred_scroll_group_ix = Some(group_ix);
                                                    cx.notify();
                                                })
                                            }
                                        })
                                    }),
                            )
                        })
                })),
            )
    }

    fn resolve_state(&self, window: &mut Window, cx: &mut App) -> Entity<SettingsState> {
        if let Some(external) = self.external_state.clone() {
            return external;
        }
        window.use_keyed_state(self.id.clone(), cx, |window, cx| {
            let search_input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(t!("Settings.search_placeholder"))
                    .default_value("")
            });

            SettingsState {
                search_input,
                selected_index: self.default_selected_index,
                deferred_scroll_group_ix: None,
                id_index: std::collections::HashMap::new(),
                focused_item_ix: None,
            }
        })
    }
}

impl Sizable for Settings {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

/// Settings navigation state shared between the host application and the
/// `Settings` element. Exposed publicly so callers can drive deep links,
/// share a search input across their own toolbar, and observe which page the
/// user last activated.
#[derive(Clone)]
pub struct SettingsState {
    pub(super) selected_index: SelectIndex,
    /// If set, defer scrolling to this group index after rendering.
    pub(super) deferred_scroll_group_ix: Option<usize>,
    pub(super) search_input: Entity<InputState>,
    /// Lookup table from stable IDs to indices. Rebuilt each render via
    /// [`SettingsState::sync_pages`]; used by [`SettingsState::navigate`] and
    /// [`SettingsState::resolve`].
    pub(super) id_index: std::collections::HashMap<String, (usize, Option<usize>, Option<usize>)>,
    pub(super) focused_item_ix: Option<usize>,
}

fn resolve_stable_id(
    index: &std::collections::HashMap<String, (usize, Option<usize>, Option<usize>)>,
    page_id: &str,
    group_id: Option<&str>,
    item_id: Option<&str>,
) -> Option<(usize, Option<usize>, Option<usize>)> {
    if let Some(item_id) = item_id {
        return index.get(&format!("{page_id}#{item_id}")).copied();
    }
    if let Some(group_id) = group_id {
        return index.get(&format!("{page_id}#{group_id}")).copied();
    }
    index.get(page_id).copied()
}

fn resolve_filtered_selection(
    pages: &[SettingPage],
    page_id: Option<&str>,
    group_id: Option<&str>,
) -> SelectIndex {
    let page_ix = page_id
        .and_then(|page_id| {
            pages
                .iter()
                .position(|page| page.page_id().as_deref() == Some(page_id.into()))
        })
        .unwrap_or(0);
    let group_ix = group_id.and_then(|group_id| {
        pages.get(page_ix).and_then(|page| {
            page.groups
                .iter()
                .position(|group| group.group_id().as_deref() == Some(group_id.into()))
        })
    });
    SelectIndex { page_ix, group_ix }
}

impl SettingsState {
    /// Construct a fresh state with a private search input attached to the
    /// supplied window. Used by the host when it wants its own state entity
    /// (e.g. to share the search box with a toolbar).
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        let search_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(t!("Settings.search_placeholder"))
                .default_value("")
        });
        Self {
            search_input,
            selected_index: SelectIndex::default(),
            deferred_scroll_group_ix: None,
            id_index: std::collections::HashMap::new(),
            focused_item_ix: None,
        }
    }

    /// Underlying shared search input. Set its value from the host toolbar to
    /// drive filtering, or read it to mirror whatever the sidebar shows.
    pub fn search_input(&self) -> Entity<InputState> {
        self.search_input.clone()
    }

    /// Page index currently shown in the content area.
    pub fn selected_page_index(&self) -> usize {
        self.selected_index.page_ix
    }

    /// Group index currently focused inside the active page, if any.
    pub fn selected_group_index(&self) -> Option<usize> {
        self.selected_index.group_ix
    }

    pub fn focused_item_index(&self) -> Option<usize> {
        self.focused_item_ix
    }

    /// Replace the current selection. Pass `None` for `group_ix` to focus the
    /// page itself; pass a value to focus a specific group within it. The
    /// caller is expected to invoke `cx.notify()` on the entity that owns
    /// this state (typically via `state.update(cx, |s, _| s.set_selected_index(...))`).
    pub fn set_selected_index(&mut self, page_ix: usize, group_ix: Option<usize>) {
        self.selected_index = SelectIndex { page_ix, group_ix };
        self.focused_item_ix = None;
        if let Some(ix) = group_ix {
            self.deferred_scroll_group_ix = Some(ix);
        }
    }

    /// Drive navigation from a stable identifier. The IDs are produced by
    /// `SettingPage::id`, `SettingGroup::id`, and `SettingItem::id`. Returns
    /// `true` when a matching entry was found and the state changed.
    ///
    /// Callers are expected to call [`SettingsState::sync_pages`] before this
    /// method so the lookup table reflects the current page list. The default
    /// built by `Settings::new`/`SettingsState::new` is empty until the host
    /// registers pages via `sync_pages`. When the lookup succeeds the caller
    /// is expected to invoke `cx.notify()` on the entity that owns this
    /// state (typically via `state.update(cx, |s, _| s.navigate(...))` and
    /// then re-rendering the host).
    pub fn navigate(
        &mut self,
        page_id: &str,
        group_id: Option<&str>,
        item_id: Option<&str>,
    ) -> bool {
        let Some((page_ix, group_ix, item_ix)) =
            resolve_stable_id(&self.id_index, page_id, group_id, item_id)
        else {
            return false;
        };

        self.selected_index = SelectIndex { page_ix, group_ix };
        self.focused_item_ix = item_ix;
        if let Some(ix) = group_ix {
            self.deferred_scroll_group_ix = Some(ix);
        }
        true
    }

    /// Replace the page ID lookup table used by [`SettingsState::navigate`].
    /// The host should call this whenever the page list changes (e.g. before
    /// invoking `navigate` after re-rendering). The index maps
    /// `"<page_id>#<group_or_item_id>"` → `(page_ix, group_ix, item_ix)`.
    pub fn sync_pages(
        &mut self,
        index: std::collections::HashMap<String, (usize, Option<usize>, Option<usize>)>,
    ) {
        self.id_index = index;
    }

    /// Look up a previously synced `(page_ix, group_ix, item_ix)` for the
    /// given stable IDs. Returns `None` if any of the requested IDs is
    /// missing from the current page list.
    pub fn resolve(
        &self,
        page_id: &str,
        group_id: Option<&str>,
        item_id: Option<&str>,
    ) -> Option<(usize, Option<usize>, Option<usize>)> {
        resolve_stable_id(&self.id_index, page_id, group_id, item_id)
    }

    /// Replace the search input value (used by host toolbars that own their
    /// own search box).
    pub fn set_search(&self, query: impl Into<SharedString>, window: &mut Window, cx: &mut App) {
        let query = query.into();
        self.search_input.update(cx, |state, cx| {
            state.set_value(query, window, cx);
        });
    }
}

/// Options for rendering setting item.
#[derive(Clone, Copy)]
pub struct RenderOptions {
    pub page_ix: usize,
    pub group_ix: usize,
    pub item_ix: usize,
    pub size: Size,
    pub group_variant: GroupBoxVariant,
    pub layout: Axis,
    pub disabled: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectIndex {
    pub page_ix: usize,
    pub group_ix: Option<usize>,
}

impl RenderOnce for Settings {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.resolve_state(window, cx);

        let query = state.read(cx).search_input.read(cx).value();
        let selected_index = state.read(cx).selected_index;
        let selected_page_id = self
            .pages
            .get(selected_index.page_ix)
            .and_then(SettingPage::page_id);
        let selected_group_id = selected_index.group_ix.and_then(|group_ix| {
            self.pages
                .get(selected_index.page_ix)
                .and_then(|page| page.groups.get(group_ix))
                .and_then(SettingGroup::group_id)
        });
        let filtered_pages = self.filtered_pages(&query, cx, &state);
        let filtered_selection = resolve_filtered_selection(
            &filtered_pages,
            selected_page_id.as_deref().map(|value| value.as_ref()),
            selected_group_id.as_deref().map(|value| value.as_ref()),
        );
        let id_index = self.build_id_index(&filtered_pages);
        state.update(cx, |state, _cx| {
            state.selected_index = filtered_selection;
            state.sync_pages(id_index.clone());
        });
        let options = RenderOptions {
            page_ix: 0,
            group_ix: 0,
            item_ix: 0,
            size: self.size,
            group_variant: self.group_variant,
            layout: Axis::Horizontal,
            disabled: false,
        };
        let sidebar_size_range = self.sidebar_size_range.clone();
        let sidebar = self
            .render_sidebar(&state, &filtered_pages, window, cx)
            .into_any_element();

        let show_sidebar = self.show_sidebar;
        let sidebar_width = self.sidebar_width;
        let id = self.id.clone();
        let page_panel = container_query(move |size, window, cx| {
            let options = RenderOptions {
                layout: if size.width <= STACKED_LAYOUT_MAX_WIDTH {
                    Axis::Vertical
                } else {
                    Axis::Horizontal
                },
                ..options
            };
            self.render_active_page(&state, &filtered_pages, &id_index, &options, window, cx)
        });

        if !show_sidebar {
            return div().size_full().child(page_panel);
        }

        div().size_full().child(
            h_resizable(id)
                .child(
                    resizable_panel()
                        .size(sidebar_width)
                        .size_range(sidebar_size_range.clone())
                        .child(sidebar),
                )
                .child(resizable_panel().child(page_panel)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_id_lookup_resolves_page_group_and_item() {
        let index = std::collections::HashMap::from([
            ("editor".to_string(), (2, None, None)),
            ("editor#editor.appearance".to_string(), (2, Some(1), None)),
            ("editor#editor.font-size".to_string(), (2, Some(1), Some(3))),
        ]);

        assert_eq!(
            resolve_stable_id(&index, "editor", None, None),
            Some((2, None, None)),
        );
        assert_eq!(
            resolve_stable_id(&index, "editor", Some("editor.appearance"), None,),
            Some((2, Some(1), None)),
        );
        assert_eq!(
            resolve_stable_id(&index, "editor", None, Some("editor.font-size"),),
            Some((2, Some(1), Some(3))),
        );
    }

    #[test]
    fn filtered_selection_follows_stable_page_and_group_ids() {
        let pages = vec![
            SettingPage::new("Editor")
                .id("editor")
                .group(SettingGroup::new().id("editor.appearance")),
            SettingPage::new("AI")
                .id("ai")
                .group(SettingGroup::new().id("ai.manage")),
        ];

        assert_eq!(
            resolve_filtered_selection(&pages, Some("ai"), Some("ai.manage")),
            SelectIndex {
                page_ix: 1,
                group_ix: Some(0),
            },
        );
    }
}
