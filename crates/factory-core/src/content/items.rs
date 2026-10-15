//! Items, resources, tools and ore blocks the engine declares, from `items.toml`.

use std::collections::BTreeMap;
use std::ops::Range;

use serde::Deserialize;
use toml::Spanned;

use super::error::ContentError;
use super::sources::Sources;
use crate::contract::ItemId;

/// The namespace of everything the engine declares: `satiscraft:flint_blade`.
pub const NAMESPACE: &str = "satiscraft";

/// Every item the engine declares, by id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Items {
    items: BTreeMap<ItemId, Item>,
}

/// One item the engine declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: ItemId,
}

impl Items {
    pub fn get(&self, id: &ItemId) -> Option<&Item> {
        self.items.get(id)
    }

    /// Every item, by id.
    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.values()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// `items.toml` as written, defaults and overrides merged.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ItemsFile {
    #[serde(default)]
    items: Vec<ItemEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemEntry {
    id: Spanned<ItemId>,
}

impl ItemsFile {
    /// The items, each declared once, in the engine's namespace, by a default file: Minecraft
    /// registers items from the defaults embedded in the mod, so an override cannot add one.
    /// Problems go to `errors`.
    pub(super) fn validate(self, sources: &Sources<'_>, errors: &mut Vec<ContentError>) -> Items {
        let mut declared: BTreeMap<ItemId, Range<usize>> = BTreeMap::new();
        let mut items = Items::default();
        for entry in self.items {
            let span = entry.id.span();
            let id = entry.id.into_inner();
            match problem(&id, span.clone(), &declared, sources) {
                Some(message) => errors.push(sources.error(span, message)),
                None => {
                    declared.insert(id.clone(), span);
                    items.items.insert(id.clone(), Item { id });
                }
            }
        }
        items
    }
}

/// What is wrong with declaring `id` at `span`, after the items `declared` so far.
fn problem(
    id: &ItemId,
    span: Range<usize>,
    declared: &BTreeMap<ItemId, Range<usize>>,
    sources: &Sources<'_>,
) -> Option<String> {
    if !in_namespace(id) {
        return Some(format!(
            "item `{id}` is outside the `{NAMESPACE}` namespace: the engine declares only its \
             own items"
        ));
    }
    if sources.is_override(span) {
        return Some(format!(
            "unknown item `{id}`: an override changes the engine's items, it cannot add one"
        ));
    }
    let first = declared.get(id)?;
    Some(format!(
        "item `{id}` is declared twice, first at {}",
        sources.locate(first.clone())
    ))
}

fn in_namespace(id: &ItemId) -> bool {
    id.as_str().split_once(':').map(|(namespace, _)| namespace) == Some(NAMESPACE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(text: &str) -> ItemId {
        ItemId::new(text).unwrap()
    }

    #[test]
    fn items_are_found_by_id_and_listed_in_id_order() {
        let mut items = Items::default();
        assert!(items.is_empty());
        for text in ["satiscraft:rope", "satiscraft:flint_blade"] {
            items.items.insert(id(text), Item { id: id(text) });
        }
        assert_eq!(items.len(), 2);
        assert_eq!(
            items.get(&id("satiscraft:rope")).map(|item| &item.id),
            Some(&id("satiscraft:rope"))
        );
        assert_eq!(items.get(&id("satiscraft:stick")), None);
        let listed: Vec<_> = items.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(listed, ["satiscraft:flint_blade", "satiscraft:rope"]);
    }

    #[test]
    fn only_the_engine_namespace_is_its_own() {
        assert!(in_namespace(&id("satiscraft:rope")));
        assert!(!in_namespace(&id("minecraft:stick")));
        assert!(!in_namespace(&id("satiscraft_world:rope")));
    }
}
