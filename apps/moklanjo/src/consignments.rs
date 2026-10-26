//! Consignment search, period billing, and credit-limit checks.

use crate::domain::Sale;

/// Filter for desk / reports consignment lists.
#[derive(Clone, Debug, Default)]
pub struct ConsignmentFilter {
    pub awb_contains: String,
    pub party_id: String,
    pub destination_contains: String,
    /// `YYYY-MM-DD` inclusive; empty = no bound.
    pub from_date: String,
    pub to_date: String,
    /// `any` | `unbilled` | `billed`
    pub billed: String,
    /// Tracking status exact match; empty = any.
    pub tracking_status: String,
}

impl ConsignmentFilter {
    pub fn matches(&self, sale: &Sale) -> bool {
        let awb_q = self.awb_contains.trim().to_lowercase();
        if !awb_q.is_empty() && !sale.awb_no.to_lowercase().contains(&awb_q) {
            return false;
        }
        if !self.party_id.trim().is_empty() {
            match sale.customer_id.as_deref() {
                Some(id) if id == self.party_id.trim() => {}
                _ => return false,
            }
        }
        let dest_q = self.destination_contains.trim().to_lowercase();
        if !dest_q.is_empty() && !sale.destination.to_lowercase().contains(&dest_q) {
            return false;
        }
        let day = sale.user_at.get(..10).unwrap_or(&sale.created_at);
        let day = if day.len() >= 10 { &day[..10] } else { day };
        let from = self.from_date.trim();
        if !from.is_empty() && day < from {
            return false;
        }
        let to = self.to_date.trim();
        if !to.is_empty() && day > to {
            return false;
        }
        match self.billed.trim().to_lowercase().as_str() {
            "unbilled" if !sale.period_bill_id.trim().is_empty() => return false,
            "billed" if sale.period_bill_id.trim().is_empty() => return false,
            _ => {}
        }
        let st = self.tracking_status.trim();
        if !st.is_empty() && !sale.tracking_status.eq_ignore_ascii_case(st) {
            return false;
        }
        true
    }
}

/// Result of a credit-limit gate before account booking.
#[derive(Clone, Debug)]
pub enum CreditGate {
    Ok,
    NoLimit,
    OverLimit {
        balance_paise: i64,
        limit_paise: i64,
        booking_paise: i64,
    },
}

impl CreditGate {
    pub fn check(balance_paise: i64, limit_paise: i64, booking_paise: i64) -> Self {
        if limit_paise <= 0 {
            return Self::NoLimit;
        }
        let projected = balance_paise.saturating_add(booking_paise.max(0));
        if projected > limit_paise {
            Self::OverLimit {
                balance_paise,
                limit_paise,
                booking_paise,
            }
        } else {
            Self::Ok
        }
    }

    pub fn allows_booking(&self) -> bool {
        matches!(self, Self::Ok | Self::NoLimit)
    }
}

/// Period bill header — one invoice covering many unbilled account dockets.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PeriodBill {
    pub id: String,
    pub party_id: String,
    pub party_name: String,
    pub invoice_no: String,
    pub created_at: String,
    pub user_at: String,
    pub device_at: String,
    pub from_date: String,
    pub to_date: String,
    pub total_paise: i64,
    pub sale_ids_json: String,
    pub note: String,
}

impl PeriodBill {
    pub fn sale_ids(&self) -> Vec<String> {
        serde_json::from_str(&self.sale_ids_json).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_gate_blocks_over_limit() {
        let g = CreditGate::check(80_00, 100_00, 30_00);
        assert!(!g.allows_booking());
        let g2 = CreditGate::check(80_00, 100_00, 20_00);
        assert!(g2.allows_booking());
        let g3 = CreditGate::check(0, 0, 99_999);
        assert!(g3.allows_booking());
    }
}
