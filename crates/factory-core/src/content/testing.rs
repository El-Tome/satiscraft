//! Content for tests: the files shipped with the core, and files written on the spot.

use std::fs;
use std::path::Path;

use super::Content;
use crate::contract::{ContentFile, ContentFiles};

/// Files from `(path, text)` pairs, in order.
pub(crate) fn files(files: &[(&str, &str)]) -> ContentFiles {
    ContentFiles {
        files: files
            .iter()
            .map(|&(path, text)| ContentFile {
                path: path.into(),
                text: text.into(),
            })
            .collect(),
    }
}

/// The `.toml` files of `crates/factory-core/content/`, by name, as the platform passes the
/// defaults: `content/<name>`.
pub(crate) fn shipped_files() -> ContentFiles {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("content");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .expect("the content folder")
        .map(|entry| entry.expect("a content file").file_name())
        .filter_map(|name| name.into_string().ok())
        .filter(|name| name.ends_with(".toml"))
        .collect();
    names.sort();
    ContentFiles {
        files: names
            .into_iter()
            .map(|name| ContentFile {
                text: fs::read_to_string(dir.join(&name)).expect("a readable content file"),
                path: format!("content/{name}"),
            })
            .collect(),
    }
}

/// The shipped files, then `overrides` as a server's `config/satiscraft/<name>`.
pub(crate) fn with_overrides(overrides: &[(&str, &str)]) -> ContentFiles {
    let mut content = shipped_files();
    content
        .files
        .extend(overrides.iter().map(|&(name, text)| ContentFile {
            path: format!("config/satiscraft/{name}"),
            text: text.into(),
        }));
    content
}

/// The content shipped with the core.
pub(crate) fn shipped() -> Content {
    Content::load(&shipped_files()).expect("the shipped content loads")
}
