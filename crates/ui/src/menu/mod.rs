use gpui::App;

mod app_menu_bar;
mod context_menu;
mod dropdown_menu;
mod menu_item;
mod popup_menu;
mod registry;

pub use app_menu_bar::AppMenuBar;
pub use context_menu::{ContextMenu, ContextMenuExt, ContextMenuState};
pub use dropdown_menu::DropdownMenu;
pub use popup_menu::{PopupMenu, PopupMenuItem};
pub use registry::{MenuId, MenuItemAction, MenuRegistry, ResolvedMenuItem};

pub(crate) fn init(cx: &mut App) {
    cx.set_global(MenuRegistry::default());
    app_menu_bar::init(cx);
    popup_menu::init(cx);
}
