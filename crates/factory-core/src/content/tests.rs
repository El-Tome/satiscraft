//! Loading whole sets of files: what ships, what overrides change, and every problem
//! reported at its `file:line`.

use std::num::NonZeroU32;

use super::testing::{files, shipped, shipped_files, with_overrides};
use super::*;
use crate::contract::ContentFile;

/// Every problem `files` has, as an operator reads them.
fn errors(files: &ContentFiles) -> Vec<String> {
    Content::load(files)
        .expect_err("invalid content")
        .errors()
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// The problems of the shipped files with `overrides` on top.
fn override_errors(overrides: &[(&str, &str)]) -> Vec<String> {
    errors(&with_overrides(overrides))
}

/// Shipped files with `items.toml` replaced by `items`.
fn with_items(items: &str) -> ContentFiles {
    let mut content = shipped_files();
    let file = content
        .files
        .iter_mut()
        .find(|file| file.path == "content/items.toml")
        .expect("items.toml ships");
    file.text = items.to_owned();
    content
}

#[test]
fn every_shipped_file_loads() {
    let names: Vec<_> = shipped_files()
        .files
        .into_iter()
        .map(|file| file.path)
        .collect();
    assert_eq!(names, FILES.map(|name| format!("content/{name}")));
    let content = shipped();
    let activity = &content.config().activity;
    assert!(activity.allowed.contains(&activity.default));
}

#[test]
fn an_override_changes_only_what_it_states() {
    let shipped = shipped();
    let content = Content::load(&with_overrides(&[("config.toml", "[sim]\nthreads = 4\n")]))
        .expect("a valid override");
    assert_eq!(content.config().sim.threads, 4);
    assert_eq!(
        content.config().sim.tick_budget_ms,
        shipped.config().sim.tick_budget_ms
    );
    assert_eq!(content.config().activity, shipped.config().activity);
    assert_eq!(content.items(), shipped.items());
}

#[test]
fn overrides_apply_in_order() {
    let content = Content::load(&with_overrides(&[
        ("config.toml", "[sim]\nthreads = 4\ntick_budget_ms = 30\n"),
        ("config.toml", "[sim]\nthreads = 8\n"),
    ]))
    .expect("valid overrides");
    assert_eq!(content.config().sim.threads, 8);
    assert_eq!(
        content.config().sim.tick_budget_ms,
        NonZeroU32::new(30).unwrap()
    );
}

#[test]
fn items_load_with_their_ids() {
    let content = Content::load(&with_items(
        "[[items]]\nid = 'satiscraft:rope'\n[[items]]\nid = 'satiscraft:flint_blade'\n",
    ))
    .expect("valid items");
    let ids: Vec<_> = content
        .items()
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    assert_eq!(ids, ["satiscraft:flint_blade", "satiscraft:rope"]);
}

#[test]
fn a_syntax_error_is_reported() {
    let found = override_errors(&[("config.toml", "[sim]\nthreads = = 4\n")]);
    assert_eq!(found.len(), 1);
    assert!(
        found[0].starts_with("config/satiscraft/config.toml:2: "),
        "{found:?}"
    );
}

#[test]
fn an_unknown_file_is_reported() {
    assert_eq!(
        override_errors(&[("itemz.toml", "")]),
        [
            "config/satiscraft/itemz.toml: unknown content file: the engine reads config.toml, \
          items.toml"
        ]
    );
}

#[test]
fn a_missing_file_is_reported() {
    let only_config = files(&[("content/config.toml", "")]);
    let found = errors(&only_config);
    assert!(
        found.contains(
            &"items.toml: missing content file: the platform passes every default of the \
              engine"
                .to_owned()
        ),
        "{found:?}"
    );
}

#[test]
fn an_unknown_key_is_reported() {
    assert_eq!(
        override_errors(&[("config.toml", "[sim]\nthread = 4\n")]),
        [
            "config/satiscraft/config.toml:2: unknown field `thread`, expected `threads` or \
          `tick_budget_ms`"
        ]
    );
}

#[test]
fn a_value_of_the_wrong_type_is_reported() {
    assert_eq!(
        override_errors(&[("config.toml", "[sim]\nthreads = '4'\n")]),
        ["config/satiscraft/config.toml:2: invalid type: string \"4\", expected u16"]
    );
}

#[test]
fn a_value_out_of_range_is_reported() {
    assert_eq!(
        override_errors(&[("config.toml", "\n[sim]\ntick_budget_ms = 0\n")]),
        ["config/satiscraft/config.toml:3: invalid value: integer `0`, expected a nonzero u32"]
    );
}

#[test]
fn a_missing_key_is_reported() {
    let mut content = shipped_files();
    content.files[0].text = "[sim]\nthreads = 0\ntick_budget_ms = 20\n".to_owned();
    assert_eq!(
        errors(&content),
        ["content/config.toml:1: missing field `activity`"]
    );
}

#[test]
fn an_unknown_policy_is_reported() {
    assert_eq!(
        override_errors(&[("config.toml", "[activity]\ndefault = 'sometimes'\n")]),
        [
            "config/satiscraft/config.toml:2: unknown variant `sometimes`, expected one of \
          `always`, `team_online`, `paused`"
        ]
    );
}

#[test]
fn a_default_policy_that_is_not_allowed_is_reported() {
    assert_eq!(
        override_errors(&[(
            "config.toml",
            "[activity]\nallowed = ['team_online', 'paused']\ndefault = 'always'\n"
        )]),
        [
            "config/satiscraft/config.toml:3: the default activity policy `always` is not among the \
          allowed ones [team_online, paused]"
        ]
    );
}

#[test]
fn an_invalid_item_id_is_reported() {
    assert_eq!(
        errors(&with_items("[[items]]\nid = 'Flint'\n")),
        ["content/items.toml:2: identifier `Flint` has no namespace (expected `namespace:path`)"]
    );
}

#[test]
fn an_item_outside_the_engine_namespace_is_reported() {
    assert_eq!(
        errors(&with_items("[[items]]\nid = 'minecraft:flint'\n")),
        [
            "content/items.toml:2: item `minecraft:flint` is outside the `satiscraft` namespace: \
          the engine declares only its own items"
        ]
    );
}

#[test]
fn a_duplicate_item_is_reported() {
    let items = "[[items]]\nid = 'satiscraft:rope'\n\n[[items]]\nid = 'satiscraft:rope'\n";
    assert_eq!(
        errors(&with_items(items)),
        [
            "content/items.toml:5: item `satiscraft:rope` is declared twice, first at \
          content/items.toml:2"
        ]
    );
}

#[test]
fn an_item_added_by_an_override_is_reported() {
    let content = with_items("[[items]]\nid = 'satiscraft:rope'\n");
    let mut overridden = content.clone();
    overridden.files.push(ContentFile {
        path: "config/satiscraft/items.toml".into(),
        text: "[[items]]\nid = 'satiscraft:rope'\n[[items]]\nid = 'satiscraft:rpoe'\n".into(),
    });
    assert_eq!(
        errors(&overridden),
        [
            "config/satiscraft/items.toml:4: unknown item `satiscraft:rpoe`: an override changes \
          the engine's items, it cannot add one"
        ]
    );
    // Shipped without items, any item in an override is new.
    assert_eq!(
        override_errors(&[("items.toml", "[[items]]\nid = 'satiscraft:rope'\n")]),
        [
            "config/satiscraft/items.toml:2: unknown item `satiscraft:rope`: an override changes \
          the engine's items, it cannot add one"
        ]
    );
}

#[test]
fn problems_in_several_files_are_reported_together() {
    let found = override_errors(&[
        ("config.toml", "[sim]\nthread = 4\n"),
        ("items.toml", "[[items]]\nid = 'satiscraft:rope'\n"),
        ("quests.toml", ""),
    ]);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[0].starts_with("config/satiscraft/quests.toml: "));
    assert!(found[1].starts_with("config/satiscraft/config.toml:2: "));
    assert!(found[2].starts_with("config/satiscraft/items.toml:2: "));
}
