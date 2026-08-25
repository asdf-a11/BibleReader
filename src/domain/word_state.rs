use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::WordStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordStateStore {
    states: HashMap<String, WordStatus>,
}

impl WordStateStore {
    pub fn new() -> Self {
        WordStateStore {
            states: HashMap::new(),
        }
    }

    pub fn get_status(&self, word: &str) -> WordStatus {
        *self.states.get(word.to_lowercase().as_str()).unwrap_or(&WordStatus::Unseen)
    }

    pub fn set_status(&mut self, word: &str, status: WordStatus) {
        self.states.insert(word.to_lowercase(), status);
    }

    pub fn access_word(&mut self, word: &str) -> WordStatus {
        let key = word.to_lowercase();
        let current = *self.states.get(&key).unwrap_or(&WordStatus::Unseen);
        
        let new_status = if current == WordStatus::Unseen {
            WordStatus::Seen
        } else {
            current
        };
        
        self.states.insert(key, new_status);
        new_status
    }

    pub fn get_all_states(&self) -> Vec<(&String, &WordStatus)> {
        self.states.iter().collect()
    }

    pub fn is_word_known(&self, word: &str) -> bool {
        matches!(self.get_status(word), WordStatus::Known)
    }

    pub fn is_word_seen(&self, word: &str) -> bool {
        matches!(self.get_status(word), WordStatus::Seen | WordStatus::Known)
    }
}

impl Default for WordStateStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_status_is_unseen() {
        let store = WordStateStore::new();
        assert_eq!(store.get_status("привет"), WordStatus::Unseen);
    }

    #[test]
    fn test_set_status() {
        let mut store = WordStateStore::new();
        store.set_status("привет", WordStatus::Known);
        assert_eq!(store.get_status("привет"), WordStatus::Known);
    }

    #[test]
    fn test_access_word_transitions_unseen_to_seen() {
        let mut store = WordStateStore::new();
        let result = store.access_word("привет");
        assert_eq!(result, WordStatus::Seen);
        assert_eq!(store.get_status("привет"), WordStatus::Seen);
    }

    #[test]
    fn test_access_word_keeps_seen_status() {
        let mut store = WordStateStore::new();
        store.set_status("привет", WordStatus::Seen);
        let result = store.access_word("привет");
        assert_eq!(result, WordStatus::Seen);
    }

    #[test]
    fn test_access_word_keeps_known_status() {
        let mut store = WordStateStore::new();
        store.set_status("привет", WordStatus::Known);
        let result = store.access_word("привет");
        assert_eq!(result, WordStatus::Known);
    }

    #[test]
    fn test_case_insensitive_storage() {
        let mut store = WordStateStore::new();
        store.set_status("Привет", WordStatus::Seen);
        assert_eq!(store.get_status("привет"), WordStatus::Seen);
        assert_eq!(store.get_status("ПРИВЕТ"), WordStatus::Seen);
    }

    #[test]
    fn test_force_back_to_unseen() {
        let mut store = WordStateStore::new();
        store.access_word("привет");
        assert_eq!(store.get_status("привет"), WordStatus::Seen);
        store.set_status("привет", WordStatus::Unseen);
        assert_eq!(store.get_status("привет"), WordStatus::Unseen);
    }
}
