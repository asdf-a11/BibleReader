use crate::domain::stats::{date_from_days, days_from_date, CountHistory, DailyCount};

fn entry(date: &str, count: usize) -> DailyCount {
    DailyCount {
        date: date.to_string(),
        count,
    }
}

#[test]
fn test_record_appends_a_new_day() {
    let mut history = CountHistory::default();
    assert!(history.record("2026-10-18", 3));
    assert!(history.record("2026-10-19", 5));
    assert_eq!(
        history.entries,
        vec![entry("2026-10-18", 3), entry("2026-10-19", 5)]
    );
}

#[test]
fn test_record_overwrites_the_same_day() {
    let mut history = CountHistory::default();
    history.record("2026-10-19", 3);
    history.record("2026-10-19", 4);
    assert_eq!(history.entries, vec![entry("2026-10-19", 4)]);
}

#[test]
fn test_record_handles_a_falling_total() {
    // A known word moved back to seen or unseen lowers the total.
    let mut history = CountHistory::default();
    history.record("2026-10-18", 5);
    assert!(history.record("2026-10-19", 4));
    assert_eq!(history.entries.last(), Some(&entry("2026-10-19", 4)));
}

#[test]
fn test_record_skips_an_unchanged_total() {
    let mut history = CountHistory::default();
    history.record("2026-10-18", 5);
    assert!(!history.record("2026-10-19", 5));
    assert_eq!(history.entries.len(), 1);
}

#[test]
fn test_history_roundtrip() {
    let mut history = CountHistory::default();
    history.record("2026-10-18", 1);
    history.record("2026-10-19", 2);
    let decoded = CountHistory::decode(&history.encode().unwrap()).unwrap();
    assert_eq!(decoded, history);
    assert!(CountHistory::decode("not json").is_err());
}

#[test]
fn test_dates_roundtrip() {
    assert_eq!(date_from_days(0), "1970-01-01");
    assert_eq!(days_from_date("1970-01-01"), Some(0));
    assert_eq!(date_from_days(20_745), "2026-10-19");
    assert_eq!(
        days_from_date("2024-02-29").map(date_from_days).as_deref(),
        Some("2024-02-29")
    );
    for days in [-1000, 0, 59, 60, 11_016, 20_745, 50_000] {
        assert_eq!(days_from_date(&date_from_days(days)), Some(days));
    }
    assert_eq!(days_from_date("garbage"), None);
    assert_eq!(days_from_date("2026-13-01"), None);
}
