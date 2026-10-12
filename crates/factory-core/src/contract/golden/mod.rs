//! Golden files of the contract: sample values with their postcard bytes, written here and
//! read back by the Kotlin tests, so that both sides provably speak the same bytes.
//!
//! After a contract change, regenerate them with
//! `UPDATE_GOLDEN=1 cargo test -p factory-core golden`, then mirror the change in Kotlin
//! (`core.contract`, and `ContractSamples` in its tests).

mod samples;
mod variants;

use std::collections::BTreeSet;
use std::fmt::{Debug, Write};
use std::path::PathBuf;
use std::{env, fs};

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{decode, encode, reencode};
use samples::{
    command_results, commands, content_file, content_files, dimension_ids, effects, item_ids,
    machine_ids, player_ids, positions, queries, query_results, rejections, team_ids,
};
use variants::variants;

const REGENERATE: &str = "UPDATE_GOLDEN=1 cargo test -p factory-core golden";

/// The folder of the golden files, read by the Kotlin tests too.
fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("golden")
}

/// One golden file: the samples of a contract type, as postcard bytes and as Rust values.
struct Golden {
    /// The file stem, and the type name [`reencode`] takes.
    name: &'static str,
    samples: Vec<(Vec<u8>, String)>,
}

impl Golden {
    /// The samples of `T`, each checked to decode back to itself.
    fn of<T>(name: &'static str, values: Vec<T>) -> Self
    where
        T: Serialize + DeserializeOwned + Debug + PartialEq,
    {
        let samples = values
            .iter()
            .map(|value| {
                let bytes = encode(value).unwrap();
                assert_eq!(&decode::<T>(&bytes).unwrap(), value, "{name}");
                (bytes, format!("{value:?}"))
            })
            .collect();
        Self { name, samples }
    }

    fn file_name(&self) -> String {
        format!("{}.txt", self.name)
    }

    /// A header, then one line per sample: its bytes in hex, a tab, its Rust value.
    fn render(&self) -> String {
        let mut text = format!(
            "# {}: postcard bytes in hex, a tab, then the Rust value.\n# Generated: {REGENERATE}\n",
            self.name
        );
        for (bytes, value) in &self.samples {
            let hex: Vec<String> = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
            writeln!(text, "{}\t{value}", hex.join(" ")).unwrap();
        }
        text
    }
}

fn goldens() -> Vec<Golden> {
    vec![
        Golden::of("command", commands()),
        Golden::of("command_batch", vec![vec![], commands()]),
        Golden::of("command_result", command_results()),
        Golden::of("command_result_batch", vec![vec![], command_results()]),
        Golden::of("rejection", rejections()),
        Golden::of("effect", effects()),
        Golden::of("effect_batch", vec![vec![], effects()]),
        Golden::of("query", queries()),
        Golden::of("query_result", query_results()),
        Golden::of("content_file", content_file()),
        Golden::of("content_files", content_files()),
        Golden::of("item_id", item_ids()),
        Golden::of("dimension_id", dimension_ids()),
        Golden::of("position", positions()),
        Golden::of("player_id", player_ids()),
        Golden::of("team_id", team_ids()),
        Golden::of("machine_id", machine_ids()),
    ]
}

/// The names of the `.txt` files in the golden folder.
fn files_on_disk() -> BTreeSet<String> {
    let Ok(entries) = fs::read_dir(golden_dir()) else {
        return BTreeSet::new();
    };
    entries
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".txt"))
        .collect()
}

/// Rewrites every golden file and removes those of types that no longer exist.
fn regenerate(goldens: &[Golden]) {
    fs::create_dir_all(golden_dir()).unwrap();
    let expected: BTreeSet<String> = goldens.iter().map(Golden::file_name).collect();
    for stale in files_on_disk().difference(&expected) {
        fs::remove_file(golden_dir().join(stale)).unwrap();
    }
    for golden in goldens {
        fs::write(golden_dir().join(golden.file_name()), golden.render()).unwrap();
    }
}

/// Fails unless the samples of the enum `T` hold each of its variants.
fn assert_covers_every_variant<T: Serialize + DeserializeOwned>(samples: &[T]) {
    let count = variants::<T>().len();
    assert!(count <= 128, "past 128 variants, an index takes two bytes");
    let covered: BTreeSet<usize> = samples
        .iter()
        .map(|sample| usize::from(encode(sample).unwrap()[0]))
        .collect();
    assert_eq!(
        covered,
        (0..count).collect(),
        "{}: give each variant a golden sample, in samples.rs and in Kotlin's ContractSamples",
        std::any::type_name::<T>()
    );
}

#[test]
fn golden_files_match_the_contract() {
    let goldens = goldens();
    if env::var_os("UPDATE_GOLDEN").is_some() {
        regenerate(&goldens);
    }
    for golden in &goldens {
        let path = golden_dir().join(golden.file_name());
        let on_disk = fs::read_to_string(&path).unwrap_or_default();
        assert_eq!(
            on_disk,
            golden.render(),
            "{} is out of date: run {REGENERATE}, then update Kotlin",
            path.display()
        );
    }
    let names: BTreeSet<String> = goldens.iter().map(Golden::file_name).collect();
    assert_eq!(
        files_on_disk(),
        names,
        "stale golden files: run {REGENERATE}"
    );
}

#[test]
fn samples_cover_every_variant() {
    assert_covers_every_variant(&commands());
    assert_covers_every_variant(&command_results());
    assert_covers_every_variant(&rejections());
    assert_covers_every_variant(&effects());
    assert_covers_every_variant(&queries());
    assert_covers_every_variant(&query_results());
}

#[test]
fn reencode_knows_every_golden_type() {
    for golden in goldens() {
        for (bytes, value) in &golden.samples {
            assert_eq!(
                reencode(golden.name, bytes).as_ref(),
                Ok(bytes),
                "{}: {value}",
                golden.name
            );
        }
    }
}

#[test]
fn render_writes_one_hex_line_per_sample() {
    let text = Golden::of("team_id", team_ids()).render();
    let lines: Vec<&str> = text.lines().skip(2).collect();
    assert_eq!(
        lines,
        [
            "00\tTeamId(0)",
            "ac 02\tTeamId(300)",
            "ff ff ff ff ff ff ff ff ff 01\tTeamId(18446744073709551615)"
        ]
    );
}
