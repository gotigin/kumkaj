//! Business-day window and history retention (shop meta).
//!
//! "Today" for books / day close follows this window — not always calendar midnight.

use chrono::{Duration, Local, NaiveTime, Timelike};

/// How the shop draws the current business day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusinessDayMode {
    /// Local calendar date 00:00 → next midnight (legacy).
    CalendarMidnight,
    /// From last day close (or start of time) until the next day close.
    UntilDayClose,
    /// Rolls at a fixed local hour each day (0–23).
    AtHour(u32),
}

impl BusinessDayMode {
    pub fn parse(raw: &str) -> Self {
        let s = raw.trim().to_ascii_lowercase();
        if s == "until_day_close" || s == "day_close" {
            return Self::UntilDayClose;
        }
        if let Some(rest) = s.strip_prefix("at_hour:") {
            if let Ok(h) = rest.trim().parse::<u32>() {
                return Self::AtHour(h.min(23));
            }
        }
        if let Some(rest) = s.strip_prefix("at_hour") {
            let h = rest.trim().trim_start_matches(':').parse::<u32>().unwrap_or(4);
            return Self::AtHour(h.min(23));
        }
        Self::CalendarMidnight
    }

    pub fn as_str(self) -> String {
        match self {
            Self::CalendarMidnight => "calendar_midnight".into(),
            Self::UntilDayClose => "until_day_close".into(),
            Self::AtHour(h) => format!("at_hour:{h}"),
        }
    }
}

/// When trading history is purged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryRetention {
    Off,
    OnDayClose,
    AfterHours(u32),
}

impl HistoryRetention {
    pub fn parse(mode: &str, hours: u32) -> Self {
        match mode.trim().to_ascii_lowercase().as_str() {
            "on_day_close" | "day_close" => Self::OnDayClose,
            "after_hours" | "hours" => Self::AfterHours(hours.max(1).min(168)),
            _ => Self::Off,
        }
    }

    pub fn mode_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::OnDayClose => "on_day_close",
            Self::AfterHours(_) => "after_hours",
        }
    }

    pub fn hours(self) -> u32 {
        match self {
            Self::AfterHours(h) => h,
            _ => 12,
        }
    }
}

/// Inclusive start / exclusive end as RFC3339-ish strings usable in SQL `>=` / `<`.
#[derive(Clone, Debug)]
pub struct BusinessDayWindow {
    pub start: String,
    pub end: String,
    /// Short label for UI (calendar date of the business day).
    pub label_date: String,
}

/// Compute the open business-day window at `now`.
pub fn business_day_window(
    mode: BusinessDayMode,
    last_day_close_at: Option<&str>,
    now: chrono::DateTime<Local>,
) -> BusinessDayWindow {
    let end = now.to_rfc3339();
    match mode {
        BusinessDayMode::CalendarMidnight => {
            let day = now.date_naive();
            let start = day
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .single()
                .unwrap_or(now)
                .to_rfc3339();
            BusinessDayWindow {
                start,
                end,
                label_date: day.format("%Y-%m-%d").to_string(),
            }
        }
        BusinessDayMode::UntilDayClose => {
            let start = last_day_close_at
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    // Never closed: treat as "since local midnight of first sale day"
                    // — use a far-past bound so all open history counts as today.
                    "1970-01-01T00:00:00+00:00".into()
                });
            let label = now.date_naive().format("%Y-%m-%d").to_string();
            BusinessDayWindow {
                start,
                end,
                label_date: label,
            }
        }
        BusinessDayMode::AtHour(hour) => {
            let hour = hour.min(23);
            let today = now.date_naive();
            let boundary_naive = today
                .and_time(NaiveTime::from_hms_opt(hour, 0, 0).unwrap_or(NaiveTime::MIN));
            let boundary = boundary_naive
                .and_local_timezone(Local)
                .single()
                .unwrap_or(now);
            let start_dt = if now >= boundary {
                boundary
            } else {
                boundary - Duration::days(1)
            };
            let label = if now.hour() >= hour {
                today.format("%Y-%m-%d").to_string()
            } else {
                (today - Duration::days(1)).format("%Y-%m-%d").to_string()
            };
            BusinessDayWindow {
                start: start_dt.to_rfc3339(),
                end,
                label_date: label,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn parses_modes() {
        assert_eq!(
            BusinessDayMode::parse("until_day_close"),
            BusinessDayMode::UntilDayClose
        );
        assert_eq!(BusinessDayMode::parse("at_hour:4"), BusinessDayMode::AtHour(4));
        assert_eq!(
            BusinessDayMode::parse("nope"),
            BusinessDayMode::CalendarMidnight
        );
    }

    #[test]
    fn at_hour_before_rollover_uses_yesterday() {
        let now = Local
            .with_ymd_and_hms(2026, 11, 5, 2, 0, 0)
            .single()
            .unwrap();
        let w = business_day_window(BusinessDayMode::AtHour(4), None, now);
        assert!(w.start.contains("2026-11-04"));
    }
}
