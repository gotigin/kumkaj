//! Text + action helpers for Ops / COD / Riders / Reports facets.
//! Keeps [`crate`] shell thin while the desk can run dispatch from the UI.

use crate::cod;
use crate::consignments::{self, ConsignmentFilter};
use crate::domain::{Money, Sale, StaffMember};
use crate::manifest::{self, HandoverKind};
use crate::store::Store;

/// Build the body shown on the Reports screen for a courier ops facet.
pub fn facet_text(store: &Store, facet: OpsFacet) -> Result<String, String> {
    let mut text = store.courier_ops_summary()?;
    text.push('\n');
    match facet {
        OpsFacet::Bank => {
            text.push_str(&store.books_report_text("today")?);
        }
        OpsFacet::Ops => {
            text.push_str("\n— Consignments (recent AWBs) —\n");
            text.push_str("Type an AWB below, then Advance / Dispatch / Label.\n\n");
            let filter = ConsignmentFilter::default();
            for s in store.search_consignments(&filter, 50)? {
                text.push_str(&format_consignment_line(&s));
            }
            text.push_str("\n— Manifests —\n");
            for m in store.list_manifests(20)? {
                let who = if m.kind == "own_fleet" {
                    m.rider_name.as_str()
                } else {
                    m.carrier_name.as_str()
                };
                text.push_str(&format!(
                    "{} · {} · {} lines · {}\n",
                    m.kind, who, m.line_count, &m.created_at[..m.created_at.len().min(16)]
                ));
            }
        }
        OpsFacet::Cod => {
            text.push_str("\n— COD register —\n");
            text.push_str("Collect pending, then Remit for a party (name = consignor).\n\n");
            for e in store.list_cod_entries("any")? {
                text.push_str(&format!(
                    "{} · {} · {} · {}{}\n",
                    e.awb_no,
                    e.status,
                    Money(e.cod_paise).format_with("INR"),
                    e.party_name,
                    if e.note.is_empty() {
                        String::new()
                    } else {
                        format!(" · {}", e.note)
                    }
                ));
            }
        }
        OpsFacet::Riders => {
            text.push_str("\n— Own-fleet riders —\n");
            text.push_str("Add a rider, then Dispatch booked AWBs to them from Ops.\n\n");
            let riders = store.list_riders()?;
            if riders.is_empty() {
                text.push_str("(none yet)\n");
            }
            for r in riders {
                text.push_str(&format!(
                    "{} · {} · {}\n",
                    r.name, r.phone, r.vehicle_label
                ));
            }
        }
    }
    Ok(text)
}

fn format_consignment_line(s: &Sale) -> String {
    format!(
        "{} · {} · {} · {} · {}{}\n",
        s.awb_no,
        s.destination,
        s.tracking_status,
        if s.period_bill_id.is_empty() {
            "unbilled"
        } else {
            "billed"
        },
        Money(s.total_paise).format_with("INR"),
        if s.cod_paise > 0 {
            format!(" · COD {}", Money(s.cod_paise).format_with("INR"))
        } else {
            String::new()
        }
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpsFacet {
    Bank,
    Ops,
    Cod,
    Riders,
}

pub fn advance_awb(
    store: &mut Store,
    awb: &str,
    status: TrackingStatus,
    note: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    let sale = store
        .find_sale_by_awb(awb)?
        .ok_or_else(|| format!("AWB not found: {awb}"))?;
    store.advance_tracking(&sale.id, status, note, "", actor)?;
    Ok(format!("{} → {}", sale.awb_no, status.label()))
}

pub fn print_label(store: &Store, awb: &str) -> Result<String, String> {
    let sale = store
        .find_sale_by_awb(awb)?
        .ok_or_else(|| format!("AWB not found: {awb}"))?;
    let path = store.write_awb_label(&sale.id)?;
    Ok(format!("Label saved: {}", path.display()))
}

/// Dispatch all currently booked AWBs (limit) to a rider by name.
pub fn dispatch_booked_to_rider(
    store: &mut Store,
    rider_name: &str,
    note: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    let rider_name = rider_name.trim();
    if rider_name.is_empty() {
        return Err("Type a rider name".into());
    }
    let rider = store
        .list_riders()?
        .into_iter()
        .find(|r| r.name.eq_ignore_ascii_case(rider_name))
        .ok_or_else(|| format!("No rider named {rider_name} — add them first"))?;
    let filter = ConsignmentFilter {
        tracking_status: "booked".into(),
        ..Default::default()
    };
    let sales = store.search_consignments(&filter, 80)?;
    let ids: Vec<String> = sales.into_iter().map(|s| s.id).collect();
    if ids.is_empty() {
        return Err("No booked AWBs to dispatch".into());
    }
    let m = store.create_manifest(
        HandoverKind::OwnFleet,
        "",
        &rider.id,
        &ids,
        note,
        actor,
    )?;
    Ok(format!(
        "Own-fleet run · {} · {} AWBs → {}",
        m.rider_name, m.line_count, rider.name
    ))
}

/// Hand booked AWBs to a carrier party by name.
pub fn handover_booked_to_carrier(
    store: &mut Store,
    carrier_name: &str,
    note: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    let carrier_name = carrier_name.trim();
    if carrier_name.is_empty() {
        return Err("Type a carrier party name".into());
    }
    let party = store
        .list_customers()?
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(carrier_name))
        .ok_or_else(|| format!("No party named {carrier_name}"))?;
    let filter = ConsignmentFilter {
        tracking_status: "booked".into(),
        ..Default::default()
    };
    let sales = store.search_consignments(&filter, 80)?;
    let ids: Vec<String> = sales.into_iter().map(|s| s.id).collect();
    if ids.is_empty() {
        return Err("No booked AWBs to hand over".into());
    }
    let m = store.create_manifest(
        HandoverKind::Carrier,
        &party.id,
        "",
        &ids,
        note,
        actor,
    )?;
    Ok(format!(
        "Carrier bag · {} · {} AWBs → {}",
        m.carrier_name, m.line_count, party.name
    ))
}

pub fn collect_cod_for_awb(store: &Store, awb: &str) -> Result<String, String> {
    let awb = awb.trim();
    let entry = store
        .list_cod_entries("pending")?
        .into_iter()
        .find(|e| e.awb_no.eq_ignore_ascii_case(awb))
        .ok_or_else(|| format!("No pending COD for {awb}"))?;
    store.mark_cod_collected(&entry.id)?;
    Ok(format!(
        "Collected COD {} on {}",
        Money(entry.cod_paise).format_with("INR"),
        entry.awb_no
    ))
}

pub fn remit_collected_for_party(store: &mut Store, party_name: &str) -> Result<String, String> {
    let party_name = party_name.trim();
    if party_name.is_empty() {
        return Err("Type the consignor party name".into());
    }
    let party = store
        .list_customers()?
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(party_name))
        .ok_or_else(|| format!("No party named {party_name}"))?;
    let entries: Vec<_> = store
        .list_cod_entries("collected")?
        .into_iter()
        .filter(|e| e.party_id == party.id)
        .collect();
    if entries.is_empty() {
        return Err("No collected COD for that party".into());
    }
    let ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
    let rem = store.remit_cod(&party.id, &ids, "cash", "desk remittance")?;
    Ok(format!(
        "Remitted {} to {} ({} dockets)",
        Money(rem.total_paise).format_with("INR"),
        rem.party_name,
        ids.len()
    ))
}

pub fn add_rider_line(
    store: &Store,
    name: &str,
    phone: &str,
    vehicle: &str,
) -> Result<String, String> {
    let r = store.add_rider(name, phone, vehicle)?;
    Ok(format!("Rider added: {}", r.name))
}

pub fn period_bill_party(
    store: &mut Store,
    party_name: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    let party_name = party_name.trim();
    if party_name.is_empty() {
        return Err("Type a party name".into());
    }
    let party = store
        .list_customers()?
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(party_name))
        .ok_or_else(|| format!("No party named {party_name}"))?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    // Broad window: last ~90 days of unbilled account dockets.
    let from = chrono::Local::now() - chrono::Duration::days(90);
    let from_s = from.format("%Y-%m-%d").to_string();
    let bill = store.create_period_bill(&party.id, &from_s, &today, "Period bill", actor)?;
    Ok(format!(
        "Period bill {} · {} · {}",
        bill.invoice_no,
        bill.party_name,
        Money(bill.total_paise).format_with("INR")
    ))
}

pub fn set_credit_limit(
    store: &Store,
    party_name: &str,
    limit_paise: i64,
) -> Result<String, String> {
    let party = store
        .list_customers()?
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(party_name.trim()))
        .ok_or_else(|| "Party not found".to_string())?;
    store.set_customer_credit_limit(&party.id, limit_paise)?;
    Ok(format!(
        "Credit limit for {} set to {}",
        party.name,
        Money(limit_paise).format_with("INR")
    ))
}

pub use cod::margin_paise;
pub use consignments::CreditGate;
pub use manifest::TrackingStatus;
