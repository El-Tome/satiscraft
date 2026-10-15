//! The text files a simulation is built from, as the platform hands them over.

use serde::{Deserialize, Serialize};

/// The engine's content and the server config, each file with its path, in one value: errors
/// can cite `file:line`, and a new file does not change how a platform calls the core.
///
/// The platform gathers the embedded defaults, then the server overrides; the core reads them
/// (see [`crate::content`]). Files with the same name, the part of the path after the last
/// `/`, are layers of one file: the first is the default, each later one overrides it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentFiles {
    pub files: Vec<ContentFile>,
}

/// One text file, under the path that errors cite: `content/items.toml` for a default,
/// `config/satiscraft/items.toml` for a server's override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentFile {
    pub path: String,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::contract::arbitrary::content_files;
    use crate::contract::{decode, encode};

    #[test]
    fn files_are_encoded_in_order_with_their_path_first() {
        // Kotlin writes these bytes: a file count, then each path and text, length first.
        assert_eq!(encode(&ContentFiles::default()).unwrap(), [0]);
        let content = ContentFiles {
            files: vec![ContentFile {
                path: "a.toml".into(),
                text: "x=1".into(),
            }],
        };
        assert_eq!(
            encode(&content).unwrap(),
            [
                1, 6, b'a', b'.', b't', b'o', b'm', b'l', 3, b'x', b'=', b'1'
            ]
        );
    }

    proptest! {
        #[test]
        fn generated_content_round_trips(content in content_files()) {
            prop_assert_eq!(decode::<ContentFiles>(&encode(&content).unwrap()), Ok(content));
        }
    }
}
