use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use smart_default::SmartDefault;
use stork_boundary::InternalWordAnnotation;
use stork_config::{OutputConfig, TitleBoost};
use stork_shared::Fields;

type EntryIndex = usize;
type AliasTarget = String;
type Score = u8;

mod build;
pub use build::build;
pub use build::errors::{DocumentError, IndexGenerationError};
pub use build::BuildResult;

mod read;

mod scores;
use scores::MATCHED_WORD_SCORE;

mod search;
pub use search::search;

mod write;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Index {
    config: PassthroughConfig,
    entries: Vec<Entry>,
    containers: HashMap<String, Container>,
}

impl Index {
    pub fn entries_len(&self) -> usize {
        self.entries.len()
    }

    pub fn word_count(&self) -> usize {
        self.entries.iter().map(|entry| entry.contents.len()).sum()
    }

    pub fn avg_entry_size(&self) -> usize {
        self.word_count() / self.entries_len()
    }

    pub fn search_term_count(&self) -> usize {
        self.containers.keys().count()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, SmartDefault)]
struct PassthroughConfig {
    url_prefix: String,
    title_boost: TitleBoost,

    #[default(OutputConfig::default().excerpt_buffer)]
    excerpt_buffer: u8,
    #[default(OutputConfig::default().excerpts_per_result)]
    excerpts_per_result: u8,
    #[default(OutputConfig::default().displayed_results_count)]
    displayed_results_count: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Entry {
    contents: String,
    title: String,
    url: String,
    fields: Fields,
}

/**
 * A Container holds:
 *
 * - a `HashMap` of `EntryIndexes` to `SearchResults`
 * - a `HashMap` of `AliasTargets` to scores
 *
 * Each valid query should return a single Container. It is possible to derive
 * all search results for a given query from a single container.
 */
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Container {
    // #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    results: HashMap<EntryIndex, SearchResult>,

    // #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    aliases: HashMap<AliasTarget, Score>,
}

impl Container {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SearchResult {
    excerpts: Vec<Excerpt>,
    score: Score,
}

impl SearchResult {
    fn new() -> SearchResult {
        SearchResult {
            excerpts: vec![],
            score: MATCHED_WORD_SCORE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Excerpt {
    word_index: usize,

    // #[serde(default, skip_serializing_if = "WordListSource::is_default")]
    source: WordListSource,

    // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    internal_annotations: Vec<InternalWordAnnotation>,

    // #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, SmartDefault)]
enum WordListSource {
    Title,

    #[default]
    Contents,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct AnnotatedWord {
    word: String,
    internal_annotations: Vec<InternalWordAnnotation>,
    fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnnotatedWordList {
    word_list: Vec<AnnotatedWord>,
}

impl AnnotatedWordList {
    fn get_full_text(&self) -> String {
        self.word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::fs;
    use std::io::{BufReader, Read};

    #[test]
    fn can_parse_0_7_0_index() {
        let file = fs::File::open("../test-assets/federalist-min-0.7.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
        let index = Index::try_from(index_bytes.as_slice()).unwrap();
        assert_eq!(1, index.entries.len());
        assert_eq!(2477, index.containers.len());
    }

    #[test]
    fn get_full_text() {
        let intended = "This is-a set of words.".to_string();
        let generated = AnnotatedWordList {
            word_list: vec![
                AnnotatedWord {
                    word: "This".to_string(),
                    ..Default::default()
                },
                AnnotatedWord {
                    word: "is-a".to_string(),
                    internal_annotations: vec![InternalWordAnnotation::UrlSuffix("a".to_string())],
                    fields: HashMap::default(),
                },
                AnnotatedWord {
                    word: "set".to_string(),
                    ..Default::default()
                },
                AnnotatedWord {
                    word: "of".to_string(),
                    ..Default::default()
                },
                AnnotatedWord {
                    word: "words.".to_string(),
                    ..Default::default()
                },
            ],
        }
        .get_full_text();

        assert_eq!(intended, generated);
    }
}
