//! Local / LAN ops that close courier-office gaps without cloud.
//!
//! Incoming manifests, pickup runs, auto-assign, rider commission slips,
//! vehicle/fuel log, franchise commission, Tally-ish voucher export,
//! RTO fee lines, destination autocomplete, customer feedback.

use crate::domain::new_id;
use chrono::Local;

/// Bag received from a hub for own-fleet delivery.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct IncomingManifest {
    pub id: String,
    pub created_at: String,
    pub hub_name: String,
    pub note: String,
    pub line_count: i64,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct IncomingManifestLine {
    pub id: String,
    pub manifest_id: String,
    pub awb_no: String,
    pub sale_id: String,
    pub destination: String,
    pub consignee_name: String,
}

/// Request to collect from a consignor (multi-pickup → one drop).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PickupRequest {
    pub id: String,
    pub created_at: String,
    pub party_id: String,
    pub party_name: String,
    pub address: String,
    pub pincode: String,
    pub area: String,
    pub pieces: i64,
    pub note: String,
    pub status: String,
    pub run_id: String,
}

/// One pickup run: several pickup requests → one drop / office return.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PickupRun {
    pub id: String,
    pub created_at: String,
    pub rider_id: String,
    pub rider_name: String,
    pub drop_note: String,
    pub status: String,
    pub request_count: i64,
}

/// Zone/area → rider assignment rule (round-robin within area).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AssignRule {
    pub id: String,
    pub area: String,
    pub pincode_prefix: String,
    pub rider_id: String,
    pub rider_name: String,
    pub round_robin_slot: i64,
}

/// Per-delivery or per-kg pay rate for a rider.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RiderPayRate {
    pub rider_id: String,
    /// Paise per delivered stop (0 = use per-kg only).
    pub per_delivery_paise: i64,
    /// Paise per chargeable kg delivered.
    pub per_kg_paise: i64,
}

/// Daily rider pay + COD settlement slip fields.
#[derive(Clone, Debug, Default)]
pub struct RiderCommissionSlip {
    pub rider_id: String,
    pub rider_name: String,
    pub date: String,
    pub deliveries: i64,
    pub kg_total: rust_decimal::Decimal,
    pub pay_paise: i64,
    pub cod_expected_paise: i64,
    pub cod_handed_paise: i64,
    pub variance_paise: i64,
}

impl RiderCommissionSlip {
    pub fn format_text(&self, shop: &str, money: &dyn Fn(i64) -> String) -> String {
        format!(
            "{shop}\nRider commission slip\n{}\nRider: {}\nDate: {}\nDeliveries: {}\nWeight: {} kg\nPay: {}\nCOD expected: {}\nCOD handed in: {}\nVariance: {}\n",
            Local::now().format("%Y-%m-%d %H:%M"),
            self.rider_name,
            self.date,
            self.deliveries,
            crate::precise::format_kg(self.kg_total),
            money(self.pay_paise),
            money(self.cod_expected_paise),
            money(self.cod_handed_paise),
            money(self.variance_paise),
        )
    }

    pub fn compute_pay(rate: &RiderPayRate, deliveries: i64, kg: rust_decimal::Decimal) -> i64 {
        let per_del = rate.per_delivery_paise.saturating_mul(deliveries.max(0));
        let kg_f = kg.to_string().parse::<f64>().unwrap_or(0.0);
        let per_kg = (rate.per_kg_paise as f64 * kg_f).round() as i64;
        per_del.saturating_add(per_kg.max(0))
    }
}

/// Vehicle inspection checklist + optional fuel note for a run.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct VehicleInspection {
    pub id: String,
    pub manifest_id: String,
    pub rider_id: String,
    pub created_at: String,
    pub lights_ok: bool,
    pub tyres_ok: bool,
    pub brakes_ok: bool,
    pub docs_ok: bool,
    pub fuel_note: String,
    pub note: String,
}

impl VehicleInspection {
    pub fn new(manifest_id: &str, rider_id: &str) -> Self {
        Self {
            id: new_id(),
            manifest_id: manifest_id.into(),
            rider_id: rider_id.into(),
            created_at: Local::now().to_rfc3339(),
            lights_ok: true,
            tyres_ok: true,
            brakes_ok: true,
            docs_ok: true,
            fuel_note: String::new(),
            note: String::new(),
        }
    }

    pub fn checklist_ok(&self) -> bool {
        self.lights_ok && self.tyres_ok && self.brakes_ok && self.docs_ok
    }
}

/// Franchise / carrier commission for a period (% of margin or revenue).
#[derive(Clone, Debug, Default)]
pub struct FranchiseCommissionRow {
    pub carrier_party_id: String,
    pub carrier_name: String,
    pub revenue_paise: i64,
    pub margin_paise: i64,
    pub rate_pct: i64,
    pub commission_paise: i64,
    pub basis: String,
}

impl FranchiseCommissionRow {
    pub fn from_totals(
        carrier_id: &str,
        carrier_name: &str,
        revenue: i64,
        margin: i64,
        rate_pct: i64,
        on_margin: bool,
    ) -> Self {
        let basis_amt = if on_margin { margin } else { revenue };
        let commission = basis_amt.saturating_mul(rate_pct.max(0)) / 100;
        Self {
            carrier_party_id: carrier_id.into(),
            carrier_name: carrier_name.into(),
            revenue_paise: revenue,
            margin_paise: margin,
            rate_pct,
            commission_paise: commission,
            basis: if on_margin { "margin".into() } else { "revenue".into() },
        }
    }
}

pub fn format_franchise_statement(
    shop: &str,
    from: &str,
    to: &str,
    rows: &[FranchiseCommissionRow],
    money: &dyn Fn(i64) -> String,
) -> String {
    let mut out = format!("{shop}\nFranchise commission statement\n{from} → {to}\n--------------------------------\n");
    let mut total = 0i64;
    for r in rows {
        total = total.saturating_add(r.commission_paise);
        out.push_str(&format!(
            "{} · rev {} · margin {} · {}% on {} → {}\n",
            r.carrier_name,
            money(r.revenue_paise),
            money(r.margin_paise),
            r.rate_pct,
            r.basis,
            money(r.commission_paise)
        ));
    }
    out.push_str("--------------------------------\n");
    out.push_str(&format!("Total commission: {}\n", money(total)));
    out
}

/// Tally-friendly voucher CSV (document columns accountants expect).
pub fn tally_voucher_csv(
    rows: &[(String, String, String, String, i64, String)],
) -> String {
    // date, voucher_type, party, ledger, amount_paise, narration
    let mut out = String::from("Date,VoucherType,Party,Ledger,Amount,Narration\n");
    for (date, vtype, party, ledger, amt, narr) in rows {
        out.push_str(&csv_escape(date));
        out.push(',');
        out.push_str(&csv_escape(vtype));
        out.push(',');
        out.push_str(&csv_escape(party));
        out.push(',');
        out.push_str(&csv_escape(ledger));
        out.push(',');
        out.push_str(&amt.to_string());
        out.push(',');
        out.push_str(&csv_escape(narr));
        out.push('\n');
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Suggest destinations from pincode hits + past destinations (beyond exact 6-digit).
pub fn suggest_destinations(
    query: &str,
    pincode_hits: &[(String, String)],
    past: &[String],
    limit: usize,
) -> Vec<String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    let mut push = |s: String| {
        if out.len() >= limit {
            return;
        }
        if !out.iter().any(|x| x.eq_ignore_ascii_case(&s)) {
            out.push(s);
        }
    };
    for (pin, zone) in pincode_hits {
        let pin_l = pin.to_lowercase();
        let zone_l = zone.to_lowercase();
        if pin_l.starts_with(&q) || pin_l.contains(&q) || zone_l.contains(&q) {
            if zone.trim().is_empty() {
                push(pin.clone());
            } else {
                push(format!("{pin} · {zone}"));
            }
        }
    }
    for dest in past {
        let d = dest.to_lowercase();
        if d.contains(&q) || d.starts_with(&q) {
            push(dest.clone());
        }
    }
    out
}

/// Default RTO fee as % of freight (paise), or fixed when `fixed_paise` > 0.
pub fn rto_fee_paise(freight_paise: i64, fixed_paise: i64, pct: i64) -> i64 {
    if fixed_paise > 0 {
        fixed_paise
    } else {
        freight_paise.saturating_mul(pct.max(0)) / 100
    }
}

/// Round-robin pick among riders for an area.
pub fn pick_round_robin<'a>(
    rules: &'a [AssignRule],
    area: &str,
    rr_counter: i64,
) -> Option<&'a AssignRule> {
    let area_l = area.trim().to_lowercase();
    let matched: Vec<&AssignRule> = rules
        .iter()
        .filter(|r| {
            let a = r.area.trim().to_lowercase();
            a.is_empty() || a == area_l || area_l.contains(&a) || a.contains(&area_l)
        })
        .collect();
    if matched.is_empty() {
        return None;
    }
    let i = (rr_counter.max(0) as usize) % matched.len();
    Some(matched[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commission_pay_per_delivery_and_kg() {
        let rate = RiderPayRate {
            rider_id: "r".into(),
            per_delivery_paise: 20_00,
            per_kg_paise: 5_00,
        };
        let pay = RiderCommissionSlip::compute_pay(&rate, 3, "2".parse().unwrap());
        assert_eq!(pay, 60_00 + 10_00);
    }

    #[test]
    fn tally_csv_non_empty() {
        let csv = tally_voucher_csv(&[
            (
                "2026-10-27".into(),
                "Sales".into(),
                "Acme".into(),
                "Freight".into(),
                150_00,
                "AWB-1".into(),
            ),
        ]);
        assert!(csv.contains("VoucherType"));
        assert!(csv.contains("15000"));
        assert!(csv.contains("Acme"));
    }

    #[test]
    fn suggest_beyond_exact_pin() {
        let hits = vec![
            ("560001".into(), "Central".into()),
            ("560002".into(), "South".into()),
        ];
        let past = vec!["Pune Deccan".into(), "Mysuru".into()];
        let s = suggest_destinations("560", &hits, &past, 10);
        assert!(s.iter().any(|x| x.contains("560001")));
        let s2 = suggest_destinations("pun", &hits, &past, 10);
        assert!(s2.iter().any(|x| x.to_lowercase().contains("pune")));
    }

    #[test]
    fn rto_fee_pct_and_fixed() {
        assert_eq!(rto_fee_paise(100_00, 0, 50), 50_00);
        assert_eq!(rto_fee_paise(100_00, 25_00, 50), 25_00);
    }

    #[test]
    fn round_robin_cycles() {
        let rules = vec![
            AssignRule {
                id: "1".into(),
                area: "North".into(),
                rider_id: "a".into(),
                rider_name: "A".into(),
                ..Default::default()
            },
            AssignRule {
                id: "2".into(),
                area: "North".into(),
                rider_id: "b".into(),
                rider_name: "B".into(),
                ..Default::default()
            },
        ];
        assert_eq!(pick_round_robin(&rules, "North", 0).unwrap().rider_id, "a");
        assert_eq!(pick_round_robin(&rules, "North", 1).unwrap().rider_id, "b");
        assert_eq!(pick_round_robin(&rules, "North", 2).unwrap().rider_id, "a");
    }

    #[test]
    fn franchise_on_margin() {
        let r = FranchiseCommissionRow::from_totals("c1", "DTDC", 1000_00, 200_00, 25, true);
        assert_eq!(r.commission_paise, 50_00);
    }
}
