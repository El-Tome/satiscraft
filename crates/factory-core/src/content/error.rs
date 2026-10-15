//! What is wrong with the content, each problem at the `file:line` that holds it.

use std::fmt;

/// One problem in the content, where an operator can fix it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentError {
    /// The file, as the platform named it: `content/items.toml`, `config/satiscraft/items.toml`.
    pub path: String,
    /// Line in that file, from 1; `None` when the problem concerns the whole file.
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "{}:{line}: {}", self.path, self.message),
            None => write!(f, "{}: {}", self.path, self.message),
        }
    }
}

impl std::error::Error for ContentError {}

/// Everything found wrong in a set of files: the core refuses to run on them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentErrors(Vec<ContentError>);

impl ContentErrors {
    /// Wraps the problems found, in the order they were found.
    ///
    /// # Panics
    ///
    /// If `errors` is empty: content without problems is not an error.
    pub(crate) fn new(errors: Vec<ContentError>) -> Self {
        assert!(!errors.is_empty(), "content errors without any error");
        Self(errors)
    }

    /// Each problem, in the order found.
    pub fn errors(&self) -> &[ContentError] {
        &self.0
    }
}

impl fmt::Display for ContentErrors {
    /// One line per problem under a count, so that an operator reads them all at once.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.0.len();
        let plural = if count == 1 { "" } else { "s" };
        write!(f, "invalid content ({count} error{plural}):")?;
        for error in &self.0 {
            write!(f, "\n  {error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ContentErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(line: Option<usize>, message: &str) -> ContentError {
        ContentError {
            path: "config/satiscraft/items.toml".into(),
            line,
            message: message.into(),
        }
    }

    #[test]
    fn an_error_cites_its_file_and_line() {
        assert_eq!(
            error(Some(3), "missing field `id`").to_string(),
            "config/satiscraft/items.toml:3: missing field `id`"
        );
        assert_eq!(
            error(None, "unknown content file").to_string(),
            "config/satiscraft/items.toml: unknown content file"
        );
    }

    #[test]
    fn errors_are_listed_one_per_line_under_their_count() {
        let one = ContentErrors::new(vec![error(Some(1), "a")]);
        assert_eq!(
            one.to_string(),
            "invalid content (1 error):\n  config/satiscraft/items.toml:1: a"
        );
        let two = ContentErrors::new(vec![error(Some(1), "a"), error(None, "b")]);
        assert_eq!(two.errors().len(), 2);
        assert_eq!(
            two.to_string(),
            "invalid content (2 errors):\n  config/satiscraft/items.toml:1: a\n  \
             config/satiscraft/items.toml: b"
        );
    }

    #[test]
    #[should_panic(expected = "content errors without any error")]
    fn errors_are_never_empty() {
        ContentErrors::new(Vec::new());
    }
}
