use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::WordStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordStateStore {
    states: HashMap<String, WordStatus>,
    dirty: bool,
}

impl WordStateStore {
    pub fn new() -> Self {
        WordStateStore {
            states: HashMap::new(),
            dirty: false,
        }
    }

    pub fn from_states(states: HashMap<String, WordStatus>) -> Self {
        WordStateStore {
            states,
            dirty: false,
        }
    }

    pub fn get_status(&self, word: &str) -> WordStatus {
        *self
            .states
            .get(word.to_lowercase().as_str())
            .unwrap_or(&WordStatus::Unseen)
    }

    pub fn set_status(&mut self, word: &str, status: WordStatus) {
        self.states.insert(word.to_lowercase(), status);
        self.dirty = true;
    }

    /// When a word with status `Unseen` is accessed it automatically becomes
    /// `Seen`; explicit state (`Seen`/`Known`) is left untouched.
    pub fn access_word(&mut self, word: &str) -> WordStatus {
        let key = word.to_lowercase();
        let current = *self.states.get(&key).unwrap_or(&WordStatus::Unseen);

        let new_status = if current == WordStatus::Unseen {
            WordStatus::Seen
        } else {
            current
        };

        self.states.insert(key, new_status);
        self.dirty = true;
        new_status
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Number of words that currently have `status`.
    pub fn count(&self, status: WordStatus) -> usize {
        self.states.values().filter(|&&s| s == status).count()
    }

    pub fn as_map(&self) -> HashMap<String, WordStatus> {
        self.states.clone()
    }
}

impl Default for WordStateStore {
    fn default() -> Self {
        Self::new()
    }
}
