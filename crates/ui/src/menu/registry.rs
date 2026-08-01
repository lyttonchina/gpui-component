use std::{collections::BTreeMap, rc::Rc};

use gpui::{App, Global, SharedString, Window};

use crate::{ContextKeyExpr, ContextKeyService, IconName};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MenuId(SharedString);

impl MenuId {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for MenuId {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

#[derive(Clone)]
pub struct MenuItemAction {
    pub id: SharedString,
    pub title: SharedString,
    pub icon: Option<IconName>,
    pub group: SharedString,
    pub order: i32,
    pub when: Option<ContextKeyExpr>,
    pub enabled: Option<ContextKeyExpr>,
    pub on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl MenuItemAction {
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon: None,
            group: "navigation".into(),
            order: 0,
            when: None,
            enabled: None,
            on_click: None,
        }
    }

    pub fn group(mut self, group: impl Into<SharedString>, order: i32) -> Self {
        self.group = group.into();
        self.order = order;
        self
    }

    pub fn when(mut self, expression: ContextKeyExpr) -> Self {
        self.when = Some(expression);
        self
    }

    pub fn enabled_when(mut self, expression: ContextKeyExpr) -> Self {
        self.enabled = Some(expression);
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

#[derive(Clone)]
pub struct ResolvedMenuItem {
    pub action: MenuItemAction,
    pub enabled: bool,
}

#[derive(Default)]
pub struct MenuRegistry {
    menus: BTreeMap<MenuId, Vec<MenuItemAction>>,
}

impl Global for MenuRegistry {}

impl MenuRegistry {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    pub fn append_menu_item(&mut self, menu_id: impl Into<MenuId>, item: MenuItemAction) {
        self.menus.entry(menu_id.into()).or_default().push(item);
    }

    pub fn clear_menu(&mut self, menu_id: &MenuId) {
        self.menus.remove(menu_id);
    }

    pub fn get_menu(&self, menu_id: &MenuId, cx: &App) -> Vec<ResolvedMenuItem> {
        let context_keys = ContextKeyService::global(cx);
        self.resolve_menu(menu_id, |expression| context_keys.evaluate(expression))
    }

    fn resolve_menu(
        &self,
        menu_id: &MenuId,
        evaluate: impl Fn(&ContextKeyExpr) -> bool,
    ) -> Vec<ResolvedMenuItem> {
        let mut items = self
            .menus
            .get(menu_id)
            .into_iter()
            .flatten()
            .filter(|item| item.when.as_ref().is_none_or(&evaluate))
            .map(|item| ResolvedMenuItem {
                enabled: item.enabled.as_ref().is_none_or(&evaluate),
                action: item.clone(),
            })
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.action
                .group
                .cmp(&right.action.group)
                .then(left.action.order.cmp(&right.action.order))
                .then(left.action.id.cmp(&right.action.id))
        });
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContextKeyContext, ContextValue};

    #[test]
    fn resolves_visibility_enabled_state_and_order() {
        let menu_id = MenuId::new("scm/resourceState/context");
        let mut registry = MenuRegistry::default();
        registry.append_menu_item(
            menu_id.clone(),
            MenuItemAction::new("stage", "Stage")
                .group("1_modification", 20)
                .when(ContextKeyExpr::has("resource")),
        );
        registry.append_menu_item(
            menu_id.clone(),
            MenuItemAction::new("open", "Open").group("0_navigation", 10),
        );
        registry.append_menu_item(
            menu_id.clone(),
            MenuItemAction::new("discard", "Discard")
                .group("1_modification", 10)
                .enabled_when(ContextKeyExpr::equals("writable", ContextValue::Bool(true))),
        );

        let mut context = ContextKeyContext::default();
        context.set("resource", ContextValue::Enum("file".into()));
        let items = registry.resolve_menu(&menu_id, |expression| expression.evaluate(&context));

        assert_eq!(
            items
                .iter()
                .map(|item| item.action.id.as_ref())
                .collect::<Vec<_>>(),
            ["open", "discard", "stage"]
        );
        assert!(!items[1].enabled);
    }
}
