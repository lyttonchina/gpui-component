use std::collections::BTreeMap;

use gpui::{App, Global, SharedString};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextValue {
    Bool(bool),
    String(SharedString),
    Enum(SharedString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextKeyExpr {
    True,
    False,
    Equals {
        key: SharedString,
        value: ContextValue,
    },
    NotEquals {
        key: SharedString,
        value: ContextValue,
    },
    Has {
        key: SharedString,
    },
    Not(Box<Self>),
    And(Vec<Self>),
    Or(Vec<Self>),
    Chord(Vec<Self>),
}

impl ContextKeyExpr {
    pub fn evaluate(&self, context: &ContextKeyContext) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
            Self::Equals { key, value } => context.get(key) == Some(value),
            Self::NotEquals { key, value } => context.get(key) != Some(value),
            Self::Has { key } => context.has(key),
            Self::Not(expression) => !expression.evaluate(context),
            Self::And(expressions) | Self::Chord(expressions) => expressions
                .iter()
                .all(|expression| expression.evaluate(context)),
            Self::Or(expressions) => expressions
                .iter()
                .any(|expression| expression.evaluate(context)),
        }
    }

    pub fn equals(key: impl Into<SharedString>, value: ContextValue) -> Self {
        Self::Equals {
            key: key.into(),
            value,
        }
    }

    pub fn has(key: impl Into<SharedString>) -> Self {
        Self::Has { key: key.into() }
    }

    pub fn and(self, other: Self) -> Self {
        match self {
            Self::And(mut expressions) => {
                expressions.push(other);
                Self::And(expressions)
            }
            expression => Self::And(vec![expression, other]),
        }
    }

    pub fn or(self, other: Self) -> Self {
        match self {
            Self::Or(mut expressions) => {
                expressions.push(other);
                Self::Or(expressions)
            }
            expression => Self::Or(vec![expression, other]),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContextKeyContext {
    keys: BTreeMap<SharedString, ContextValue>,
}

impl ContextKeyContext {
    pub fn set(&mut self, key: impl Into<SharedString>, value: ContextValue) {
        self.keys.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&ContextValue> {
        self.keys.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.keys.contains_key(key)
    }

    pub fn unset(&mut self, key: &str) {
        self.keys.remove(key);
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }
}

#[derive(Clone, Debug, Default)]
pub struct ContextKeyService {
    context: ContextKeyContext,
}

impl Global for ContextKeyService {}

impl ContextKeyService {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    pub fn set(&mut self, key: impl Into<SharedString>, value: ContextValue) {
        self.context.set(key, value);
    }

    pub fn unset(&mut self, key: &str) {
        self.context.unset(key);
    }

    pub fn evaluate(&self, expression: &ContextKeyExpr) -> bool {
        expression.evaluate(&self.context)
    }

    pub fn context(&self) -> &ContextKeyContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_composed_expressions() {
        let mut context = ContextKeyContext::default();
        context.set("resource", ContextValue::Enum("file".into()));
        context.set("focused", ContextValue::Bool(true));

        let expression = ContextKeyExpr::equals("resource", ContextValue::Enum("file".into()))
            .and(ContextKeyExpr::has("focused"));
        assert!(expression.evaluate(&context));
        assert!(!ContextKeyExpr::Not(Box::new(expression)).evaluate(&context));
    }

    #[test]
    fn unsetting_a_key_changes_has_and_equals() {
        let mut context = ContextKeyContext::default();
        context.set("resource", ContextValue::String("changes".into()));
        context.unset("resource");

        assert!(!ContextKeyExpr::has("resource").evaluate(&context));
        assert!(
            !ContextKeyExpr::equals("resource", ContextValue::String("changes".into()))
                .evaluate(&context)
        );
    }
}
