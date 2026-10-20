//! Text + action helpers for Ops / COD / Riders / Reports facets.
//! Keeps [`crate`] shell thin while the desk can run dispatch from the UI.
//!
//! Barcode desk loop: print Code 128 on the AWB label; USB/Bluetooth wedge
//! scanners type into the AWB field; Enter / Find resolves the parcel for
//! advance, label, and COD actions.

use crate::cod;
use crate::consignments::{self, ConsignmentFilter};
use crate::domain::{Sale, StaffMember};
use crate::manifest::{self, HandoverKind};
use crate::store::Store;

/// Strip wedge-scanner noise (`*AWB*`, CR/LF) so barcode input matches `awb_no`.
pub fn normalize_awb_input(raw: &str) -> String {
    let mut s = raw.trim().trim_matches(|c: char| c == '\r' || c == '\n' || c == '\0');
    // Code-128 human-readable often printed as *PAYLOAD*
    if s.len() >= 2 && s.starts_with('*') && s.ends_with('*') {
        s = s[1..s.len() - 1].trim();
    }
    s.trim().to_string()
}

/// Resolve a typed or scanned AWB to a sale (exact, then case-insensitive).
pub fn find_sale_for_scan(store: &Store, raw_awb: &str) -> Result<Sale, String> {
    let awb = normalize_awb_input(raw_awb);
    if awb.is_empty() {
        return Err("Scan or type an AWB".into());
    }
    if let Some(sale) = store.find_sale_by_awb(&awb)? {
        return Ok(sale);
    }
    // Case-insensitive fallback (some scanners change case).
    let filter = ConsignmentFilter {
        awb_contains: awb.clone(),
        ..Default::default()
    };
    let hits = store.search_consignments(&filter, 5)?;
    let exact: Vec<_> = hits
        .into_iter()
        .filter(|s| s.awb_no.eq_ignore_ascii_case(&awb))
        .collect();
    match exact.len() {
        0 => Err(format!("AWB not found: {awb}")),
        1 => Ok(exact.into_iter().next().unwrap()),
        _ => Err(format!("Several AWBs match {awb} — type the full number")),
    }
}

/// Desk card after scan/find: status, party, COD, suggested next actions.
pub fn awb_desk_card(store: &Store, raw_awb: &str) -> Result<String, String> {
    let sale = find_sale_for_scan(store, raw_awb)?;
    let status = TrackingStatus::parse(&sale.tracking_status);
    let party = sale
        .customer_id
        .as_deref()
        .filter(|id| !id.is_empty())
        .and_then(|id| store.get_customer(id).ok().flatten());
    let party_line = party
        .as_ref()
        .map(|p| {
            let kind = if p.kind_label.trim().is_empty() {
                String::new()
            } else {
                format!(" ({})", p.kind_label.trim())
            };
            format!("{}{kind}", p.name)
        })
        .unwrap_or_else(|| "(walk-in / no party)".into());
    let events = store.list_tracking_events(&sale.id)?;
    let last_note = events
        .last()
        .map(|e| e.note.trim())
        .filter(|n| !n.is_empty())
        .unwrap_or("");
    let next = match status {
        TrackingStatus::Booked => "Next: Dispatch to rider / Hand to carrier · or Out for delivery",
        TrackingStatus::OutForDelivery => "Next: Mark delivered (+ POD photo optional)",
        TrackingStatus::Delivered => "Done — Collect COD if pending · Print label if reprint",
        _ => "Next: Advance status or Print label",
    };
    let mut out = format!(
        "=== Scanned AWB ===\n\
         AWB {}\n\
         Status: {}\n\
         Party: {}\n\
         To: {} · {}\n\
         Weight: {} kg\n\
         Charge: {}\n",
        sale.awb_no,
        status.label(),
        party_line,
        sale.destination,
        sale.consignee_name,
        crate::precise::format_kg(sale.chargeable_weight_kg),
        store.money_display(sale.total_paise),
    );
    if sale.cod_paise > 0 {
        out.push_str(&format!(
            "COD: {}\n",
            store.money_display(sale.cod_paise)
        ));
    }
    if !last_note.is_empty() {
        out.push_str(&format!("Last note: {last_note}\n"));
    }
    out.push_str(next);
    out.push('\n');
    out.push_str(
        "Scan again or tap Out for delivery / Delivered / Print label / Collect COD.\n",
    );
    Ok(out)
}

/// Build the body shown on the Reports screen for a courier ops facet.
pub fn facet_text(store: &Store, facet: OpsFacet) -> Result<String, String> {
    let mut text = store.courier_ops_summary()?;
    text.push('\n');
    match facet {
        OpsFacet::Bank => {
            text.push_str(&store.books_report_text("today")?);
        }
        OpsFacet::Ops => {
            text.push_str("\n— Barcode desk —\n");
            text.push_str(
                "Scan AWB into the field (USB scanner) or type it → Find / Enter.\n\
                 Then Out for delivery, Delivered, Print label, or Collect COD.\n\
                 Labels print Code 128 for the same AWB.\n\n",
            );
            text.push_str("— Consignments (recent AWBs) —\n");
            let filter = ConsignmentFilter::default();
            for s in store.search_consignments(&filter, 50)? {
                text.push_str(&format_consignment_line(store, &s));
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
                    store.money_display(e.cod_paise),
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

fn format_consignment_line(store: &Store, s: &Sale) -> String {
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
        store.money_display(s.total_paise),
        if s.cod_paise > 0 {
            format!(" · COD {}", store.money_display(s.cod_paise))
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
    advance_awb_with_pod(store, awb, status, note, None, actor)
}

/// Advance tracking; optional POD photo → WebP attachment on the sale; id in `pod_ref`.
pub fn advance_awb_with_pod(
    store: &mut Store,
    awb: &str,
    status: TrackingStatus,
    note: &str,
    pod_image: Option<&std::path::Path>,
    actor: &StaffMember,
) -> Result<String, String> {
    use crate::domain::AttachmentEntity;
    let sale = find_sale_for_scan(store, awb)?;
    let mut pod_ref = String::new();
    if let Some(path) = pod_image {
        let att =
            store.add_attachment_image(AttachmentEntity::Sale, &sale.id, path, "POD", actor)?;
        pod_ref = att.id;
    }
    if !note.trim().is_empty() {
        let _ = store.add_attachment_note(
            AttachmentEntity::Sale,
            &sale.id,
            "Tracking note",
            note,
            actor,
        )?;
    }
    let _ = store.advance_tracking(&sale.id, status, note, &pod_ref, actor)?;
    Ok(format!("{} → {}", sale.awb_no, status.label()))
}

pub fn print_label(store: &Store, awb: &str) -> Result<String, String> {
    let sale = find_sale_for_scan(store, awb)?;
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
    let awb = normalize_awb_input(awb);
    let entry = store
        .list_cod_entries("pending")?
        .into_iter()
        .find(|e| e.awb_no.eq_ignore_ascii_case(&awb))
        .ok_or_else(|| format!("No pending COD for {awb}"))?;
    store.mark_cod_collected(&entry.id)?;
    Ok(format!(
        "Collected COD {} on {}",
        store.money_display(entry.cod_paise),
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
        store.money_display(rem.total_paise),
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
        store.money_display(bill.total_paise)
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
        store.money_display(limit_paise)
    ))
}

pub use cod::margin_paise;
pub use consignments::CreditGate;
pub use manifest::TrackingStatus;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_stars_and_whitespace() {
        assert_eq!(normalize_awb_input("  *AWB-1* \r\n"), "AWB-1");
        assert_eq!(normalize_awb_input("AWB-1"), "AWB-1");
        assert_eq!(normalize_awb_input(""), "");
    }
}
