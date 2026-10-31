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

    /// Auto-dismiss seconds (Material-style: errors linger longer).
    pub fn duration_secs(self) -> f64 {
        match self {
            Self::Info => 4.0,
            Self::Success => 4.0,
            Self::Warn => 6.0,
            Self::Error => 8.0,
        }
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
    }

    #[test]
    fn classify_success_and_warn() {
        assert_eq!(classify("Menu order saved"), SnackKind::Success);
        assert_eq!(classify("Sale undone — cart restored"), SnackKind::Success);
        assert_eq!(classify("Read-only on this extra till"), SnackKind::Warn);
    }

    #[test]
    fn priority_error_beats_info() {
        assert!(SnackKind::Error.priority() > SnackKind::Info.priority());
        assert!(SnackKind::Warn.priority() > SnackKind::Success.priority());
    }

    #[test]
    fn durations_sensible() {
        assert!(SnackKind::Error.duration_secs() >= SnackKind::Info.duration_secs());
    }
}
