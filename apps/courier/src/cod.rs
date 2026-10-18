//! COD register and remittance to consignors; carrier cost / margin on docket.

use crate::domain::Money;

/// One COD collection recorded against an AWB.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CodEntry {
    pub id: String,
    pub sale_id: String,
    pub awb_no: String,
    pub party_id: String,
    pub party_name: String,
    pub cod_paise: i64,
    /// `pending` | `collected` | `remitted`
    pub status: String,
    pub created_at: String,
    pub user_at: String,
    pub device_at: String,
    pub collected_at: String,
    pub remittance_id: String,
    pub note: String,
}

impl CodEntry {
    pub fn cod_money(&self) -> Money {
        Money(self.cod_paise)
    }

    pub fn is_pending(&self) -> bool {
        self.status == "pending" || self.status.is_empty()
    }
}

/// Batch remittance of collected COD back to a consignor.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CodRemittance {
    pub id: String,
    pub party_id: String,
    pub party_name: String,
    pub created_at: String,
    pub user_at: String,
    pub device_at: String,
    pub total_paise: i64,
    pub pay_method: String,
    pub note: String,
    pub entry_ids_json: String,
}

impl CodRemittance {
    pub fn entry_ids(&self) -> Vec<String> {
        serde_json::from_str(&self.entry_ids_json).unwrap_or_default()
    }
}

/// Sell vs carrier cost on a docket.
pub fn margin_paise(sell_paise: i64, carrier_cost_paise: i64) -> i64 {
    sell_paise.saturating_sub(carrier_cost_paise.max(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margin_is_sell_minus_cost() {
        assert_eq!(margin_paise(100_00, 70_00), 30_00);
        assert_eq!(margin_paise(50_00, 70_00), -20_00);
    }
}
