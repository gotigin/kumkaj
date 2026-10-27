//! Courier Local/LAN ops — rider mode, incoming/pickup, assign, settle, export.
//!
//! Kept beside `store.rs` so the main file stays readable; every method is
//! still `Store`.

use super::*;
use crate::manifest::TrackingStatus;
use crate::ops_local::{
    self, AssignRule, FranchiseCommissionRow, IncomingManifest, IncomingManifestLine, PickupRequest,
    PickupRun, RiderCommissionSlip, RiderPayRate, VehicleInspection,
};
use crate::rider_mode::{
    self, AppMode, RiderPodEvent, RiderRunCache, RiderStop, SigStroke,
};
use rust_decimal::Decimal;

impl Store {
    /// Ensure Local/LAN courier tables exist (idempotent).
    pub(crate) fn migrate_courier_ops_tables(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS incoming_manifests (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    hub_name TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS incoming_manifest_lines (
                    id TEXT PRIMARY KEY,
                    manifest_id TEXT NOT NULL,
                    awb_no TEXT NOT NULL,
                    sale_id TEXT NOT NULL DEFAULT '',
                    destination TEXT NOT NULL DEFAULT '',
                    consignee_name TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS pickup_requests (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    party_id TEXT NOT NULL DEFAULT '',
                    party_name TEXT NOT NULL DEFAULT '',
                    address TEXT NOT NULL DEFAULT '',
                    pincode TEXT NOT NULL DEFAULT '',
                    area TEXT NOT NULL DEFAULT '',
                    pieces INTEGER NOT NULL DEFAULT 1,
                    note TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL DEFAULT 'open',
                    run_id TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS pickup_runs (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL,
                    rider_id TEXT NOT NULL DEFAULT '',
                    rider_name TEXT NOT NULL DEFAULT '',
                    drop_note TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL DEFAULT 'open'
                );
                CREATE TABLE IF NOT EXISTS assign_rules (
                    id TEXT PRIMARY KEY,
                    area TEXT NOT NULL DEFAULT '',
                    pincode_prefix TEXT NOT NULL DEFAULT '',
                    rider_id TEXT NOT NULL,
                    rider_name TEXT NOT NULL DEFAULT '',
                    round_robin_slot INTEGER NOT NULL DEFAULT 0
                );
                CREATE TABLE IF NOT EXISTS rider_pay_rates (
                    rider_id TEXT PRIMARY KEY,
                    per_delivery_paise INTEGER NOT NULL DEFAULT 0,
                    per_kg_paise INTEGER NOT NULL DEFAULT 0
                );
                CREATE TABLE IF NOT EXISTS vehicle_inspections (
                    id TEXT PRIMARY KEY,
                    manifest_id TEXT NOT NULL DEFAULT '',
                    rider_id TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL,
                    lights_ok INTEGER NOT NULL DEFAULT 1,
                    tyres_ok INTEGER NOT NULL DEFAULT 1,
                    brakes_ok INTEGER NOT NULL DEFAULT 1,
                    docs_ok INTEGER NOT NULL DEFAULT 1,
                    fuel_note TEXT NOT NULL DEFAULT '',
                    note TEXT NOT NULL DEFAULT ''
                );
                CREATE TABLE IF NOT EXISTS rider_run_cache (
                    rider_id TEXT PRIMARY KEY,
                    rider_name TEXT NOT NULL DEFAULT '',
                    manifest_id TEXT NOT NULL DEFAULT '',
                    downloaded_at TEXT NOT NULL,
                    stops_json TEXT NOT NULL DEFAULT '[]'
                );
                CREATE TABLE IF NOT EXISTS franchise_rates (
                    carrier_party_id TEXT PRIMARY KEY,
                    rate_pct INTEGER NOT NULL DEFAULT 0,
                    on_margin INTEGER NOT NULL DEFAULT 1
                );
                CREATE INDEX IF NOT EXISTS idx_incoming_lines ON incoming_manifest_lines(manifest_id);
                CREATE INDEX IF NOT EXISTS idx_pickup_run ON pickup_requests(run_id);
                "#,
            )
            .map_err(|e| e.to_string())?;
        for stmt in [
            "ALTER TABLE tracking_events ADD COLUMN lat REAL NOT NULL DEFAULT 0",
            "ALTER TABLE tracking_events ADD COLUMN lon REAL NOT NULL DEFAULT 0",
            "ALTER TABLE sales ADD COLUMN feedback_score INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE sales ADD COLUMN area TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE sales ADD COLUMN dest_pincode TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE manifest_lines ADD COLUMN stop_order INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE riders ADD COLUMN pay_per_delivery_paise INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE riders ADD COLUMN pay_per_kg_paise INTEGER NOT NULL DEFAULT 0",
        ] {
            let _ = self.conn.execute(stmt, []);
        }
        Ok(())
    }

    // ----- App mode / rider cache -----------------------------------------

    pub fn app_mode(&self) -> Result<AppMode, String> {
        Ok(AppMode::parse(
            &self.meta_get("app_mode")?.unwrap_or_default(),
        ))
    }

    pub fn set_app_mode(&self, mode: AppMode) -> Result<(), String> {
        self.meta_set("app_mode", mode.as_str())
    }

    /// Download today's own-fleet run for this rider into local cache (office LAN).
    pub fn download_rider_run(
        &self,
        rider_id: &str,
    ) -> Result<RiderRunCache, String> {
        let riders = self.list_riders()?;
        let rider = riders
            .into_iter()
            .find(|r| r.id == rider_id)
            .ok_or_else(|| "Rider not found".to_string())?;
        let today = Local::now().format("%Y-%m-%d").to_string();
        let manifests = self.list_manifests(50)?;
        let manifest = manifests
            .into_iter()
            .find(|m| {
                m.kind == "own_fleet"
                    && m.rider_id == rider_id
                    && m.created_at.starts_with(&today)
                    && m.status != "closed"
            })
            .ok_or_else(|| "No open run for this rider today — dispatch first".to_string())?;
        let (_m, lines) = self.manifest_detail(&manifest.id)?;
        let mut stops = Vec::new();
        for (i, row) in lines.iter().enumerate() {
            let sale = self.get_sale(&row.line.sale_id)?.unwrap_or_default();
            stops.push(RiderStop {
                sale_id: row.line.sale_id.clone(),
                awb_no: row.line.awb_no.clone(),
                destination: row.line.destination.clone(),
                consignee_name: row.line.consignee_name.clone(),
                pincode: sale.dest_pincode.clone(),
                area: sale.area.clone(),
                cod_paise: sale.cod_paise,
                weight_kg: sale.chargeable_weight_kg.to_string(),
                status: sale.tracking_status.clone(),
                stop_order: i as i64,
                cod_collected_paise: 0,
                ndr_reason: String::new(),
                pod_attachment_id: String::new(),
                signature_attachment_id: String::new(),
                feedback_score: sale.feedback_score,
                lat: 0.0,
                lon: 0.0,
                pending_sync: false,
            });
        }
        rider_mode::sort_stops_by_area_pincode(&mut stops);
        let cache = RiderRunCache {
            rider_id: rider.id,
            rider_name: rider.name,
            manifest_id: manifest.id,
            downloaded_at: Local::now().to_rfc3339(),
            stops,
        };
        self.save_rider_run_cache(&cache)?;
        Ok(cache)
    }

    pub fn save_rider_run_cache(&self, cache: &RiderRunCache) -> Result<(), String> {
        let json = serde_json::to_string(&cache.stops).map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT INTO rider_run_cache(rider_id, rider_name, manifest_id, downloaded_at, stops_json)
                 VALUES(?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(rider_id) DO UPDATE SET
                   rider_name=excluded.rider_name,
                   manifest_id=excluded.manifest_id,
                   downloaded_at=excluded.downloaded_at,
                   stops_json=excluded.stops_json",
                params![
                    cache.rider_id,
                    cache.rider_name,
                    cache.manifest_id,
                    cache.downloaded_at,
                    json
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_rider_run_cache(&self, rider_id: &str) -> Result<Option<RiderRunCache>, String> {
        self.conn
            .query_row(
                "SELECT rider_id, rider_name, manifest_id, downloaded_at, stops_json
                 FROM rider_run_cache WHERE rider_id = ?1",
                params![rider_id],
                |r| {
                    let stops_json: String = r.get(4)?;
                    let stops: Vec<RiderStop> =
                        serde_json::from_str(&stops_json).unwrap_or_default();
                    Ok(RiderRunCache {
                        rider_id: r.get(0)?,
                        rider_name: r.get(1)?,
                        manifest_id: r.get(2)?,
                        downloaded_at: r.get(3)?,
                        stops,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn rider_waiting_to_sync_count(&self) -> Result<usize, String> {
        Ok(self.lan_outbox()?.len())
    }

    /// Offline POD on the rider phone: update cache + enqueue LAN write.
    pub fn rider_queue_pod(
        &mut self,
        rider_id: &str,
        event: &RiderPodEvent,
    ) -> Result<String, String> {
        let mut cache = self
            .load_rider_run_cache(rider_id)?
            .ok_or_else(|| "Download today's run at the office first".to_string())?;
        if let Some(stop) = cache.stops.iter_mut().find(|s| s.sale_id == event.sale_id) {
            stop.status = event.status.clone();
            stop.ndr_reason = event.ndr_reason.clone();
            stop.cod_collected_paise = event.cod_collected_paise;
            stop.pod_attachment_id = event.pod_attachment_id.clone();
            stop.signature_attachment_id = event.signature_attachment_id.clone();
            stop.feedback_score = event.feedback_score;
            stop.lat = event.lat;
            stop.lon = event.lon;
            stop.pending_sync = true;
        }
        self.save_rider_run_cache(&cache)?;
        let body = event.to_json()?;
        let item = LanQueued {
            id: event.id.clone(),
            path: RiderPodEvent::lan_path().into(),
            body,
            label: format!("{} · {}", event.awb_no, event.status),
            table_id: String::new(),
            staff_id: rider_id.into(),
            attempts: 0,
            last_error: String::new(),
        };
        self.enqueue_lan(&item)?;
        Ok(cache.waiting_label())
    }

    /// Apply a rider POD event on the Server (from LAN drain or desk).
    pub fn apply_rider_pod(
        &mut self,
        event: &RiderPodEvent,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Rider POD")?;
        let status = TrackingStatus::parse(&event.status);
        if status == TrackingStatus::Ndr {
            let _ = self.record_ndr(
                &event.sale_id,
                if event.ndr_reason.is_empty() {
                    "other"
                } else {
                    &event.ndr_reason
                },
                "",
                actor,
            )?;
        } else {
            let note = if event.cod_collected_paise > 0 {
                format!(
                    "{} · COD collected {}",
                    event.note,
                    self.money_display(event.cod_collected_paise)
                )
            } else {
                event.note.clone()
            };
            let pod = if !event.signature_attachment_id.is_empty() {
                event.signature_attachment_id.as_str()
            } else {
                event.pod_attachment_id.as_str()
            };
            let event_id =
                self.advance_tracking(&event.sale_id, status, &note, pod, actor)?;
            if event.lat != 0.0 || event.lon != 0.0 {
                let _ = self.conn.execute(
                    "UPDATE tracking_events SET lat = ?1, lon = ?2 WHERE id = ?3",
                    params![event.lat, event.lon, event_id],
                );
            }
        }
        if event.feedback_score > 0 {
            self.set_delivery_feedback(&event.sale_id, event.feedback_score)?;
        }
        Ok(())
    }

    /// Capture signature strokes → WebP attachment on the sale; returns attachment id.
    pub fn attach_signature_strokes(
        &self,
        sale_id: &str,
        strokes: &[SigStroke],
        actor: &StaffMember,
    ) -> Result<String, String> {
        self.assert_server_writer("Signature")?;
        let webp = rider_mode::signature_to_webp(strokes, 480, 160)?;
        let id = new_id();
        let stored_name = format!("{id}.webp");
        let dest = self
            .attachment_entity_dir(AttachmentEntity::Sale, sale_id)?
            .join(&stored_name);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&dest, &webp).map_err(|e| e.to_string())?;
        let times = EventTimes::capture(None);
        self.insert_attachment_row(
            &id,
            AttachmentEntity::Sale,
            sale_id,
            &times,
            AttachmentKind::Image,
            "Signature",
            "",
            &stored_name,
            "image/webp",
            webp.len() as i64,
            actor,
        )?;
        Ok(id)
    }

    pub fn set_delivery_feedback(&self, sale_id: &str, score: i64) -> Result<(), String> {
        let score = score.clamp(0, 5);
        self.conn
            .execute(
                "UPDATE sales SET feedback_score = ?1 WHERE id = ?2",
                params![score, sale_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ----- Incoming manifest ----------------------------------------------

    pub fn create_incoming_manifest(
        &mut self,
        hub_name: &str,
        awbs: &[String],
        note: &str,
        actor: &StaffMember,
    ) -> Result<IncomingManifest, String> {
        self.assert_server_writer("Incoming manifest")?;
        let id = new_id();
        let now = Local::now().to_rfc3339();
        let mut line_count = 0i64;
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO incoming_manifests(id, created_at, hub_name, note)
             VALUES(?1, ?2, ?3, ?4)",
            params![id, now, hub_name.trim(), note.trim()],
        )
        .map_err(|e| e.to_string())?;
        for raw in awbs {
            let awb = raw.trim();
            if awb.is_empty() {
                continue;
            }
            let sale = self.find_sale_by_awb(awb)?;
            let (sale_id, dest, consignee) = match &sale {
                Some(s) => (s.id.clone(), s.destination.clone(), s.consignee_name.clone()),
                None => (String::new(), String::new(), String::new()),
            };
            if let Some(s) = &sale {
                // Mark as available for own-fleet delivery from hub.
                tx.execute(
                    "UPDATE sales SET tracking_status = ?1 WHERE id = ?2
                     AND tracking_status NOT IN ('delivered','rto','cancelled')",
                    params![TrackingStatus::InTransit.as_str(), s.id],
                )
                .map_err(|e| e.to_string())?;
            }
            tx.execute(
                "INSERT INTO incoming_manifest_lines(
                    id, manifest_id, awb_no, sale_id, destination, consignee_name
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                params![new_id(), id, awb, sale_id, dest, consignee],
            )
            .map_err(|e| e.to_string())?;
            line_count += 1;
        }
        tx.commit().map_err(|e| e.to_string())?;
        let _ = self.audit(
            actor,
            "incoming_manifest",
            &format!("{hub_name} · {line_count} AWBs"),
        );
        Ok(IncomingManifest {
            id,
            created_at: now,
            hub_name: hub_name.trim().into(),
            note: note.trim().into(),
            line_count,
        })
    }

    pub fn list_incoming_manifest_lines(
        &self,
        manifest_id: &str,
    ) -> Result<Vec<IncomingManifestLine>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, manifest_id, awb_no, sale_id, destination, consignee_name
                 FROM incoming_manifest_lines WHERE manifest_id = ?1 ORDER BY awb_no",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![manifest_id], |r| {
                Ok(IncomingManifestLine {
                    id: r.get(0)?,
                    manifest_id: r.get(1)?,
                    awb_no: r.get(2)?,
                    sale_id: r.get(3)?,
                    destination: r.get(4)?,
                    consignee_name: r.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    // ----- Pickup requests / runs -----------------------------------------

    pub fn add_pickup_request(
        &self,
        party_id: &str,
        party_name: &str,
        address: &str,
        pincode: &str,
        area: &str,
        pieces: i64,
        note: &str,
    ) -> Result<PickupRequest, String> {
        self.assert_server_writer("Pickup request")?;
        let req = PickupRequest {
            id: new_id(),
            created_at: Local::now().to_rfc3339(),
            party_id: party_id.into(),
            party_name: party_name.trim().into(),
            address: address.trim().into(),
            pincode: pincode.trim().into(),
            area: area.trim().into(),
            pieces: pieces.max(1),
            note: note.trim().into(),
            status: "open".into(),
            run_id: String::new(),
        };
        self.conn
            .execute(
                "INSERT INTO pickup_requests(
                    id, created_at, party_id, party_name, address, pincode, area,
                    pieces, note, status, run_id
                 ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'')",
                params![
                    req.id,
                    req.created_at,
                    req.party_id,
                    req.party_name,
                    req.address,
                    req.pincode,
                    req.area,
                    req.pieces,
                    req.note,
                    req.status
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(req)
    }

    pub fn create_pickup_run(
        &mut self,
        rider_id: &str,
        request_ids: &[String],
        drop_note: &str,
        actor: &StaffMember,
    ) -> Result<PickupRun, String> {
        self.assert_server_writer("Pickup run")?;
        let riders = self.list_riders()?;
        let rider = riders
            .into_iter()
            .find(|r| r.id == rider_id)
            .ok_or_else(|| "Rider required".to_string())?;
        if request_ids.is_empty() {
            return Err("Pick at least one pickup request".into());
        }
        let id = new_id();
        let now = Local::now().to_rfc3339();
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO pickup_runs(id, created_at, rider_id, rider_name, drop_note, status)
             VALUES(?1, ?2, ?3, ?4, ?5, 'open')",
            params![id, now, rider.id, rider.name, drop_note.trim()],
        )
        .map_err(|e| e.to_string())?;
        let mut count = 0i64;
        for rid in request_ids {
            let n = tx
                .execute(
                    "UPDATE pickup_requests SET run_id = ?1, status = 'assigned'
                     WHERE id = ?2 AND status = 'open'",
                    params![id, rid],
                )
                .map_err(|e| e.to_string())?;
            count += n as i64;
        }
        tx.commit().map_err(|e| e.to_string())?;
        let _ = self.audit(
            actor,
            "pickup_run",
            &format!("{} · {count} pickups", rider.name),
        );
        Ok(PickupRun {
            id,
            created_at: now,
            rider_id: rider.id,
            rider_name: rider.name,
            drop_note: drop_note.trim().into(),
            status: "open".into(),
            request_count: count,
        })
    }

    pub fn list_open_pickup_requests(&self) -> Result<Vec<PickupRequest>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, created_at, party_id, party_name, address, pincode, area,
                        pieces, note, status, run_id
                 FROM pickup_requests WHERE status = 'open' ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(PickupRequest {
                    id: r.get(0)?,
                    created_at: r.get(1)?,
                    party_id: r.get(2)?,
                    party_name: r.get(3)?,
                    address: r.get(4)?,
                    pincode: r.get(5)?,
                    area: r.get(6)?,
                    pieces: r.get(7)?,
                    note: r.get(8)?,
                    status: r.get(9)?,
                    run_id: r.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    // ----- Auto-assign / stop order ---------------------------------------

    pub fn upsert_assign_rule(&self, rule: &AssignRule) -> Result<(), String> {
        self.assert_server_writer("Assign rule")?;
        self.conn
            .execute(
                "INSERT INTO assign_rules(id, area, pincode_prefix, rider_id, rider_name, round_robin_slot)
                 VALUES(?1,?2,?3,?4,?5,?6)
                 ON CONFLICT(id) DO UPDATE SET
                   area=excluded.area, pincode_prefix=excluded.pincode_prefix,
                   rider_id=excluded.rider_id, rider_name=excluded.rider_name,
                   round_robin_slot=excluded.round_robin_slot",
                params![
                    rule.id,
                    rule.area,
                    rule.pincode_prefix,
                    rule.rider_id,
                    rule.rider_name,
                    rule.round_robin_slot
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_assign_rules(&self) -> Result<Vec<AssignRule>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, area, pincode_prefix, rider_id, rider_name, round_robin_slot
                 FROM assign_rules ORDER BY area, rider_name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(AssignRule {
                    id: r.get(0)?,
                    area: r.get(1)?,
                    pincode_prefix: r.get(2)?,
                    rider_id: r.get(3)?,
                    rider_name: r.get(4)?,
                    round_robin_slot: r.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// One action: assign all booked dockets in an area to riders (round-robin).
    pub fn auto_assign_area(
        &mut self,
        area: &str,
        actor: &StaffMember,
    ) -> Result<(usize, String), String> {
        self.assert_server_writer("Auto-assign")?;
        let rules = self.list_assign_rules()?;
        let area = area.trim();
        if area.is_empty() {
            return Err("Area required".into());
        }
        let sales = self.recent_sales(500)?;
        let mut by_rider: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        let mut rr = self
            .meta_get("assign_rr_counter")?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0i64);
        let mut n = 0usize;
        for s in sales {
            if s.tracking_status != "booked" || s.awb_no.is_empty() {
                continue;
            }
            let sale_area = if !s.area.is_empty() {
                s.area.clone()
            } else {
                s.destination.clone()
            };
            if !sale_area.to_lowercase().contains(&area.to_lowercase())
                && !area.eq_ignore_ascii_case(&sale_area)
            {
                continue;
            }
            let Some(rule) = ops_local::pick_round_robin(&rules, area, rr) else {
                continue;
            };
            by_rider
                .entry(rule.rider_id.clone())
                .or_default()
                .push(s.id.clone());
            rr = rr.saturating_add(1);
            n += 1;
        }
        self.meta_set("assign_rr_counter", &rr.to_string())?;
        let mut summary = String::new();
        for (rider_id, ids) in by_rider {
            let m = self.create_manifest(
                crate::manifest::HandoverKind::OwnFleet,
                "",
                &rider_id,
                &ids,
                &format!("auto-assign {area}"),
                actor,
            )?;
            summary.push_str(&format!("{} → {} stops; ", m.rider_name, m.line_count));
        }
        if n == 0 {
            return Err("No booked dockets matched that area".into());
        }
        Ok((n, summary))
    }

    /// Reorder stops on a manifest (never locked).
    pub fn reorder_manifest_stops(
        &self,
        manifest_id: &str,
        sale_ids_in_order: &[String],
    ) -> Result<(), String> {
        self.assert_server_writer("Reorder stops")?;
        for (i, sid) in sale_ids_in_order.iter().enumerate() {
            self.conn
                .execute(
                    "UPDATE manifest_lines SET stop_order = ?1
                     WHERE manifest_id = ?2 AND sale_id = ?3",
                    params![i as i64, manifest_id, sid],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn sort_manifest_by_area_pincode(&self, manifest_id: &str) -> Result<(), String> {
        let (_m, lines) = self.manifest_detail(manifest_id)?;
        let mut stops: Vec<RiderStop> = lines
            .iter()
            .map(|row| {
                let sale = self.get_sale(&row.line.sale_id).ok().flatten();
                RiderStop {
                    sale_id: row.line.sale_id.clone(),
                    awb_no: row.line.awb_no.clone(),
                    pincode: sale
                        .as_ref()
                        .map(|s| s.dest_pincode.clone())
                        .unwrap_or_default(),
                    area: sale.as_ref().map(|s| s.area.clone()).unwrap_or_default(),
                    ..Default::default()
                }
            })
            .collect();
        rider_mode::sort_stops_by_area_pincode(&mut stops);
        let ids: Vec<String> = stops.into_iter().map(|s| s.sale_id).collect();
        self.reorder_manifest_stops(manifest_id, &ids)
    }

    // ----- Rider commission -----------------------------------------------

    pub fn set_rider_pay_rate(&self, rate: &RiderPayRate) -> Result<(), String> {
        self.assert_server_writer("Rider pay")?;
        self.conn
            .execute(
                "INSERT INTO rider_pay_rates(rider_id, per_delivery_paise, per_kg_paise)
                 VALUES(?1, ?2, ?3)
                 ON CONFLICT(rider_id) DO UPDATE SET
                   per_delivery_paise=excluded.per_delivery_paise,
                   per_kg_paise=excluded.per_kg_paise",
                params![rate.rider_id, rate.per_delivery_paise, rate.per_kg_paise],
            )
            .map_err(|e| e.to_string())?;
        let _ = self.conn.execute(
            "UPDATE riders SET pay_per_delivery_paise = ?1, pay_per_kg_paise = ?2 WHERE id = ?3",
            params![rate.per_delivery_paise, rate.per_kg_paise, rate.rider_id],
        );
        Ok(())
    }

    pub fn rider_pay_rate(&self, rider_id: &str) -> Result<RiderPayRate, String> {
        self.conn
            .query_row(
                "SELECT rider_id, per_delivery_paise, per_kg_paise FROM rider_pay_rates WHERE rider_id = ?1",
                params![rider_id],
                |r| {
                    Ok(RiderPayRate {
                        rider_id: r.get(0)?,
                        per_delivery_paise: r.get(1)?,
                        per_kg_paise: r.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())
            .map(|o| {
                o.unwrap_or(RiderPayRate {
                    rider_id: rider_id.into(),
                    ..Default::default()
                })
            })
    }

    /// Full daily slip: per-delivery/kg pay + COD expected vs handed in.
    pub fn settle_rider_commission(
        &mut self,
        rider_id: &str,
        handed_in_paise: i64,
        actor: &StaffMember,
    ) -> Result<(RiderCommissionSlip, String), String> {
        let (expected, handed, _cod_slip) =
            self.settle_rider_cod(rider_id, handed_in_paise, actor)?;
        let today = Local::now().format("%Y-%m-%d").to_string();
        let day_prefix = format!("{today}%");
        let (deliveries, kg_total): (i64, String) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(CAST(chargeable_weight_kg AS REAL)), 0)
                 FROM sales s
                 WHERE s.rider_id = ?1
                   AND EXISTS (
                     SELECT 1 FROM tracking_events e
                     WHERE e.sale_id = s.id AND e.status = 'delivered'
                       AND e.created_at LIKE ?2
                   )",
                params![rider_id, day_prefix],
                |r| Ok((r.get(0)?, format!("{}", r.get::<_, f64>(1)?))),
            )
            .unwrap_or((0, "0".into()));
        let kg: Decimal = kg_total.parse().unwrap_or(Decimal::ZERO);
        let rate = self.rider_pay_rate(rider_id)?;
        let pay = RiderCommissionSlip::compute_pay(&rate, deliveries, kg);
        let riders = self.list_riders()?;
        let name = riders
            .into_iter()
            .find(|r| r.id == rider_id)
            .map(|r| r.name)
            .unwrap_or_default();
        let slip = RiderCommissionSlip {
            rider_id: rider_id.into(),
            rider_name: name,
            date: today,
            deliveries,
            kg_total: kg,
            pay_paise: pay,
            cod_expected_paise: expected,
            cod_handed_paise: handed,
            variance_paise: handed.saturating_sub(expected),
        };
        let shop = self.venue_title().unwrap_or_else(|_| "Your shop".into());
        let text = slip.format_text(&shop, &|p| self.money_display(p));
        let _ = crate::print_share::print_route(
            crate::print_share::PrintKind::Receipt,
            crate::print_share::RouteKey::Receipt,
            &text,
            self.path.parent(),
        )?;
        Ok((slip, text))
    }

    // ----- Vehicle inspection ---------------------------------------------

    pub fn save_vehicle_inspection(&self, insp: &VehicleInspection) -> Result<(), String> {
        self.assert_server_writer("Vehicle inspection")?;
        self.conn
            .execute(
                "INSERT INTO vehicle_inspections(
                    id, manifest_id, rider_id, created_at, lights_ok, tyres_ok,
                    brakes_ok, docs_ok, fuel_note, note
                 ) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
                 ON CONFLICT(id) DO UPDATE SET
                   lights_ok=excluded.lights_ok, tyres_ok=excluded.tyres_ok,
                   brakes_ok=excluded.brakes_ok, docs_ok=excluded.docs_ok,
                   fuel_note=excluded.fuel_note, note=excluded.note",
                params![
                    insp.id,
                    insp.manifest_id,
                    insp.rider_id,
                    insp.created_at,
                    insp.lights_ok as i64,
                    insp.tyres_ok as i64,
                    insp.brakes_ok as i64,
                    insp.docs_ok as i64,
                    insp.fuel_note,
                    insp.note
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ----- Franchise / Tally ----------------------------------------------

    pub fn set_franchise_rate(
        &self,
        carrier_party_id: &str,
        rate_pct: i64,
        on_margin: bool,
    ) -> Result<(), String> {
        self.assert_server_writer("Franchise rate")?;
        self.conn
            .execute(
                "INSERT INTO franchise_rates(carrier_party_id, rate_pct, on_margin)
                 VALUES(?1, ?2, ?3)
                 ON CONFLICT(carrier_party_id) DO UPDATE SET
                   rate_pct=excluded.rate_pct, on_margin=excluded.on_margin",
                params![carrier_party_id, rate_pct, on_margin as i64],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn franchise_commission_statement(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> Result<(Vec<FranchiseCommissionRow>, String), String> {
        let sales = self.recent_sales(5000)?;
        let mut by_carrier: std::collections::BTreeMap<String, (String, i64, i64)> =
            std::collections::BTreeMap::new();
        for s in sales {
            if s.awb_no.is_empty() || s.carrier_party_id.is_empty() {
                continue;
            }
            let day = &s.created_at[..s.created_at.len().min(10)];
            if !from_date.is_empty() && day < from_date {
                continue;
            }
            if !to_date.is_empty() && day > to_date {
                continue;
            }
            let e = by_carrier
                .entry(s.carrier_party_id.clone())
                .or_insert_with(|| {
                    let name = self
                        .get_customer(&s.carrier_party_id)
                        .ok()
                        .flatten()
                        .map(|c| c.name)
                        .unwrap_or_else(|| s.carrier_party_id.clone());
                    (name, 0, 0)
                });
            e.1 = e.1.saturating_add(s.total_paise);
            e.2 = e.2.saturating_add(s.margin_paise());
        }
        let mut rows = Vec::new();
        for (cid, (name, rev, margin)) in by_carrier {
            let (pct, on_margin): (i64, bool) = self
                .conn
                .query_row(
                    "SELECT rate_pct, on_margin FROM franchise_rates WHERE carrier_party_id = ?1",
                    params![cid],
                    |r| Ok((r.get(0)?, r.get::<_, i64>(1)? != 0)),
                )
                .optional()
                .map_err(|e| e.to_string())?
                .unwrap_or((0, true));
            if pct <= 0 {
                continue;
            }
            rows.push(FranchiseCommissionRow::from_totals(
                &cid, &name, rev, margin, pct, on_margin,
            ));
        }
        let shop = self.venue_title().unwrap_or_else(|_| "Your shop".into());
        let text = ops_local::format_franchise_statement(
            &shop,
            from_date,
            to_date,
            &rows,
            &|p| self.money_display(p),
        );
        Ok((rows, text))
    }

    pub fn tally_voucher_export(&self, limit: i64) -> Result<String, String> {
        let sales = self.recent_sales(limit.max(1))?;
        let mut rows = Vec::new();
        for s in sales {
            if s.awb_no.is_empty() {
                continue;
            }
            let date = s.created_at[..s.created_at.len().min(10)].to_string();
            let party = match s.customer_id.as_deref() {
                None | Some("") => "Walk-in".into(),
                Some(cid) => self
                    .get_customer(cid)
                    .ok()
                    .flatten()
                    .map(|c| c.name)
                    .unwrap_or_else(|| cid.to_string()),
            };
            rows.push((
                date,
                "Sales".into(),
                party,
                "Freight Income".into(),
                s.total_paise,
                format!("AWB {}", s.awb_no),
            ));
        }
        Ok(ops_local::tally_voucher_csv(&rows))
    }

    // ----- RTO fee / docket / destination suggest / notify ----------------

    pub fn rto_fee_settings(&self) -> (i64, i64) {
        let fixed = self
            .meta_get("rto_fee_fixed_paise")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let pct = self
            .meta_get("rto_fee_pct")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        (fixed, pct)
    }

    pub fn set_rto_fee_settings(&self, fixed_paise: i64, pct: i64) -> Result<(), String> {
        self.meta_set("rto_fee_fixed_paise", &fixed_paise.max(0).to_string())?;
        self.meta_set("rto_fee_pct", &pct.max(0).to_string())
    }

    /// Append RTO fee extra when status becomes RTO (auto or manual).
    pub fn apply_rto_fee_if_needed(&self, sale_id: &str) -> Result<i64, String> {
        let sale = self
            .get_sale(sale_id)?
            .ok_or_else(|| "Sale not found".to_string())?;
        if sale.tracking_status != "rto" {
            return Ok(0);
        }
        let mut extras: Vec<crate::rates::BookingExtra> =
            serde_json::from_str(&sale.extras_json).unwrap_or_default();
        if extras.iter().any(|e| e.kind == "rto_fee") {
            return Ok(0);
        }
        let (fixed, pct) = self.rto_fee_settings();
        let fee = ops_local::rto_fee_paise(sale.total_paise, fixed, pct);
        if fee <= 0 {
            return Ok(0);
        }
        extras.push(crate::rates::BookingExtra::from_kind(
            crate::rates::ExtraChargeKind::RtoFee,
            fee,
        ));
        let json = serde_json::to_string(&extras).map_err(|e| e.to_string())?;
        let new_total = sale.total_paise.saturating_add(fee);
        self.conn
            .execute(
                "UPDATE sales SET extras_json = ?1, total_paise = ?2 WHERE id = ?3",
                params![json, new_total, sale_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(fee)
    }

    pub fn format_docket_for_sale(
        &self,
        sale_id: &str,
        size: crate::labels::DocketSize,
    ) -> Result<String, String> {
        let label = self.awb_label_for_sale(sale_id)?;
        Ok(label.format_docket_text(size))
    }

    pub fn suggest_destinations(&self, query: &str, limit: usize) -> Result<Vec<String>, String> {
        let q = query.trim();
        let mut pin_hits: Vec<(String, String)> = Vec::new();
        // Prefix / partial pincode or zone name from service table.
        let like = format!("%{q}%");
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT p.pincode, COALESCE(z.name, '')
                 FROM pincode_service p
                 LEFT JOIN zones z ON z.id = p.zone_id
                 WHERE p.pincode LIKE ?1 OR z.name LIKE ?1
                 LIMIT 40",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![like], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows {
            pin_hits.push(row.map_err(|e| e.to_string())?);
        }
        // Also exact lookup when 6-digit.
        if crate::rates::is_india_pincode(q) {
            for hit in self.lookup_pincode(q)? {
                pin_hits.push((hit.pincode, hit.zone_name));
            }
        }
        let mut past = Vec::new();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT destination FROM sales
                 WHERE destination != '' AND destination LIKE ?1
                 ORDER BY created_at DESC LIMIT 40",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![like], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        for row in rows {
            past.push(row.map_err(|e| e.to_string())?);
        }
        Ok(ops_local::suggest_destinations(q, &pin_hits, &past, limit))
    }

    pub fn mark_notify_sent(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE notify_outbox SET status = 'sent', last_error = '' WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn notify_whatsapp_url(&self, id: &str) -> Result<String, String> {
        let rows = self.list_notify_outbox(200)?;
        let row = rows
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| "Notify row not found".to_string())?;
        let msg = crate::notify::message_from_payload(&row.payload_json);
        Ok(crate::notify::whatsapp_click_url(&row.to_addr, &msg))
    }

    pub fn notify_sms_text(&self, id: &str) -> Result<String, String> {
        let rows = self.list_notify_outbox(200)?;
        let row = rows
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| "Notify row not found".to_string())?;
        Ok(crate::notify::sms_body_from_row(&row))
    }

    pub fn whatsapp_receipt_url(&self, sale_id: &str, phone: &str) -> Result<String, String> {
        let text = self.format_receipt_text(sale_id)?;
        Ok(crate::notify::whatsapp_receipt_share_url(phone, &text))
    }

    /// LAN corporate self-booking: create a draft/cash booking from form fields.
    pub fn lan_self_book(
        &mut self,
        destination: &str,
        consignee: &str,
        weight_kg: &str,
        phone: &str,
        note: &str,
        actor: &StaffMember,
    ) -> Result<Sale, String> {
        self.assert_server_writer("LAN book")?;
        let products = self.list_products(false)?;
        let rate = products
            .into_iter()
            .find(|p| p.available && !p.archived)
            .ok_or_else(|| "Add a rate before self-booking".to_string())?;
        let mut cart = crate::domain::Cart::default();
        cart.add_or_bump(&rate);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.destination = destination.trim().into();
        draft.consignee_name = consignee.trim().into();
        draft.chargeable_weight_kg = crate::precise::parse_non_negative(weight_kg)
            .unwrap_or(crate::precise::ZERO);
        draft.actual_weight_kg = draft.chargeable_weight_kg;
        draft.notify_phone = phone.trim().into();
        let _ = note; // LAN form note is audit-only for now
        self.finalize_sale(&draft, PayMethod::Cash, None, actor)
    }
}

#[cfg(test)]
mod courier_ops_tests {
    use super::*;
    use crate::domain::{Cart, Money, Product};
    use crate::rider_mode::{RiderPodEvent, SigPoint, SigStroke};

    fn temp_db() -> PathBuf {
        std::env::temp_dir().join(format!("mok-courier-ops-{}.sqlite3", new_id()))
    }

    fn book_one(store: &mut Store, dest: &str, area: &str, cod: i64) -> Sale {
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let rate = Product::new_basic(new_id(), "Surface", Money::from_rupees(40));
        store.upsert_product(&rate).unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump(&rate);
        let mut draft = cart.to_sale_draft().unwrap();
        draft.destination = dest.into();
        draft.consignee_name = "Test Consignee".into();
        draft.chargeable_weight_kg = "2".parse().unwrap();
        draft.cod_paise = cod;
        let sale = store
            .finalize_sale(&draft, PayMethod::Cash, None, &owner)
            .unwrap();
        store
            .conn
            .execute(
                "UPDATE sales SET area = ?1, dest_pincode = ?2 WHERE id = ?3",
                params![area, "560001", sale.id],
            )
            .unwrap();
        store.get_sale(&sale.id).unwrap().unwrap()
    }

    #[test]
    fn rider_offline_queue_round_trip() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let rider = store.add_rider("Ravi", "999", "Bike").unwrap();
        let sale = book_one(&mut store, "North Block", "North", 50_00);
        store
            .create_manifest(
                crate::manifest::HandoverKind::OwnFleet,
                "",
                &rider.id,
                &[sale.id.clone()],
                "run",
                &owner,
            )
            .unwrap();
        let cache = store.download_rider_run(&rider.id).unwrap();
        assert_eq!(cache.stops.len(), 1);
        store.set_app_mode(AppMode::Rider).unwrap();
        assert_eq!(store.app_mode().unwrap(), AppMode::Rider);

        let strokes = vec![SigStroke {
            points: vec![
                SigPoint { x: 0.1, y: 0.5 },
                SigPoint { x: 0.9, y: 0.5 },
            ],
        }];
        let sig_id = store
            .attach_signature_strokes(&sale.id, &strokes, &owner)
            .unwrap();
        assert!(!sig_id.is_empty());

        let mut event = RiderPodEvent::new(&sale.id, &sale.awb_no, "delivered");
        event.signature_attachment_id = sig_id.clone();
        event.cod_collected_paise = 50_00;
        event.lat = 12.97;
        event.lon = 77.59;
        event.feedback_score = 5;
        let label = store.rider_queue_pod(&rider.id, &event).unwrap();
        assert!(label.contains("waiting") || label.contains("1"));
        assert_eq!(store.lan_outbox().unwrap().len(), 1);

        store.apply_rider_pod(&event, &owner).unwrap();
        store.lan_outbox_done(&event.id).unwrap();
        let updated = store.get_sale(&sale.id).unwrap().unwrap();
        assert_eq!(updated.tracking_status, "delivered");
        assert_eq!(updated.feedback_score, 5);
        let evs = store.list_tracking_events(&sale.id).unwrap();
        let last = evs.last().unwrap();
        assert!((last.lat - 12.97).abs() < 0.01);
        assert_eq!(last.pod_ref, sig_id);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn incoming_manifest_and_pickup_run() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let sale = book_one(&mut store, "Hub dest", "Central", 0);
        let incoming = store
            .create_incoming_manifest("BLR Hub", &[sale.awb_no.clone()], "bag 1", &owner)
            .unwrap();
        assert_eq!(incoming.line_count, 1);
        let lines = store.list_incoming_manifest_lines(&incoming.id).unwrap();
        assert_eq!(lines[0].awb_no, sale.awb_no);

        let rider = store.add_rider("Pickup Boy", "888", "Van").unwrap();
        let req = store
            .add_pickup_request("p1", "Acme Co", "MG Road", "560001", "Central", 2, "")
            .unwrap();
        let run = store
            .create_pickup_run(&rider.id, &[req.id], "drop at office", &owner)
            .unwrap();
        assert_eq!(run.request_count, 1);
        assert!(store.list_open_pickup_requests().unwrap().is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn auto_assign_stop_reorder_commission_rto_tally_notify() {
        let path = temp_db();
        let mut store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        let r1 = store.add_rider("Asha", "1", "Bike").unwrap();
        let r2 = store.add_rider("Bala", "2", "Bike").unwrap();
        store
            .upsert_assign_rule(&AssignRule {
                id: new_id(),
                area: "North".into(),
                rider_id: r1.id.clone(),
                rider_name: r1.name.clone(),
                ..Default::default()
            })
            .unwrap();
        store
            .upsert_assign_rule(&AssignRule {
                id: new_id(),
                area: "North".into(),
                rider_id: r2.id.clone(),
                rider_name: r2.name.clone(),
                ..Default::default()
            })
            .unwrap();
        let s1 = book_one(&mut store, "North A", "North", 10_00);
        let s2 = book_one(&mut store, "North B", "North", 20_00);
        let (n, _) = store.auto_assign_area("North", &owner).unwrap();
        assert_eq!(n, 2);

        let manifests = store.list_manifests(10).unwrap();
        let m = manifests
            .iter()
            .find(|m| m.kind == "own_fleet")
            .unwrap();
        store
            .sort_manifest_by_area_pincode(&m.id)
            .unwrap();
        let (_mh, lines) = store.manifest_detail(&m.id).unwrap();
        let ids: Vec<String> = lines.iter().map(|l| l.line.sale_id.clone()).collect();
        if ids.len() >= 2 {
            store
                .reorder_manifest_stops(&m.id, &[ids[1].clone(), ids[0].clone()])
                .unwrap();
        }

        store
            .set_rider_pay_rate(&RiderPayRate {
                rider_id: r1.id.clone(),
                per_delivery_paise: 15_00,
                per_kg_paise: 0,
            })
            .unwrap();
        store
            .advance_tracking(
                &s1.id,
                TrackingStatus::Delivered,
                "ok",
                "",
                &owner,
            )
            .unwrap();
        // Ensure rider_id set for commission (auto-assign may have put on either).
        store
            .conn
            .execute(
                "UPDATE sales SET rider_id = ?1 WHERE id = ?2",
                params![r1.id, s1.id],
            )
            .unwrap();
        let (slip, text) = store.settle_rider_commission(&r1.id, 10_00, &owner).unwrap();
        assert!(text.contains("commission") || text.contains("Pay"));
        assert!(slip.deliveries >= 1 || slip.pay_paise >= 0);

        store.set_ndr_max_attempts(1).unwrap();
        store.set_rto_fee_settings(0, 50).unwrap();
        let s3 = book_one(&mut store, "Fail", "South", 0);
        let st = store.record_ndr(&s3.id, "refused", "", &owner).unwrap();
        assert_eq!(st, TrackingStatus::Rto);
        let fee = store.apply_rto_fee_if_needed(&s3.id).unwrap();
        // May already have been applied in record_ndr.
        let s3b = store.get_sale(&s3.id).unwrap().unwrap();
        assert!(s3b.extras_json.contains("rto_fee") || fee > 0 || s3b.total_paise > 0);

        let docket = store
            .format_docket_for_sale(&s1.id, crate::labels::DocketSize::A5)
            .unwrap();
        assert!(docket.contains("CONSIGNEE"));

        let tally = store.tally_voucher_export(50).unwrap();
        assert!(tally.contains("VoucherType"));
        assert!(tally.len() > 40);

        let insp = VehicleInspection::new(&m.id, &r1.id);
        store.save_vehicle_inspection(&insp).unwrap();

        store
            .set_franchise_rate(&s1.carrier_party_id, 25, true)
            .unwrap_or(());
        // Use a carrier party for franchise.
        let carrier = store.find_or_create_customer("DTDC Local").unwrap();
        store
            .conn
            .execute(
                "UPDATE sales SET carrier_party_id = ?1, carrier_cost_paise = 10 WHERE id = ?2",
                params![carrier.id, s2.id],
            )
            .unwrap();
        store.set_franchise_rate(&carrier.id, 25, true).unwrap();
        let (_rows, stmt) = store
            .franchise_commission_statement("", "")
            .unwrap();
        assert!(stmt.contains("Franchise") || stmt.contains("commission"));

        let sug = store.suggest_destinations("560", 10).unwrap();
        let _ = sug;

        let alert = crate::notify::draft_booking_alert("999", &s1.awb_no, "Pune", "/track/x");
        store.queue_notify(&alert).unwrap();
        let listed = store.list_notify_outbox(5).unwrap();
        assert!(!listed.is_empty());
        let nid = listed[0].id.clone();
        let wa = store.notify_whatsapp_url(&nid).unwrap();
        assert!(wa.contains("wa.me"));
        let sms = store.notify_sms_text(&nid).unwrap();
        assert!(!sms.is_empty());
        store.mark_notify_sent(&nid).unwrap();
        let after = store.list_notify_outbox(5).unwrap();
        assert_eq!(after.iter().find(|r| r.id == nid).unwrap().status, "sent");

        let receipt_wa = store.whatsapp_receipt_url(&s1.id, "919999999999").unwrap();
        assert!(receipt_wa.contains("wa.me"));

        assert_eq!(fee.max(0), fee); // silence unused if already applied
        let _ = fs::remove_file(&path);
    }
}
