//! Standard snackbar kinds — privacy-safe brief feedback above chrome.
//!
//! Duplicated per app (no shared product crate). UI chrome lives in `main.rs`;
//! this module owns classification, priority, and durations.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SnackKind {
    #[default]
    Info,
    Success,
    Warn,
    Error,
}

impl SnackKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    /// Higher wins when a new snack arrives while one is showing.
    pub fn priority(self) -> u8 {
        match self {
            Self::Info => 1,
            Self::Success => 2,
            Self::Warn => 3,
            Self::Error => 4,
        }
    }

    /// Default base seconds when the user has not set a preference. The old
    /// fixed 4 s went before a busy counter could read it (Khaynjo's shop
    /// trials, 2026-10); 10 s is the suite default.
    pub const DEFAULT_SECS: f64 = 10.0;

    /// Clamp a typed preference into a sane range.
    pub fn clamp_secs(secs: f64) -> f64 {
        if !secs.is_finite() {
            return Self::DEFAULT_SECS;
        }
        secs.clamp(3.0, 60.0)
    }

    /// Auto-dismiss seconds from the signed-in user's base time (Settings →
    /// Alerts). Warnings and errors still linger longer than plain news.
    pub fn duration_secs(self, user_secs: f64) -> f64 {
        let base = Self::clamp_secs(user_secs);
        let factor = match self {
            Self::Info | Self::Success => 1.0,
            Self::Warn => 1.5,
            Self::Error => 2.0,
        };
        (base * factor).min(60.0)
    }
}

/// Guess kind from a status string so existing `set_status` call sites get a
/// sensible face without rewriting every `Err` branch.
pub fn classify(msg: &str) -> SnackKind {
    let m = msg.trim().to_ascii_lowercase();
    if m.is_empty() {
        return SnackKind::Info;
    }
    const ERR: &[&str] = &[
        "error",
        "fail",
        "failed",
        "refused",
        "denied",
        "not allowed",
        "not ready",
        "gone",
        "invalid",
        // Lock / ClientPair: "Wrong PIN…", "Wrong pairing code…" (must Error —
        // Info snacks are suppressed while locked, so the bar never appeared).
        "wrong",
        "unable",
        "could not",
        "couldn't",
        "cannot",
        "can't",
        "blocked",
        "timeout",
        "timed out",
        "storage error",
        "no store",
        "nothing to",
        "missing",
        "required",
        "share blocked",
    ];
    for w in ERR {
        if m.contains(w) {
            return SnackKind::Error;
        }
    }
    const WARN: &[&str] = &[
        "warning",
        "overdue",
        "read-only",
        "offline",
        "retry",
        "waiting",
        "still",
        "partial",
        // Print route empty — must beat OK "printed" inside "not printed".
        "not printed",
        "no printer",
        "add a kitchen printer",
        "shop device",
    ];
    for w in WARN {
        if m.contains(w) {
            return SnackKind::Warn;
        }
    }
    const OK: &[&str] = &[
        "saved",
        "done",
        "ready",
        "recorded",
        "posted",
        "copied",
        "undone",
        "restored",
        "imported",
        "exported",
        "cleared",
        "approved",
        "paired",
        "connected",
        "printed",
        "sent",
        "ok —",
        "shop ready",
    ];
    for w in OK {
        // "KOTs are not printed…" must never look like success.
        if *w == "printed" && m.contains("not printed") {
            continue;
        }
        if m.contains(w) {
            return SnackKind::Success;
        }
    }
    SnackKind::Info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_errors() {
        assert_eq!(classify("Not allowed for your role"), SnackKind::Error);
        assert_eq!(classify("Storage error: disk full"), SnackKind::Error);
        assert_eq!(classify("Share blocked: approve first"), SnackKind::Error);
        assert_eq!(
            classify("Wrong pairing code — check it on the shop device"),
            SnackKind::Error
        );
        assert_eq!(
            classify("Wrong PIN for Ahmed · Cashier · #abc12345 — 4 tries left"),
            SnackKind::Error
        );
    }

    #[test]
    fn classify_success_and_warn() {
        assert_eq!(classify("Menu order saved"), SnackKind::Success);
        assert_eq!(classify("Sale undone — cart restored"), SnackKind::Success);
        assert_eq!(classify("Read-only on this extra till"), SnackKind::Warn);
        assert_eq!(
            classify(
                "KOTs are not printed on this device — add a kitchen printer in Settings → Printers"
            ),
            SnackKind::Warn
        );
        assert_eq!(
            classify("Print selected again on the shop device (kitchen printers are on the counter)"),
            SnackKind::Warn
        );
        assert_eq!(classify("2 KOT(s) → Kitchen · printed"), SnackKind::Success);
    }

    #[test]
    fn priority_error_beats_info() {
        assert!(SnackKind::Error.priority() > SnackKind::Info.priority());
        assert!(SnackKind::Warn.priority() > SnackKind::Success.priority());
    }

    #[test]
    fn durations_follow_the_users_time_and_errors_linger() {
        assert!((SnackKind::Info.duration_secs(10.0) - 10.0).abs() < f64::EPSILON);
        assert!((SnackKind::Success.duration_secs(4.0) - 4.0).abs() < f64::EPSILON);
        assert!((SnackKind::Warn.duration_secs(10.0) - 15.0).abs() < f64::EPSILON);
        assert!((SnackKind::Error.duration_secs(10.0) - 20.0).abs() < f64::EPSILON);
        // Never past a minute, never under 3 s, and junk falls back to 10.
        assert!((SnackKind::Error.duration_secs(45.0) - 60.0).abs() < f64::EPSILON);
        assert!((SnackKind::clamp_secs(1.0) - 3.0).abs() < f64::EPSILON);
        assert!((SnackKind::clamp_secs(99.0) - 60.0).abs() < f64::EPSILON);
        assert!((SnackKind::clamp_secs(f64::NAN) - SnackKind::DEFAULT_SECS).abs() < f64::EPSILON);
    }
}
