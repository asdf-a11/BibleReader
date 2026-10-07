use std::collections::HashMap;

use crate::domain::{WordStateStore, WordStatus};

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
fn test_access_word_keeps_explicit_status() {
    let mut store = WordStateStore::new();
    store.set_status("привет", WordStatus::Known);
    assert_eq!(store.access_word("привет"), WordStatus::Known);
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
    store.set_status("привет", WordStatus::Unseen);
    assert_eq!(store.get_status("привет"), WordStatus::Unseen);
}

#[test]
fn test_from_states_and_as_map_roundtrip() {
    let mut map = HashMap::new();
    map.insert("бог".to_string(), WordStatus::Known);
    let store = WordStateStore::from_states(map);
    assert_eq!(store.get_status("бог"), WordStatus::Known);
    assert_eq!(store.as_map().get("бог"), Some(&WordStatus::Known));
}

#[test]
fn test_dirty_flag_tracks_changes_only() {
    let mut store = WordStateStore::new();
    assert!(!store.is_dirty());

    store.set_status("начале", WordStatus::Seen);
    assert!(store.is_dirty());

    store.clear_dirty();
    assert!(!store.is_dirty());

    store.access_word("небо");
    assert!(store.is_dirty());
}
