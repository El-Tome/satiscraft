//! The files content is read from, laid end to end in one virtual text: a span found after
//! merging defaults and overrides still tells which file, and which line, it comes from.

use std::ops::Range;

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use super::error::ContentError;
use crate::contract::ContentFiles;

/// One file, placed in the virtual text.
#[derive(Debug)]
struct Source<'a> {
    path: &'a str,
    text: &'a str,
    /// Offset of the file's first byte in the virtual text.
    start: usize,
    /// 0 for the first file of its name, the default; 1, 2... for the overrides that follow.
    layer: usize,
}

/// The files of a [`ContentFiles`], in order, and how to turn a span back into `file:line`.
#[derive(Debug)]
pub(crate) struct Sources<'a> {
    sources: Vec<Source<'a>>,
}

impl<'a> Sources<'a> {
    pub(crate) fn new(files: &'a ContentFiles) -> Self {
        let mut sources: Vec<Source<'a>> = Vec::with_capacity(files.files.len());
        let mut start = 0;
        for file in &files.files {
            let layer = sources
                .iter()
                .filter(|earlier| name(earlier.path) == name(&file.path))
                .count();
            sources.push(Source {
                path: &file.path,
                text: &file.text,
                start,
                layer,
            });
            // The offset just past the end still belongs to the file: errors at its very end.
            start += file.text.len() + 1;
        }
        Self { sources }
    }

    /// Each file's index and name, in order.
    pub(crate) fn names(&self) -> impl Iterator<Item = (usize, &'a str)> + '_ {
        self.sources
            .iter()
            .enumerate()
            .map(|(index, source)| (index, name(source.path)))
    }

    /// Indices of the files called `name`: its default first, then its overrides in order.
    pub(crate) fn layers(&self, name: &str) -> Vec<usize> {
        self.names()
            .filter(|&(_, other)| other == name)
            .map(|(index, _)| index)
            .collect()
    }

    /// Parses file `index`, with spans in the virtual text.
    pub(crate) fn parse(&self, index: usize) -> Result<Spanned<DeTable<'a>>, ContentError> {
        let source = &self.sources[index];
        match DeTable::parse(source.text) {
            Ok(root) => {
                let span = moved(root.span(), source.start);
                Ok(Spanned::new(
                    span,
                    shift_table(root.into_inner(), source.start),
                ))
            }
            Err(err) => {
                Err(self.file_error(index, err.span().map(|span| span.start), err.message()))
            }
        }
    }

    /// An error at `span` of the virtual text.
    pub(crate) fn error(&self, span: Range<usize>, message: impl Into<String>) -> ContentError {
        let index = self.file_at(span.start);
        let offset = span.start - self.sources[index].start;
        self.file_error(index, Some(offset), message)
    }

    /// An error in file `index`: at `offset` of its text, or on the whole file.
    pub(crate) fn file_error(
        &self,
        index: usize,
        offset: Option<usize>,
        message: impl Into<String>,
    ) -> ContentError {
        let source = &self.sources[index];
        ContentError {
            path: source.path.to_owned(),
            line: offset.map(|offset| line_at(source.text, offset)),
            message: message.into(),
        }
    }

    /// An error on the whole file that holds `span`.
    pub(crate) fn file_error_at(
        &self,
        span: Range<usize>,
        message: impl Into<String>,
    ) -> ContentError {
        self.file_error(self.file_at(span.start), None, message)
    }

    /// `path:line` of `span`, for messages that point to a second place.
    pub(crate) fn locate(&self, span: Range<usize>) -> String {
        let error = self.error(span, "");
        match error.line {
            Some(line) => format!("{}:{line}", error.path),
            None => error.path,
        }
    }

    /// Whether `span` lies in an override rather than in a default.
    pub(crate) fn is_override(&self, span: Range<usize>) -> bool {
        self.sources[self.file_at(span.start)].layer > 0
    }

    /// Index of the file holding `offset` of the virtual text.
    fn file_at(&self, offset: usize) -> usize {
        self.sources
            .partition_point(|source| source.start <= offset)
            .saturating_sub(1)
    }
}

/// What a file is known as, whatever folder it comes from: the part of its path after the
/// last `/` (`config/satiscraft/items.toml` -> `items.toml`).
pub(crate) fn name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Line of `offset` in `text`, counted from 1.
fn line_at(text: &str, offset: usize) -> usize {
    let before = &text.as_bytes()[..offset.min(text.len())];
    before.iter().filter(|&&byte| byte == b'\n').count() + 1
}

fn moved(span: Range<usize>, by: usize) -> Range<usize> {
    span.start + by..span.end + by
}

/// `table` with every span, keys included, moved `by` bytes.
fn shift_table(table: DeTable<'_>, by: usize) -> DeTable<'_> {
    table
        .into_iter()
        .map(|(key, value)| {
            let span = moved(key.span(), by);
            (Spanned::new(span, key.into_inner()), shift_value(value, by))
        })
        .collect()
}

/// `value` with every span moved `by` bytes.
fn shift_value(value: Spanned<DeValue<'_>>, by: usize) -> Spanned<DeValue<'_>> {
    let span = moved(value.span(), by);
    let value = match value.into_inner() {
        DeValue::Table(table) => DeValue::Table(shift_table(table, by)),
        // In place: the array remembers whether it was written as `[[tables]]`.
        DeValue::Array(mut array) => {
            for item in array.iter_mut() {
                let placeholder = Spanned::new(0..0, DeValue::Boolean(false));
                *item = shift_value(std::mem::replace(item, placeholder), by);
            }
            DeValue::Array(array)
        }
        other => other,
    };
    Spanned::new(span, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::testing::files;

    /// Span of `key`'s value in the parsed `table`.
    fn value_span(table: &Spanned<DeTable<'_>>, key: &str) -> Range<usize> {
        table.get_ref().get(key).expect("a key of the table").span()
    }

    #[test]
    fn a_file_is_known_by_its_last_path_segment() {
        assert_eq!(name("config/satiscraft/items.toml"), "items.toml");
        assert_eq!(name("content/items.toml"), "items.toml");
        assert_eq!(name("items.toml"), "items.toml");
    }

    #[test]
    fn lines_count_from_one() {
        let text = "a = 1\nb = 2\n";
        assert_eq!(line_at(text, 0), 1);
        assert_eq!(line_at(text, 5), 1);
        assert_eq!(line_at(text, 6), 2);
        assert_eq!(line_at(text, text.len()), 3);
        assert_eq!(line_at(text, 99), 3);
    }

    #[test]
    fn layers_are_files_of_the_same_name_in_order() {
        let files = files(&[
            ("content/config.toml", ""),
            ("content/items.toml", ""),
            ("config/satiscraft/items.toml", ""),
        ]);
        let sources = Sources::new(&files);
        assert_eq!(sources.layers("items.toml"), [1, 2]);
        assert_eq!(sources.layers("config.toml"), [0]);
        assert_eq!(sources.layers("quests.toml"), Vec::<usize>::new());
        let names: Vec<_> = sources.names().collect();
        assert_eq!(
            names,
            [(0, "config.toml"), (1, "items.toml"), (2, "items.toml")]
        );
    }

    #[test]
    fn spans_point_back_to_their_file_and_line() {
        let files = files(&[
            ("content/items.toml", "a = 1\n"),
            ("config/satiscraft/items.toml", "\n\nb = 2\n"),
        ]);
        let sources = Sources::new(&files);
        let (default, over) = (sources.parse(0).unwrap(), sources.parse(1).unwrap());
        let a = value_span(&default, "a");
        let b = value_span(&over, "b");
        assert_eq!(
            sources.error(a.clone(), "x").to_string(),
            "content/items.toml:1: x"
        );
        assert_eq!(
            sources.error(b.clone(), "y").to_string(),
            "config/satiscraft/items.toml:3: y"
        );
        assert_eq!(sources.locate(b.clone()), "config/satiscraft/items.toml:3");
        assert!(!sources.is_override(a));
        assert!(sources.is_override(b));
    }

    #[test]
    fn spans_inside_arrays_and_nested_tables_move_too() {
        let files = files(&[
            ("content/a.toml", "x = 0"),
            ("content/b.toml", "[t]\nlist = [1, { k = 2 }]\n"),
        ]);
        let sources = Sources::new(&files);
        let root = sources.parse(1).unwrap();
        let DeValue::Table(table) = root.get_ref()["t"].get_ref() else {
            panic!("a table");
        };
        let DeValue::Array(list) = table["list"].get_ref() else {
            panic!("an array");
        };
        let DeValue::Table(inline) = list[1].get_ref() else {
            panic!("an inline table");
        };
        for span in [list[0].span(), inline["k"].span()] {
            assert_eq!(sources.locate(span), "content/b.toml:2");
        }
    }

    #[test]
    fn the_end_of_a_file_belongs_to_it() {
        let files = files(&[("content/a.toml", "x = 1"), ("content/b.toml", "")]);
        let sources = Sources::new(&files);
        assert_eq!(sources.locate(5..5), "content/a.toml:1");
        assert_eq!(sources.locate(6..6), "content/b.toml:1");
    }

    #[test]
    fn a_syntax_error_cites_its_file_and_line() {
        let files = files(&[("content/a.toml", "x = 1\ny = \n")]);
        let error = Sources::new(&files).parse(0).unwrap_err();
        assert_eq!(
            (error.path.as_str(), error.line),
            ("content/a.toml", Some(2))
        );
        assert!(!error.message.is_empty());
    }

    #[test]
    fn a_whole_file_error_has_no_line() {
        let files = files(&[("content/a.toml", "x = 1"), ("content/b.toml", "")]);
        let sources = Sources::new(&files);
        let error = sources.file_error(0, None, "unknown content file");
        assert_eq!(error.to_string(), "content/a.toml: unknown content file");
        let error = sources.file_error_at(6..6, "no field");
        assert_eq!(error.to_string(), "content/b.toml: no field");
    }
}
