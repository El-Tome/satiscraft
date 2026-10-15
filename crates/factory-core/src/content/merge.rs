//! How an override changes a default: tables key by key, entries by their `id`, any other
//! value replaced whole. An override thus only states what it changes.

use std::collections::BTreeMap;

use toml::Spanned;
use toml::de::{DeArray, DeTable, DeValue};

use super::error::ContentError;
use super::sources::Sources;

/// The key that names an entry of an array of tables (`[[items]]`).
const ID: &str = "id";

/// Lays `over` onto `base`:
/// - a table merges key by key, recursively;
/// - an array whose default entries are all tables with an `id` merges entry by entry: an
///   override entry changes the default entry of the same `id`, or comes after them all;
/// - any other value replaces the default one.
///
/// A changed entry keeps the `id`, and so the span, of its default: validation tells the
/// entries an override changed from those it added.
pub(crate) fn merge_table<'a>(
    base: &mut DeTable<'a>,
    over: DeTable<'a>,
    sources: &Sources<'_>,
    errors: &mut Vec<ContentError>,
) {
    for (key, value) in over {
        let name: &str = key.get_ref();
        match base.get_mut(name) {
            Some(existing) => merge_value(existing, value, sources, errors),
            None => {
                base.insert(key, value);
            }
        }
    }
}

fn merge_value<'a>(
    base: &mut Spanned<DeValue<'a>>,
    over: Spanned<DeValue<'a>>,
    sources: &Sources<'_>,
    errors: &mut Vec<ContentError>,
) {
    let span = over.span();
    if let Some(replacement) = merge_into(base.get_mut(), over.into_inner(), sources, errors) {
        *base = Spanned::new(span, replacement);
    }
}

/// Merges `over` into `base` when they merge, or hands `over` back to replace `base`.
fn merge_into<'a>(
    base: &mut DeValue<'a>,
    over: DeValue<'a>,
    sources: &Sources<'_>,
    errors: &mut Vec<ContentError>,
) -> Option<DeValue<'a>> {
    match (base, over) {
        (DeValue::Table(base), DeValue::Table(over)) => merge_table(base, over, sources, errors),
        (DeValue::Array(base), DeValue::Array(over)) if is_keyed(base) => {
            merge_by_id(base, over, sources, errors);
        }
        (_, over) => return Some(over),
    }
    None
}

/// Merges each entry of `over` into the entry of `base` with the same `id`, or appends it.
fn merge_by_id<'a>(
    base: &mut DeArray<'a>,
    over: DeArray<'a>,
    sources: &Sources<'_>,
    errors: &mut Vec<ContentError>,
) {
    let mut seen = BTreeMap::new();
    for entry in over {
        let span = entry.span();
        let Some(id) = id_of(&entry).map(str::to_owned) else {
            let message = format!(
                "this entry has no `{ID}`: an override names the entry it changes by its `{ID}`"
            );
            errors.push(sources.error(span, message));
            continue;
        };
        if let Some(first) = seen.insert(id.clone(), span.clone()) {
            let first = sources.locate(first);
            let message = format!("`{id}` is overridden twice, first at {first}");
            errors.push(sources.error(span, message));
            continue;
        }
        match base
            .iter_mut()
            .find(|existing| id_of(existing) == Some(id.as_str()))
        {
            Some(existing) => {
                let mut changes = entry.into_inner();
                if let DeValue::Table(changes) = &mut changes {
                    changes.remove(ID);
                }
                merge_value(existing, Spanned::new(span, changes), sources, errors);
            }
            None => base.push(entry),
        }
    }
}

/// Whether `entries` are named by their `id`: all tables with a string `id`, and at least one.
fn is_keyed(entries: &DeArray<'_>) -> bool {
    !entries.is_empty() && entries.iter().all(|entry| id_of(entry).is_some())
}

/// The `id` of an entry that is a table with a string `id`.
fn id_of<'v>(entry: &'v Spanned<DeValue<'_>>) -> Option<&'v str> {
    match entry.get_ref() {
        DeValue::Table(table) => table.get(ID).and_then(|id| id.get_ref().as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::content::testing::files;

    /// `over` laid onto `default`, as plain TOML, with the errors found.
    fn merged(default: &str, over: &str) -> (toml::Table, Vec<String>) {
        let files = files(&[
            ("content/a.toml", default),
            ("config/satiscraft/a.toml", over),
        ]);
        let sources = Sources::new(&files);
        let mut base = sources.parse(0).unwrap();
        let mut errors = Vec::new();
        merge_table(
            base.get_mut(),
            sources.parse(1).unwrap().into_inner(),
            &sources,
            &mut errors,
        );
        let table = toml::Table::deserialize(toml::de::Deserializer::from(base)).unwrap();
        (table, errors.iter().map(ToString::to_string).collect())
    }

    fn toml(text: &str) -> toml::Table {
        toml::from_str(text).unwrap()
    }

    #[test]
    fn tables_merge_key_by_key() {
        let (table, errors) = merged(
            "top = 1\n[sim]\nthreads = 0\nbudget = 20\n",
            "[sim]\nthreads = 4\nextra = true\n",
        );
        assert_eq!(
            table,
            toml("top = 1\n[sim]\nthreads = 4\nbudget = 20\nextra = true\n")
        );
        assert_eq!(errors, Vec::<String>::new());
    }

    #[test]
    fn entries_merge_by_id_and_new_ones_come_last() {
        let (table, errors) = merged(
            "[[items]]\nid = 'a'\nx = 1\ny = 2\n[[items]]\nid = 'b'\nx = 3\n",
            "[[items]]\nid = 'c'\n[[items]]\nid = 'a'\nx = 5\n",
        );
        let expected = "[[items]]\nid = 'a'\nx = 5\ny = 2\n[[items]]\nid = 'b'\nx = 3\n\
                        [[items]]\nid = 'c'\n";
        assert_eq!(table, toml(expected));
        assert_eq!(errors, Vec::<String>::new());
    }

    #[test]
    fn a_changed_entry_keeps_the_id_of_its_default() {
        let files = files(&[
            ("content/a.toml", "[[items]]\nid = 'a'\n"),
            (
                "config/satiscraft/a.toml",
                "[[items]]\nid = 'a'\n[[items]]\nid = 'b'\n",
            ),
        ]);
        let sources = Sources::new(&files);
        let mut base = sources.parse(0).unwrap();
        let over = sources.parse(1).unwrap().into_inner();
        merge_table(base.get_mut(), over, &sources, &mut Vec::new());
        let DeValue::Array(items) = base.get_ref()["items"].get_ref() else {
            panic!("an array");
        };
        let id_span = |index: usize| {
            let DeValue::Table(item) = items[index].get_ref() else {
                panic!("a table");
            };
            item["id"].span()
        };
        assert!(!sources.is_override(id_span(0)));
        assert!(sources.is_override(id_span(1)));
    }

    #[test]
    fn other_values_are_replaced_whole() {
        let (table, errors) = merged(
            "list = [1, 2]\nt = { a = 1 }\n[[plain]]\nx = 1\n[[plain]]\nx = 2\n",
            "list = [3]\nt = 5\n[[plain]]\nx = 9\n",
        );
        assert_eq!(table, toml("list = [3]\nt = 5\n[[plain]]\nx = 9\n"));
        assert_eq!(errors, Vec::<String>::new());
    }

    #[test]
    fn an_override_entry_without_id_is_refused() {
        let (table, errors) = merged(
            "[[items]]\nid = 'a'\n",
            "[[items]]\nid = 'a'\n[[items]]\nx = 1\n",
        );
        assert_eq!(table, toml("[[items]]\nid = 'a'\n"));
        assert_eq!(
            errors,
            [
                "config/satiscraft/a.toml:3: this entry has no `id`: an override names the entry \
              it changes by its `id`"
            ]
        );
    }

    #[test]
    fn an_entry_overridden_twice_is_refused() {
        let (table, errors) = merged(
            "[[items]]\nid = 'a'\nx = 1\n",
            "[[items]]\nid = 'a'\nx = 2\n\n[[items]]\nid = 'a'\nx = 3\n",
        );
        assert_eq!(table, toml("[[items]]\nid = 'a'\nx = 2\n"));
        assert_eq!(
            errors,
            [
                "config/satiscraft/a.toml:5: `a` is overridden twice, first at \
              config/satiscraft/a.toml:1"
            ]
        );
    }
}
