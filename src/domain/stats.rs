//! How many words are known and seen, and how that changed over time.
//!
//! The history keeps one entry per day: the total at the end of that day.
//! Totals rather than "+1" events are stored, so a word moving back from known
//! to seen (or to unseen) is reflected without any bookkeeping of its own.

use serde::{Deserialize, Serialize};

/// The total on one day. `date` is `YYYY-MM-DD`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyCount {
    pub date: String,
    pub count: usize,
}

/// Day by day totals of one word status, oldest first.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountHistory {
    pub entries: Vec<DailyCount>,
}

impl CountHistory {
    /// Record `count` as the total on `date`. Returns whether anything changed.
    ///
    /// A second change on the same day overwrites that day's entry, and a
    /// total equal to the last recorded one adds nothing, so the file only
    /// grows on days the total actually moved.
    pub fn record(&mut self, date: &str, count: usize) -> bool {
        match self.entries.last_mut() {
            Some(last) if last.count == count => false,
            Some(last) if last.date == date => {
                last.count = count;
                true
            }
            _ => {
                self.entries.push(DailyCount {
                    date: date.to_string(),
                    count,
                });
                true
            }
        }
    }

    pub fn encode(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize history: {}", e))
    }

    pub fn decode(content: &str) -> Result<CountHistory, String> {
        serde_json::from_str(content).map_err(|e| format!("Failed to parse history file: {}", e))
    }
}

/// The known and seen histories, saved to a file each.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatsHistory {
    pub known: CountHistory,
    pub seen: CountHistory,
}

/// Today's date (UTC) as `YYYY-MM-DD`.
pub fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    date_from_days((secs / 86_400) as i64)
}

/// `YYYY-MM-DD` of the day `days` days after 1970-01-01.
pub fn date_from_days(days: i64) -> String {
    // Howard Hinnant's `civil_from_days`.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/// Days since 1970-01-01 of a `YYYY-MM-DD` date, or `None` if it is malformed.
pub fn days_from_date(date: &str) -> Option<i64> {
    let mut parts = date.splitn(3, '-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Howard Hinnant's `days_from_civil`.
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}
