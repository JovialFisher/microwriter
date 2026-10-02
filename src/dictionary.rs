//! Completion candidates, ranked by a weighted prefix trie.
//!
//! The search itself is delegated to the [`autocomplete`] crate: a trie built
//! from `(word, weight)` pairs whose `words(prefix)` returns every match, best
//! weight first. Every lexicon the writer can draw on is folded into one trie —
//! the bundled vocabulary, words from other recent notes, and the words already
//! in the document — so a single lookup ranks them all together.
//!
//! The bundled list is deliberately small and ordered roughly by frequency, so
//! a short prefix completes to the most ordinary word rather than a random one.
//! It ships inside the binary — nothing is downloaded and no file has to be
//! found at runtime.

use crate::editor::clean_word;
use autocomplete::Dictionary;
use std::collections::HashMap;

/// Raw list, whitespace-separated so the file stays readable in columns.
const COMMON_WORDS: &str = include_str!("common_words.txt");

/// A word from the document always outranks one the writer has never used.
pub const DOCUMENT_WEIGHT: i64 = 10_000_000;
/// Words gathered from other recent notes sit between the document and the
/// bundled vocabulary.
pub const RECENT_WEIGHT: i64 = 1_000_000;

/// Iterate the bundled list, most common first.
pub fn common_words() -> impl Iterator<Item = &'static str> {
    COMMON_WORDS.split_whitespace()
}

/// A weighted prefix trie over every word available to completion.
///
/// Words are keyed in lowercase so a lookup is case-insensitive, while the
/// original spelling is remembered so `API` and `London` come back intact.
pub struct Completer {
    trie: Dictionary<i64>,
    /// Lowercase key -> the spelling to hand back.
    spelling: HashMap<String, String>,
    /// The weight currently held for each key, so a lower-ranked source can
    /// never overwrite a higher-ranked one.
    weights: HashMap<String, i64>,
}

impl Default for Completer {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer {
    /// A completer seeded with the bundled vocabulary, its most common word
    /// weighted highest.
    pub fn new() -> Self {
        let mut completer = Self {
            trie: Dictionary::new(),
            spelling: HashMap::new(),
            weights: HashMap::new(),
        };
        let words: Vec<&str> = common_words().collect();
        let total = words.len() as i64;
        for (rank, word) in words.into_iter().enumerate() {
            completer.add(word, total - rank as i64);
        }
        completer
    }

    /// Offer `word` as a candidate at `weight`. A word already held at an equal
    /// or higher weight is left alone, so adding the same word from several
    /// sources keeps the strongest source's spelling and rank.
    pub fn add(&mut self, word: &str, weight: i64) {
        let cleaned = clean_word(word);
        if cleaned.chars().count() < 2 {
            return;
        }
        let key = cleaned.to_lowercase();
        if self.weights.get(&key).is_some_and(|held| *held >= weight) {
            return;
        }
        self.weights.insert(key.clone(), weight);
        self.spelling.insert(key.clone(), cleaned.to_string());
        self.trie.insert(key, weight);
    }

    /// Candidate completions for `prefix`, best first, with the prefix's
    /// capitalisation carried onto each suggestion. The exact prefix is never
    /// suggested back.
    pub fn complete(&self, prefix: &str, limit: usize) -> Vec<String> {
        let needle = prefix.to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        self.trie
            .words(&needle)
            .into_iter()
            .map(|(word, _)| word)
            .filter(|word| word != &needle)
            .take(limit)
            .map(|word| {
                let spelled = self.spelling.get(&word).cloned().unwrap_or(word);
                match_case(&spelled, prefix)
            })
            .collect()
    }
}

/// Give a suggestion the same leading capital as the word the writer typed, so
/// `Rec` completes to `Receive` rather than `receive`.
fn match_case(word: &str, prefix: &str) -> String {
    let uppercase = prefix
        .chars()
        .find(|c| c.is_alphabetic())
        .is_some_and(char::is_uppercase);
    if !uppercase {
        return word.to_string();
    }
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_holds_a_usable_vocabulary() {
        let words: Vec<&str> = common_words().collect();
        assert!(words.len() >= 500, "only {} words bundled", words.len());
        assert!(words.iter().all(|w| w.chars().all(char::is_alphabetic)));
        assert!(words.contains(&"the"));
        assert!(words.contains(&"receive"));
    }

    #[test]
    fn a_prefix_runs_down_the_bundled_vocabulary() {
        let completer = Completer::new();
        let matches = completer.complete("rec", 10);
        assert!(!matches.is_empty());
        // Every candidate keeps the prefix, since the trie is the whole list.
        assert!(matches.iter().all(|w| w.to_lowercase().starts_with("rec")));
        assert!(matches.contains(&"receive".to_string()));
    }

    #[test]
    fn a_document_word_beats_the_bundled_list() {
        let mut completer = Completer::new();
        completer.add("serendipity", DOCUMENT_WEIGHT);
        assert_eq!(completer.complete("seren", 5)[0], "serendipity");
    }

    #[test]
    fn a_higher_weight_survives_a_later_lower_one() {
        let mut completer = Completer::new();
        completer.add("widget", DOCUMENT_WEIGHT);
        completer.add("widget", RECENT_WEIGHT);
        assert_eq!(completer.complete("wid", 5)[0], "widget");
    }

    #[test]
    fn candidates_carry_the_prefix_capitalisation() {
        let completer = Completer::new();
        let matches = completer.complete("Ther", 5);
        assert!(!matches.is_empty());
        assert!(matches.iter().all(|w| w.starts_with("Ther")));
    }

    #[test]
    fn the_original_spelling_is_kept() {
        let mut completer = Completer::new();
        completer.add("Kubernetes", DOCUMENT_WEIGHT);
        let matches = completer.complete("kub", 5);
        assert_eq!(matches[0], "Kubernetes");
    }

    #[test]
    fn the_exact_prefix_is_not_suggested_back() {
        let mut completer = Completer::new();
        completer.add("serenade", DOCUMENT_WEIGHT);
        assert!(!completer
            .complete("serenade", 5)
            .contains(&"serenade".to_string()));
    }
}
