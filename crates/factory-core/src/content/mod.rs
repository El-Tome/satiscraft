//! The engine's content: everything a simulation runs on, read from TOML files.
//!
//! The platform hands over [`ContentFiles`]: the defaults embedded in the mod
//! (`content/items.toml`), then a server's overrides (`config/satiscraft/items.toml`). Files
//! with the same name are layers of one file, default first. An override only states what
//! it changes: tables merge key by key, `[[entries]]` by their `id`, and any other value
//! replaces the default. The core checks the result as a whole and refuses to run on any
//! problem, each reported at its `file:line`.
//!
//! Units: durations in ticks, energy in milli-units, chances as decimals read into exact
//! fractions ([`Chance`]).

mod chance;
mod config;
mod error;
mod items;
mod merge;
mod sources;
#[cfg(test)]
pub(crate) mod testing;
#[cfg(test)]
mod tests;

use serde::Deserialize;
use toml::Spanned;
use toml::de::DeTable;

pub use chance::Chance;
use config::ConfigFile;
pub use config::{Activity, ActivityPolicy, Config, Sim};
pub use error::{ContentError, ContentErrors};
use items::ItemsFile;
pub use items::{Item, Items, NAMESPACE};
use merge::merge_table;
use sources::Sources;

use crate::contract::ContentFiles;

/// The server settings.
const CONFIG: &str = "config.toml";
/// Items, resources, tools and ore blocks.
const ITEMS: &str = "items.toml";

/// Every file the engine reads, by name. Each comes with the issue that needs it.
pub const FILES: [&str; 2] = [CONFIG, ITEMS];

/// What a simulation runs on, checked as a whole. It belongs to the simulation, never to
/// global state, so that a reload can swap it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    config: Config,
    items: Items,
}

impl Content {
    /// Reads the engine's files, overrides merged into their defaults, and checks them. On
    /// any problem, returns them all, each at its `file:line`.
    pub fn load(files: &ContentFiles) -> Result<Self, ContentErrors> {
        let sources = Sources::new(files);
        let mut errors = unknown_files(&sources);
        let config = document(CONFIG, &sources, &mut errors)
            .and_then(|root| read::<ConfigFile>(root, &sources, &mut errors))
            .map(|file| file.validate(&sources, &mut errors));
        let items = document(ITEMS, &sources, &mut errors)
            .and_then(|root| read::<ItemsFile>(root, &sources, &mut errors))
            .map(|file| file.validate(&sources, &mut errors));
        match (config, items) {
            (Some(config), Some(items)) if errors.is_empty() => Ok(Self { config, items }),
            // Each `None` comes with its error.
            _ => Err(ContentErrors::new(errors)),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn items(&self) -> &Items {
        &self.items
    }
}

/// An error for each file the engine does not read: a misnamed one would go unnoticed.
fn unknown_files(sources: &Sources<'_>) -> Vec<ContentError> {
    sources
        .names()
        .filter(|(_, name)| !FILES.contains(name))
        .map(|(index, _)| {
            let message = format!(
                "unknown content file: the engine reads {}",
                FILES.join(", ")
            );
            sources.file_error(index, None, message)
        })
        .collect()
}

/// File `name`, its overrides merged into its default; `None` once an error is recorded.
fn document<'a>(
    name: &str,
    sources: &Sources<'a>,
    errors: &mut Vec<ContentError>,
) -> Option<Spanned<DeTable<'a>>> {
    let mut layers = Vec::new();
    let mut parsed = true;
    for index in sources.layers(name) {
        match sources.parse(index) {
            Ok(layer) => layers.push(layer),
            Err(error) => {
                errors.push(error);
                parsed = false;
            }
        }
    }
    // A broken layer would make the others look wrong: report it alone.
    if !parsed {
        return None;
    }
    let mut layers = layers.into_iter();
    let Some(mut root) = layers.next() else {
        errors.push(ContentError {
            path: name.to_owned(),
            line: None,
            message: "missing content file: the platform passes every default of the engine"
                .to_owned(),
        });
        return None;
    };
    for layer in layers {
        merge_table(root.get_mut(), layer.into_inner(), sources, errors);
    }
    Some(root)
}

/// Reads a merged file into its schema; `None` once an error is recorded.
fn read<'a, T: Deserialize<'a>>(
    root: Spanned<DeTable<'a>>,
    sources: &Sources<'a>,
    errors: &mut Vec<ContentError>,
) -> Option<T> {
    let whole = root.span();
    T::deserialize(toml::de::Deserializer::from(root))
        .map_err(|error| {
            let message = error.message();
            errors.push(match error.span() {
                Some(span) => sources.error(span, message),
                None => sources.file_error_at(whole, message),
            });
        })
        .ok()
}
