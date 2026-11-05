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
        TrackingStatus::OutForDelivery => {
            "Next: Mark delivered (+ POD) · or Failed delivery (NDR)"
        }
        TrackingStatus::Ndr => "Next: Reattempt · or wait for auto-RTO after max attempts",
        TrackingStatus::Delivered => "Done — Collect COD if pending · Print label if reprint",
        TrackingStatus::Rto => "RTO — returned to origin",
        _ => "Next: Advance status or Print label",
    };
    let mut out = format!(
        "=== Scanned AWB ===\n\
         AWB: {}\n\
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
    if sale.attempt_count > 0 || !sale.ndr_reason.trim().is_empty() {
        out.push_str(&format!(
            "Attempts: {} · NDR: {}\n",
            sale.attempt_count,
            if sale.ndr_reason.trim().is_empty() {
                "—"
            } else {
                sale.ndr_reason.trim()
            }
        ));
    }
    if !sale.next_attempt_at.trim().is_empty() {
        out.push_str(&format!("Next attempt: {}\n", sale.next_attempt_at.trim()));
    }
    if !last_note.is_empty() {
        out.push_str(&format!("Last note: {last_note}\n"));
    }
    out.push_str(next);
    out.push('\n');
    out.push_str(
        "Scan again or tap Out for delivery / Delivered / Failed delivery / Reattempt / Print label / Collect COD.\n",
    );
    Ok(out)
}

/// Build the body shown on the Reports screen for a courier ops facet.
pub fn facet_text(store: &Store, facet: OpsFacet) -> Result<String, String> {
    match facet {
        OpsFacet::Cod => return cod_facet_text(store),
        OpsFacet::Riders => return riders_facet_text(store),
        _ => {}
    }
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
            text.push_str("Print run sheet / Print DRS prints the latest manifest.\n");
            for m in store.list_manifests(20)? {
                let who = if m.kind == "own_fleet" {
                    m.rider_name.as_str()
                } else {
                    m.carrier_name.as_str()
                };
                text.push_str(&format!(
                    "{} · {} · {} lines · {} · id {}\n",
                    m.kind,
                    who,
                    m.line_count,
                    &m.created_at[..m.created_at.len().min(16)],
                    &m.id[..m.id.len().min(8)]
                ));
            }
        }
        OpsFacet::Cod => {
            // Rebuilt below without the generic ops summary.
        }
        OpsFacet::Riders => {}
    }
    Ok(text)
}

/// COD the way the desk asks about it: who still owes us (delivered, not
/// yet collected), what is still on the road, and what we owe each
/// consignor (collected, not yet paid out). Words in the shop's language.
fn cod_facet_text(store: &Store) -> Result<String, String> {
    let lang = store.locale_settings()?.language;
    let t = |k: &str| crate::locale::t(&lang, k);
    let mut to_collect: Vec<(String, i64)> = Vec::new();
    let mut on_road: Vec<(String, i64)> = Vec::new();
    for e in store.list_cod_entries("pending")? {
        let Some(sale) = store.get_sale(&e.sale_id)? else {
            continue;
        };
        if sale.status != "completed" {
            continue;
        }
        let status = TrackingStatus::parse(&sale.tracking_status);
        let who = if sale.consignee_name.trim().is_empty() {
            e.party_name.clone()
        } else {
            sale.consignee_name.trim().to_string()
        };
        if status == TrackingStatus::Delivered {
            to_collect.push((format!("{} · {who}", e.awb_no), e.cod_paise));
        } else if !status.is_final() {
            on_road.push((format!("{} · {}", e.awb_no, t(status.label_key())), e.cod_paise));
        }
    }
    let mut owed: Vec<(String, usize, i64)> = Vec::new();
    for e in store.list_cod_entries("collected")? {
        let name = if e.party_name.trim().is_empty() {
            "—".to_string()
        } else {
            e.party_name.trim().to_string()
        };
        match owed.iter_mut().find(|(n, _, _)| *n == name) {
            Some(row) => {
                row.1 += 1;
                row.2 += e.cod_paise;
            }
            None => owed.push((name, 1, e.cod_paise)),
        }
    }
    let mut text = String::new();
    let mut section = |head: String, rows: Vec<(String, i64)>| {
        text.push_str(&format!("=== {head} ===\n"));
        if rows.is_empty() {
            text.push_str(&format!("{}\n", t("lane_empty")));
        }
        let total: i64 = rows.iter().map(|(_, p)| *p).sum();
        for (label, paise) in &rows {
            text.push_str(&format!("{label}: {}\n", store.money_display(*paise)));
        }
        if rows.len() > 1 {
            text.push_str(&format!("{}: {}\n", t("total"), store.money_display(total)));
        }
        text.push('\n');
    };
    let rto_due: Vec<(String, i64)> = store
        .list_rto_fees_to_collect()?
        .into_iter()
        .map(|(awb, who, paise)| (format!("{awb} · {who}"), paise))
        .collect();
    section(t("cod_to_collect"), to_collect);
    section(t("cod_on_road"), on_road);
    section(
        t("cod_owed_consignors"),
        owed.into_iter()
            .map(|(name, n, paise)| {
                (format!("{name} · {}", crate::locale::parcels(&lang, n as i64)), paise)
            })
            .collect(),
    );
    section(t("cod_rto_fees"), rto_due);
    Ok(text)
}

/// Each rider today: what they have out, what they delivered, and the COD
/// they should hand in.
fn riders_facet_text(store: &Store) -> Result<String, String> {
    let lang = store.locale_settings()?.language;
    let t = |k: &str| crate::locale::t(&lang, k);
    let riders = store.list_riders()?;
    if riders.is_empty() {
        return Ok(format!("{}\n", t("lane_empty")));
    }
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut text = String::new();
    for r in riders {
        let (out_now, delivered) = store.rider_day_counts(&r.id, &today)?;
        let expected = store.rider_cod_expected(&r.id, &today).unwrap_or(0);
        let mut head = r.name.clone();
        for part in [r.vehicle_label.trim(), r.phone.trim()] {
            if !part.is_empty() {
                head.push_str(&format!(" · {part}"));
            }
        }
        text.push_str(&format!("=== {head} ===\n"));
        text.push_str(&format!("{}: {out_now}\n", t("out_for_delivery")));
        text.push_str(&format!("{}: {delivered}\n", t("trk_delivered")));
        text.push_str(&format!("{}: {}\n\n", t("cod_to_hand_in"), store.money_display(expected)));
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
    store: &Store,
    awb: &str,
    status: TrackingStatus,
    note: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    advance_awb_with_pod(store, awb, status, note, None, actor)
}

/// Advance tracking; optional POD photo → WebP attachment on the sale; id in `pod_ref`.
pub fn advance_awb_with_pod(
    store: &Store,
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
    let path = store.print_awb_label(&sale.id)?;
    if crate::print_share::prints_route(crate::print_share::RouteKey::Label) {
        Ok(format!("Label printed · also saved: {}", path.display()))
    } else {
        Ok(format!(
            "Label saved: {} — set an AWB label printer in Settings → Printers to print on paper",
            path.display()
        ))
    }
}

/// Print the run sheet / DRS of the bag or run the person picked.
pub fn print_manifest_named(store: &Store, m: &manifest::Manifest) -> Result<String, String> {
    let _ = store.print_manifest(&m.id)?;
    let who = if m.kind == "own_fleet" {
        m.rider_name.as_str()
    } else {
        m.carrier_name.as_str()
    };
    if crate::print_share::prints_route(crate::print_share::RouteKey::Receipt) {
        Ok(format!("Run sheet printed · {who} · {} parcels", m.line_count))
    } else {
        Ok(format!(
            "Run sheet ready · {who} — set a receipt printer in Settings → Printers to print on paper"
        ))
    }
}

/// Print the latest carrier bag / rider DRS via the receipt route.
pub fn print_latest_run_sheet(store: &Store) -> Result<String, String> {
    let list = store.list_manifests(1)?;
    let m = list
        .into_iter()
        .next()
        .ok_or_else(|| "No manifests yet — dispatch or hand over first".to_string())?;
    let _ = store.print_manifest(&m.id)?;
    let who = if m.kind == "own_fleet" {
        m.rider_name
    } else {
        m.carrier_name
    };
    if crate::print_share::prints_route(crate::print_share::RouteKey::Receipt) {
        Ok(format!(
            "Run sheet / DRS printed · {} · {} AWBs → {}",
            m.kind, m.line_count, who
        ))
    } else {
        Ok(format!(
            "Run sheet ready · {} · {} AWBs → {} — set a receipt printer in Settings → Printers",
            m.kind, m.line_count, who
        ))
    }
}

/// Failed delivery (NDR). `reason` from note or NDR chip; optional YYYY-MM-DD in note after `|`.
pub fn record_failed_delivery(
    store: &Store,
    awb: &str,
    reason: &str,
    next_attempt_date: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    let sale = find_sale_for_scan(store, awb)?;
    let status = store.record_ndr(&sale.id, reason, next_attempt_date, actor)?;
    Ok(format!("{} → {} · {}", sale.awb_no, status.label(), reason.trim()))
}

pub fn reattempt_awb(
    store: &Store,
    awb: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    let sale = find_sale_for_scan(store, awb)?;
    store.reattempt_delivery(&sale.id, actor)?;
    Ok(format!("{} → Out for delivery (reattempt)", sale.awb_no))
}

pub fn settle_rider_cod_line(
    store: &Store,
    rider_name: &str,
    handed_in_text: &str,
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
        .ok_or_else(|| format!("No rider named {rider_name}"))?;
    // Same money parsing as every other amount field (the shop currency's
    // minor units and separators); a blank means nothing was handed in.
    let handed = if handed_in_text.trim().is_empty() {
        0
    } else {
        let currency = store.locale_settings()?.currency;
        crate::locale::parse_major_input(handed_in_text, &currency)?
    };
    let (expected, handed, _slip) = store.settle_rider_cod(&rider.id, handed, actor)?;
    Ok(format!(
        "Settled {} · expected {} · handed {} · variance {}",
        rider.name,
        store.money_display(expected),
        store.money_display(handed),
        store.money_display(handed.saturating_sub(expected))
    ))
}

/// Picked AWBs (typed, scanned or tapped on the Ops board) → their sale ids,
/// in the order picked. Refuses an empty pick: a run or bag carries exactly
/// what the person picked, never "everything booked".
fn picked_sale_ids(store: &Store, awbs: &[String]) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();
    for raw in awbs {
        let awb = normalize_awb_input(raw);
        if awb.is_empty() {
            continue;
        }
        ids.push(find_sale_for_scan(store, &awb)?.id);
    }
    if ids.is_empty() {
        return Err("Pick the parcels first — scan them or tap them on Ops".into());
    }
    Ok(ids)
}

/// One own-fleet run: the picked parcels go out with the named rider.
pub fn dispatch_to_rider(
    store: &Store,
    rider_name: &str,
    awbs: &[String],
    note: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    let rider_name = rider_name.trim();
    if rider_name.is_empty() {
        return Err("Pick the rider".into());
    }
    let rider = store
        .list_riders()?
        .into_iter()
        .find(|r| r.name.eq_ignore_ascii_case(rider_name))
        .ok_or_else(|| format!("No rider named {rider_name} — add them on Riders first"))?;
    let ids = picked_sale_ids(store, awbs)?;
    let m = store.create_manifest(HandoverKind::OwnFleet, "", &rider.id, &ids, note, actor)?;
    Ok(format!(
        "{} {} out with {}",
        m.line_count,
        if m.line_count == 1 { "parcel" } else { "parcels" },
        rider.name
    ))
}

/// One carrier bag: the picked parcels are handed to the named carrier.
pub fn handover_to_carrier(
    store: &Store,
    carrier_name: &str,
    awbs: &[String],
    note: &str,
    actor: &StaffMember,
) -> Result<String, String> {
    let carrier_name = carrier_name.trim();
    if carrier_name.is_empty() {
        return Err("Pick the carrier".into());
    }
    let party = store
        .list_customers()?
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(carrier_name))
        .ok_or_else(|| format!("No party named {carrier_name}"))?;
    let ids = picked_sale_ids(store, awbs)?;
    let m = store.create_manifest(HandoverKind::Carrier, &party.id, "", &ids, note, actor)?;
    Ok(format!(
        "{} {} handed to {}",
        m.line_count,
        if m.line_count == 1 { "parcel" } else { "parcels" },
        party.name
    ))
}

/// Run one tracking step on each picked AWB. Each parcel is its own unit:
/// one refused (already delivered, voided, …) never stops the others, and
/// the message names what was refused and why.
pub fn each_awb(
    store: &Store,
    awbs: &[String],
    done_label: &str,
    mut step: impl FnMut(&Sale) -> Result<(), String>,
) -> Result<String, String> {
    let mut ok = 0usize;
    let mut refused: Vec<String> = Vec::new();
    let mut picked = 0usize;
    for raw in awbs {
        let awb = normalize_awb_input(raw);
        if awb.is_empty() {
            continue;
        }
        picked += 1;
        let result = find_sale_for_scan(store, &awb)
            .and_then(|sale| store.in_transaction(|| step(&sale)));
        match result {
            Ok(()) => ok += 1,
            Err(e) => refused.push(e),
        }
    }
    if picked == 0 {
        return Err("Pick the parcels first — scan them or tap them on Ops".into());
    }
    if ok == 0 {
        return Err(refused.join("\n"));
    }
    let mut msg = format!("{done_label} · {ok}");
    if !refused.is_empty() {
        msg.push_str(&format!(" · {} refused", refused.len()));
        for e in refused.iter().take(3) {
            msg.push('\n');
            msg.push_str(e);
        }
    }
    Ok(msg)
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

pub fn remit_collected_for_party(store: &Store, party_name: &str) -> Result<String, String> {
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
    store: &Store,
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
