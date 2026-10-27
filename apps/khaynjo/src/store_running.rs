//! A table's running order: KOT now, more KOTs, print the bill, settle.
//!
//! Petpooja's core loop (and every restaurant's): the waiter punches items
//! and sends a KOT; more KOTs follow; the counter prints the bill and then
//! settles it. Until 2026-10-23 Khaynjo kept only a KOT *summary* and cleared
//! the cart on send, so the counter had nothing to bill. The open order now
//! lives here, on the Server, line by line (`BUSINESS_BOOKS.md` § Open orders
//! are not books yet): cancelled and complimentary lines are kept with reason
//! and approver (vision §18), moves keep the same order id, and finalize
//! closes the order with its sale.
//!
//! Child module of `store` so it shares the connection and private helpers
//! without growing `store.rs` further.

use super::{kot_line_label, Store};
use crate::domain::{
    new_id, DiningTable, KotStatus, Money, Right, RightLevel, RunningLine, RunningOrder,
    RunningOrderStatus, SaleDraft, StaffMember, TableStatus,
};
use chrono::Local;
use rusqlite::{params, OptionalExtension};

/// Sort key that puts `D2` before `D10` (owners number tables, not spell them).
pub(crate) fn natural_key(name: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut digits = String::new();
    for ch in name.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else {
            if !digits.is_empty() {
                out.push((std::mem::take(&mut text).to_lowercase(), digits.parse().unwrap_or(0)));
                digits.clear();
            }
            text.push(ch);
        }
    }
    out.push((text.to_lowercase(), digits.parse().unwrap_or(0)));
    out
}

fn row_to_line(row: &rusqlite::Row<'_>) -> rusqlite::Result<RunningLine> {
    Ok(RunningLine {
        id: row.get(0)?,
        order_id: row.get(1)?,
        kot_id: row.get(2)?,
        product_id: row.get(3)?,
        name: row.get(4)?,
        unit_price_paise: row.get(5)?,
        qty_milli: row.get(6)?,
        unit_label: row.get(7)?,
        note: row.get(8)?,
        cancelled: row.get::<_, i64>(9)? != 0,
        complimentary: row.get::<_, i64>(10)? != 0,
        reason: row.get(11)?,
        prepared: row.get::<_, i64>(12)? != 0,
        changed_by: row.get(13)?,
        approved_by: row.get(14)?,
        created_at: row.get(15)?,
        created_by: row.get(16)?,
        seat_no: row.get::<_, i64>(17).unwrap_or(0),
        held: row.get::<_, i64>(18).unwrap_or(0) != 0,
        course_no: row.get::<_, i64>(19).unwrap_or(0),
    })
}

const LINE_COLUMNS: &str = "id, order_id, kot_id, product_id, name, unit_price_paise,
    qty_milli, unit_label, note, cancelled, complimentary, reason, prepared,
    changed_by, approved_by, created_at, created_by,
    COALESCE(seat_no, 0), COALESCE(held, 0), COALESCE(course_no, 0)";

const LINE_INSERT_COLUMNS: &str = "id, order_id, kot_id, product_id, name, unit_price_paise,
    qty_milli, unit_label, note, cancelled, complimentary, reason, prepared,
    changed_by, approved_by, created_at, created_by, seat_no, held, course_no";

impl Store {
    /// May `actor` do `right`? `Allowed` passes; `WithApproval` needs an
    /// Owner/Manager PIN now (the approver is returned so the action can
    /// record them); `Denied` is refused. Enforced here, on the Server, so a
    /// paired terminal or a skipped screen cannot bypass it (vision §17).
    pub fn check_right(
        &self,
        actor: &StaffMember,
        right: Right,
        approval_pin: Option<&str>,
    ) -> Result<Option<StaffMember>, String> {
        match actor.rights().level(right) {
            RightLevel::Allowed => Ok(None),
            RightLevel::WithApproval => {
                let pin = approval_pin.map(str::trim).unwrap_or_default();
                if pin.is_empty() {
                    return Err(format!(
                        "{} needs a manager's approval — enter an Owner or Manager PIN",
                        right_words(right)
                    ));
                }
                Ok(Some(self.verify_approval_pin(pin)?))
            }
            RightLevel::Denied => Err(format!(
                "{} is not allowed for {} — ask the owner or a manager",
                right_words(right),
                actor.name
            )),
        }
    }

    /// The open (running or printed) order on a table, with all its lines —
    /// cancelled and complimentary ones included.
    pub fn running_order(&self, table_id: &str) -> Result<Option<RunningOrder>, String> {
        let order = self
            .conn
            .query_row(
                "SELECT id, table_id, status, opened_at, opened_by, printed_at
                   FROM running_orders
                  WHERE table_id = ?1 AND status IN ('running', 'printed')
                  ORDER BY opened_at DESC LIMIT 1",
                params![table_id],
                |r| {
                    Ok(RunningOrder {
                        id: r.get(0)?,
                        table_id: r.get(1)?,
                        status: RunningOrderStatus::parse(&r.get::<_, String>(2)?),
                        opened_at: r.get(3)?,
                        opened_by: r.get(4)?,
                        printed_at: r.get(5)?,
                        lines: Vec::new(),
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let Some(mut order) = order else {
            return Ok(None);
        };
        order.lines = self.running_lines(&order.id)?;
        Ok(Some(order))
    }

    fn running_lines(&self, order_id: &str) -> Result<Vec<RunningLine>, String> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {LINE_COLUMNS} FROM running_lines WHERE order_id = ?1
                  ORDER BY seq, created_at"
            ))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![order_id], row_to_line)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    fn running_line(&self, line_id: &str) -> Result<RunningLine, String> {
        self.conn
            .query_row(
                &format!("SELECT {LINE_COLUMNS} FROM running_lines WHERE id = ?1"),
                params![line_id],
                row_to_line,
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "That item is no longer on the order".to_string())
    }

    /// Every open order (for a waiter's tablet: it shows what each table has).
    pub fn list_running_orders(&self) -> Result<Vec<RunningOrder>, String> {
        let table_ids: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT DISTINCT table_id FROM running_orders
                      WHERE status IN ('running', 'printed')",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        };
        let mut out = Vec::new();
        for tid in table_ids {
            if let Some(order) = self.running_order(&tid)? {
                out.push(order);
            }
        }
        Ok(out)
    }

    /// A Client's copy of the Server's open orders (sole writer: the Server).
    pub fn replace_running_from_sync(&self, orders: &[RunningOrder]) -> Result<(), String> {
        self.conn
            .execute_batch("DELETE FROM running_lines; DELETE FROM running_orders;")
            .map_err(|e| e.to_string())?;
        for order in orders {
            self.conn
                .execute(
                    "INSERT INTO running_orders(id, table_id, status, opened_at, opened_by, printed_at)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        order.id,
                        order.table_id,
                        order.status.as_str(),
                        order.opened_at,
                        order.opened_by,
                        order.printed_at
                    ],
                )
                .map_err(|e| e.to_string())?;
            for (seq, line) in order.lines.iter().enumerate() {
                self.insert_running_line(line, seq as i64)?;
            }
        }
        Ok(())
    }

    fn insert_running_line(&self, l: &RunningLine, seq: i64) -> Result<(), String> {
        self.conn
            .execute(
                &format!(
                    "INSERT INTO running_lines({LINE_INSERT_COLUMNS}, seq)
                     VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)"
                ),
                params![
                    l.id,
                    l.order_id,
                    l.kot_id,
                    l.product_id,
                    l.name,
                    l.unit_price_paise,
                    l.qty_milli,
                    l.unit_label,
                    l.note,
                    l.cancelled as i64,
                    l.complimentary as i64,
                    l.reason,
                    l.prepared as i64,
                    l.changed_by,
                    l.approved_by,
                    l.created_at,
                    l.created_by,
                    l.seat_no,
                    l.held as i64,
                    l.course_no,
                    seq
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn next_line_seq(&self, order_id: &str) -> Result<i64, String> {
        self.conn
            .query_row(
                "SELECT COALESCE(MAX(seq), -1) + 1 FROM running_lines WHERE order_id = ?1",
                params![order_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    /// Live lines not yet on a KOT (empty `kot_id`) — the punched, unsent ticket.
    /// Held lines are excluded (coursing / combo components wait for Fire).
    fn pending_live_lines(&self, order_id: &str) -> Result<Vec<RunningLine>, String> {
        self.pending_live_lines_filtered(order_id, None, true)
    }

    fn pending_live_lines_filtered(
        &self,
        order_id: &str,
        course: Option<i64>,
        skip_held: bool,
    ) -> Result<Vec<RunningLine>, String> {
        Ok(self
            .running_lines(order_id)?
            .into_iter()
            .filter(|l| l.is_live() && l.kot_id.is_empty())
            .filter(|l| !skip_held || !l.held)
            .filter(|l| course.map(|c| l.course_no == c).unwrap_or(true))
            .collect())
    }

    /// Add or bump a pending (not yet KOT'd) line on the table's open order.
    /// Negative `delta_milli` removes quantity; at 0 the pending row is deleted
    /// (never kitchen-told, so no cancel record).
    pub fn bump_pending_product(
        &self,
        table_id: &str,
        product: &crate::domain::Product,
        delta_milli: i64,
        area: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Change the table's order")?;
        if delta_milli == 0 {
            return Ok(());
        }
        if !product.available || product.archived {
            return Err(format!("{} is marked sold out", product.name));
        }
        let unit_price = product.price_in_area(area);
        if unit_price.paise() < 0 {
            return Err(format!("{} has a negative price — fix it in Menu & items", product.name));
        }
        let order_id = self.ensure_running_order(table_id, actor)?;
        let pending = self.pending_live_lines(&order_id)?;
        let existing = pending.iter().find(|l| {
            l.product_id == product.id && l.note.is_empty() && !l.complimentary
        });
        if let Some(line) = existing {
            let new_qty = line.qty_milli + delta_milli;
            if new_qty <= 0 {
                self.conn
                    .execute("DELETE FROM running_lines WHERE id = ?1 AND kot_id = ''", params![line.id])
                    .map_err(|e| e.to_string())?;
                self.release_if_empty(table_id, actor)?;
            } else {
                self.conn
                    .execute(
                        "UPDATE running_lines SET qty_milli = ?2 WHERE id = ?1 AND kot_id = ''",
                        params![line.id, new_qty],
                    )
                    .map_err(|e| e.to_string())?;
            }
        } else if delta_milli > 0 {
            let seq = self.next_line_seq(&order_id)?;
            self.insert_running_line(
                &RunningLine {
                    id: new_id(),
                    order_id: order_id.clone(),
                    product_id: product.id.clone(),
                    name: product.name.clone(),
                    unit_price_paise: unit_price.paise(),
                    qty_milli: delta_milli,
                    unit_label: product.unit_label.clone(),
                    created_at: Local::now().to_rfc3339(),
                    created_by: actor.name.clone(),
                    ..Default::default()
                },
                seq,
            )?;
        }
        Ok(())
    }

    /// Move not-yet-KOT'd lines from one table's ticket to another (Order →
    /// Change table). Sent lines stay on the old table. No Move-table right —
    /// this is re-seating punches, not relocating a party that already has KOTs.
    pub fn reseat_pending(
        &self,
        from_table_id: &str,
        to_table_id: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Change table")?;
        if from_table_id == to_table_id {
            return Ok(());
        }
        let Some(order) = self.running_order(from_table_id)? else {
            return Ok(());
        };
        let ids: Vec<String> = order
            .live_lines()
            .filter(|l| l.kot_id.is_empty())
            .map(|l| l.id.clone())
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        self.move_lines(&ids, to_table_id, actor)?;
        Ok(())
    }

    /// Write unsent cart lines onto the table as pending (seat / change-table).
    pub fn seat_unsent_lines(
        &self,
        table_id: &str,
        lines: &[crate::domain::CartLine],
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Seat items on a table")?;
        let order_id = self.ensure_running_order(table_id, actor)?;
        for line in lines.iter().filter(|l| !l.is_sent() && l.qty_milli > 0) {
            let pending = self.pending_live_lines(&order_id)?;
            if let Some(existing) = pending.iter().find(|l| {
                l.product_id == line.product_id
                    && l.note == line.note
                    && l.complimentary == line.complimentary
            }) {
                self.conn
                    .execute(
                        "UPDATE running_lines SET qty_milli = qty_milli + ?2, unit_price_paise = ?3
                          WHERE id = ?1 AND kot_id = ''",
                        params![existing.id, line.qty_milli, line.unit_price.paise()],
                    )
                    .map_err(|e| e.to_string())?;
            } else {
                let seq = self.next_line_seq(&order_id)?;
                self.insert_running_line(
                    &RunningLine {
                        id: if line.running_line_id.is_empty() {
                            new_id()
                        } else {
                            line.running_line_id.clone()
                        },
                        order_id: order_id.clone(),
                        product_id: line.product_id.clone(),
                        name: line.name.clone(),
                        unit_price_paise: line.unit_price.paise(),
                        qty_milli: line.qty_milli,
                        unit_label: line.unit_label.clone(),
                        note: line.note.clone(),
                        complimentary: line.complimentary,
                        created_at: Local::now().to_rfc3339(),
                        created_by: actor.name.clone(),
                        ..Default::default()
                    },
                    seq,
                )?;
            }
        }
        Ok(())
    }

    /// The open order on this table, opening one (and seating the table) if
    /// there is none.
    fn ensure_running_order(&self, table_id: &str, actor: &StaffMember) -> Result<String, String> {
        if let Some(order) = self.running_order(table_id)? {
            return Ok(order.id);
        }
        let exists: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM tables_dining WHERE id = ?1",
                params![table_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists == 0 {
            return Err("Table not found".into());
        }
        let id = new_id();
        self.conn
            .execute(
                "INSERT INTO running_orders(id, table_id, status, opened_at, opened_by)
                 VALUES(?1, ?2, ?3, ?4, ?5)",
                params![
                    id,
                    table_id,
                    RunningOrderStatus::Running.as_str(),
                    Local::now().to_rfc3339(),
                    actor.name
                ],
            )
            .map_err(|e| e.to_string())?;
        self.occupy_table(table_id)?;
        Ok(id)
    }

    /// KOT number and send time for each of these tickets — the order
    /// ticket's group headers ("KOT 3 · 20:40").
    pub fn kot_heads(&self, kot_ids: &[String]) -> Result<Vec<(String, i64, String)>, String> {
        let mut out = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT id, COALESCE(kot_no, 0), created_at FROM kot_tickets WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        for id in kot_ids {
            if let Ok(row) = stmt.query_row(params![id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?))
            }) {
                out.push(row);
            }
        }
        Ok(out)
    }

    /// Next KOT number for today (kitchens call tickets by number).
    fn next_kot_no(&self) -> Result<i64, String> {
        let key = format!("kot_seq_{}", Local::now().format("%Y-%m-%d"));
        let next = self
            .meta_get(&key)?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0)
            + 1;
        self.meta_set(&key, &next.to_string())?;
        Ok(next)
    }

    fn insert_kot(
        &self,
        summary: &str,
        station_id: &str,
        table_id: &str,
        order_id: &str,
        total_paise: i64,
        kind: &str,
        actor: &StaffMember,
    ) -> Result<String, String> {
        let kot_id = new_id();
        self.conn
            .execute(
                "INSERT INTO kot_tickets(
                    id, sale_id, created_at, status, summary, branch_id, station_id,
                    table_id, total_paise, order_id, kot_no, kind, created_by
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    kot_id,
                    format!("prepay:{}", actor.id),
                    Local::now().to_rfc3339(),
                    KotStatus::New.as_str(),
                    summary,
                    self.active_branch_id()?,
                    station_id,
                    table_id,
                    total_paise,
                    order_id,
                    self.next_kot_no()?,
                    kind,
                    actor.name
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(kot_id)
    }

    pub(crate) fn table_label(&self, table_id: &str) -> Result<String, String> {
        Ok(self
            .conn
            .query_row(
                "SELECT name FROM tables_dining WHERE id = ?1",
                params![table_id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or_default())
    }

    /// Send new lines to the kitchen: one KOT per station. With a table,
    /// promotes **pending** running lines (empty `kot_id`) onto KOTs — the
    /// ticket is already the Server order. Without a table (counter), only
    /// kitchen tickets are written from the draft. Held lines are skipped.
    /// Returns KOT ids to print.
    pub fn send_kot(
        &self,
        draft: &SaleDraft,
        table_id: Option<&str>,
        actor: &StaffMember,
    ) -> Result<Vec<String>, String> {
        let table_id = table_id.filter(|t| !t.is_empty());
        if let Some(tid) = table_id {
            self.seat_draft_unsent(tid, draft, actor)?;
            self.send_kot_filtered(tid, actor, None, true)
        } else {
            self.send_kot_counter(draft, actor)
        }
    }

    fn seat_draft_unsent(
        &self,
        tid: &str,
        draft: &SaleDraft,
        actor: &StaffMember,
    ) -> Result<(), String> {
        let order_id = self.ensure_running_order(tid, actor)?;
        let unsent: Vec<crate::domain::CartLine> = draft
            .lines
            .iter()
            .filter(|l| !l.kot_sent)
            .map(|l| {
                let qty_milli = if l.qty_milli > 0 {
                    l.qty_milli
                } else {
                    l.qty * 1000
                };
                crate::domain::CartLine {
                    product_id: l.product_id.clone(),
                    name: l.name.clone(),
                    unit_price: Money(l.unit_price_paise),
                    qty: l.qty,
                    qty_milli,
                    unit_label: l.unit_label.clone(),
                    note: l.note.clone(),
                    base_unit_paise: l.unit_price_paise,
                    complimentary: l.line_total_paise == 0 && l.unit_price_paise > 0,
                    ..Default::default()
                }
            })
            .collect();
        if unsent.is_empty() {
            return Ok(());
        }
        let pending = self.pending_live_lines_filtered(&order_id, None, false)?;
        let to_seat: Vec<_> = unsent
            .into_iter()
            .filter(|l| {
                !pending.iter().any(|p| {
                    p.product_id == l.product_id
                        && p.note == l.note
                        && p.complimentary == l.complimentary
                })
            })
            .collect();
        if !to_seat.is_empty() {
            self.seat_unsent_lines(tid, &to_seat, actor)?;
        }
        Ok(())
    }

    fn send_kot_counter(
        &self,
        draft: &SaleDraft,
        actor: &StaffMember,
    ) -> Result<Vec<String>, String> {
        self.assert_server_writer("Send a KOT")?;
        if !self.venue_mode()?.creates_kot() {
            return Err("Turn on Tables & kitchen in Settings to send kitchen tickets".into());
        }
        let stations = self.list_stations()?;
        let lines: Vec<_> = draft.lines.iter().filter(|l| !l.kot_sent).collect();
        if lines.is_empty() {
            return Err("Nothing new to send — tap items first".into());
        }
        let mut by_station: Vec<(String, Vec<&crate::domain::SaleLine>)> = Vec::new();
        for line in &lines {
            let station_id = self
                .get_product(&line.product_id)?
                .map(|p| p.station_id)
                .unwrap_or_default();
            match by_station.iter_mut().find(|(sid, _)| *sid == station_id) {
                Some((_, items)) => items.push(line),
                None => by_station.push((station_id, vec![line])),
            }
        }
        let mut kot_ids = Vec::new();
        for (station_id, items) in &by_station {
            let head = stations
                .iter()
                .find(|s| s.id == *station_id)
                .map(|s| format!("[{}] ", s.name))
                .unwrap_or_default();
            let summary = items.iter().map(|l| kot_line_label(l)).collect::<Vec<_>>().join(", ");
            let total: i64 = items.iter().map(|l| l.line_total_paise).sum();
            let kot_id = self.insert_kot(
                &format!("{head}{summary}"),
                station_id,
                "",
                "",
                total,
                "kot",
                actor,
            )?;
            kot_ids.push(kot_id);
        }
        self.audit(actor, "kot.send", &format!("{} ticket(s)", kot_ids.len()))?;
        Ok(kot_ids)
    }

    /// Seat a fixed-combo parent at combo price and held component lines
    /// (they print only when the combo KOT fires).
    pub fn seat_fixed_combo(
        &self,
        table_id: &str,
        parent_product_id: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Seat combo")?;
        let schemes = self.list_schemes()?;
        let scheme = schemes
            .iter()
            .find(|s| {
                s.active
                    && s.kind == crate::schemes::SchemeKind::Combo
                    && s.trigger_json.contains(parent_product_id)
            })
            .ok_or_else(|| "No fixed combo scheme for that item".to_string())?;
        let trigger: crate::schemes::ComboTrigger =
            serde_json::from_str(&scheme.trigger_json).unwrap_or_default();
        let reward: crate::schemes::ComboReward =
            serde_json::from_str(&scheme.reward_json).unwrap_or_default();
        let parent = self
            .get_product(parent_product_id)?
            .ok_or_else(|| "Combo item not found".to_string())?;
        let order_id = self.ensure_running_order(table_id, actor)?;
        let parent_id = new_id();
        let price = if reward.price_paise > 0 {
            reward.price_paise
        } else {
            parent.price.paise()
        };
        let seq = self.next_line_seq(&order_id)?;
        self.insert_running_line(
            &RunningLine {
                id: parent_id.clone(),
                order_id: order_id.clone(),
                product_id: parent.id.clone(),
                name: parent.name.clone(),
                unit_price_paise: price,
                qty_milli: 1000,
                unit_label: parent.unit_label.clone(),
                note: format!("Scheme: {}", scheme.name),
                created_at: Local::now().to_rfc3339(),
                created_by: actor.name.clone(),
                ..Default::default()
            },
            seq,
        )?;
        let mut seq = seq + 1;
        for comp in &trigger.components {
            let Some(p) = self.get_product(&comp.product_id)? else {
                continue;
            };
            self.insert_running_line(
                &RunningLine {
                    id: new_id(),
                    order_id: order_id.clone(),
                    product_id: p.id.clone(),
                    name: p.name.clone(),
                    unit_price_paise: p.price.paise(),
                    qty_milli: comp.qty.max(1) * 1000,
                    unit_label: p.unit_label.clone(),
                    note: format!("combo:{parent_id}"),
                    complimentary: true,
                    held: true,
                    created_at: Local::now().to_rfc3339(),
                    created_by: actor.name.clone(),
                    ..Default::default()
                },
                seq,
            )?;
            seq += 1;
        }
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?1 WHERE id = ?2",
                params![TableStatus::Occupied.as_str(), table_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Send pending lines; optionally only one course. When sending a combo
    /// parent, its held `combo:<parent>` components are unheld and included.
    pub fn send_kot_filtered(
        &self,
        table_id: &str,
        actor: &StaffMember,
        course: Option<i64>,
        skip_held: bool,
    ) -> Result<Vec<String>, String> {
        self.assert_server_writer("Send a KOT")?;
        if !self.venue_mode()?.creates_kot() {
            return Err("Turn on Tables & kitchen in Settings to send kitchen tickets".into());
        }
        let tid = table_id.trim();
        if tid.is_empty() {
            return Err("Nothing new to send — tap items first".into());
        }
        let order_id = self.ensure_running_order(tid, actor)?;
        // Fire combo components that belong to parents about to send.
        let parents: Vec<String> = self
            .pending_live_lines_filtered(&order_id, course, skip_held)?
            .into_iter()
            .map(|l| l.id)
            .collect();
        for pid in &parents {
            self.conn
                .execute(
                    "UPDATE running_lines SET held = 0
                      WHERE order_id = ?1 AND kot_id = '' AND held = 1
                        AND note = ?2",
                    params![order_id, format!("combo:{pid}")],
                )
                .map_err(|e| e.to_string())?;
        }
        let mut pending = self.pending_live_lines_filtered(&order_id, course, skip_held)?;
        // Include just-unheld combo components even when skip_held (they match note).
        if skip_held {
            let have: std::collections::HashSet<_> = pending.iter().map(|l| l.id.clone()).collect();
            let extra: Vec<_> = self
                .running_lines(&order_id)?
                .into_iter()
                .filter(|l| {
                    l.is_live()
                        && l.kot_id.is_empty()
                        && !l.held
                        && parents.iter().any(|p| l.note == format!("combo:{p}"))
                        && !have.contains(&l.id)
                })
                .collect();
            pending.extend(extra);
        }
        if pending.is_empty() {
            return Err("Nothing new to send — tap items first".into());
        }
        let table_prefix = format!("{} · ", self.table_label(tid)?);
        let stations = self.list_stations()?;
        let mut by_station: Vec<(String, Vec<RunningLine>)> = Vec::new();
        for line in pending {
            let station_id = self
                .get_product(&line.product_id)?
                .map(|p| p.station_id)
                .unwrap_or_default();
            match by_station.iter_mut().find(|(sid, _)| *sid == station_id) {
                Some((_, items)) => items.push(line),
                None => by_station.push((station_id, vec![line])),
            }
        }
        let mut kot_ids = Vec::new();
        for (station_id, items) in &by_station {
            let head = stations
                .iter()
                .find(|s| s.id == *station_id)
                .map(|s| format!("[{}] ", s.name))
                .unwrap_or_default();
            let summary = items
                .iter()
                .map(|l| {
                    let qty = crate::domain::format_qty_display(l.qty_milli, 0, &l.unit_label);
                    let allergens = self
                        .product_allergens(&l.product_id)
                        .unwrap_or_default();
                    let mut s = if l.note.is_empty() || l.note.starts_with("combo:") || l.note.starts_with("Scheme:")
                    {
                        format!("{qty}× {}", l.name)
                    } else {
                        format!("{qty}× {} ({})", l.name, l.note)
                    };
                    if !allergens.is_empty() {
                        s.push_str(&format!(" [allergen: {allergens}]"));
                    }
                    s
                })
                .collect::<Vec<_>>()
                .join(", ");
            let total: i64 = items.iter().map(|l| l.billed_paise()).sum();
            let kot_id = self.insert_kot(
                &format!("{table_prefix}{head}{summary}"),
                station_id,
                tid,
                &order_id,
                total,
                "kot",
                actor,
            )?;
            for l in items {
                self.conn
                    .execute(
                        "UPDATE running_lines SET kot_id = ?2, held = 0 WHERE id = ?1 AND kot_id = ''",
                        params![l.id, kot_id],
                    )
                    .map_err(|e| e.to_string())?;
            }
            kot_ids.push(kot_id);
        }
        self.conn
            .execute(
                "UPDATE running_orders SET status = 'running', printed_at = ''
                  WHERE id = ?1 AND status = 'printed'",
                params![order_id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?1 WHERE id = ?2",
                params![TableStatus::Occupied.as_str(), tid],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "kot.send",
            &format!("{table_prefix}{} ticket(s)", kot_ids.len()),
        )?;
        Ok(kot_ids)
    }

    /// Cancel a line the kitchen already has — all of it, or `qty_milli` of
    /// it. Nothing is deleted: the cancelled quantity stays on the order with
    /// the reason, whether the kitchen had already cooked it (wastage), who
    /// asked and who approved; the kitchen gets a cancellation ticket.
    /// Returns that ticket's id (for printing).
    pub fn cancel_running_line(
        &self,
        line_id: &str,
        qty_milli: Option<i64>,
        reason: &str,
        prepared: bool,
        actor: &StaffMember,
        approval_pin: Option<&str>,
    ) -> Result<String, String> {
        self.assert_server_writer("Cancel an item")?;
        let reason = reason.trim();
        if reason.is_empty() {
            return Err("Say why it is cancelled (e.g. guest changed mind, out of stock)".into());
        }
        let approver = self.check_right(actor, Right::CancelSentItem, approval_pin)?;
        let line = self.running_line(line_id)?;
        if line.cancelled {
            return Err("Already cancelled".into());
        }
        let cancel_qty = qty_milli.unwrap_or(line.qty_milli);
        if cancel_qty <= 0 || cancel_qty > line.qty_milli {
            return Err("Cancel between 1 and the quantity sent".into());
        }
        let approved_by = approver.as_ref().map(|a| a.name.clone()).unwrap_or_default();
        let now = Local::now().to_rfc3339();
        let cancelled_id = if cancel_qty == line.qty_milli {
            self.conn
                .execute(
                    "UPDATE running_lines
                        SET cancelled = 1, reason = ?2, prepared = ?3, changed_by = ?4,
                            approved_by = ?5
                      WHERE id = ?1",
                    params![line.id, reason, prepared as i64, actor.name, approved_by],
                )
                .map_err(|e| e.to_string())?;
            line.id.clone()
        } else {
            // Part of a line: the rest keeps cooking, the cancelled part
            // becomes its own (cancelled) line right after it.
            self.conn
                .execute(
                    "UPDATE running_lines SET qty_milli = qty_milli - ?2 WHERE id = ?1",
                    params![line.id, cancel_qty],
                )
                .map_err(|e| e.to_string())?;
            let seq: i64 = self
                .conn
                .query_row(
                    "SELECT seq FROM running_lines WHERE id = ?1",
                    params![line.id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            let part = RunningLine {
                id: new_id(),
                qty_milli: cancel_qty,
                cancelled: true,
                reason: reason.to_string(),
                prepared,
                changed_by: actor.name.clone(),
                approved_by: approved_by.clone(),
                created_at: now.clone(),
                ..line.clone()
            };
            self.insert_running_line(&part, seq)?;
            part.id
        };
        let (table_id, station_id): (String, String) = self
            .conn
            .query_row(
                "SELECT o.table_id, COALESCE(k.station_id, '')
                   FROM running_orders o
                   LEFT JOIN kot_tickets k ON k.id = ?2
                  WHERE o.id = ?1",
                params![line.order_id, line.kot_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        let qty = crate::domain::format_qty_display(
            cancel_qty,
            crate::domain::milli_to_whole_qty(cancel_qty),
            &line.unit_label,
        );
        let kot_id = self.insert_kot(
            &format!(
                "{} · CANCEL {qty} {} — {reason}",
                self.table_label(&table_id)?,
                line.name
            ),
            &station_id,
            &table_id,
            &line.order_id,
            0,
            "cancel",
            actor,
        )?;
        self.audit(
            actor,
            "kot.cancel_item",
            &format!(
                "{qty} {} · {reason} · {} · approved by {}",
                line.name,
                if prepared { "already cooked (wastage)" } else { "not cooked" },
                if approved_by.is_empty() { actor.name.as_str() } else { approved_by.as_str() }
            ),
        )?;
        let _ = cancelled_id;
        // Everything sent is cancelled: no food, no bill — the table frees
        // instead of staying "Eating" with an empty order.
        if let Some(order) = self.running_order(&table_id)?
            && order.live_lines().next().is_none()
        {
            self.close_order_without_sale(&order, actor, "table.release_all_cancelled")?;
        }
        Ok(kot_id)
    }

    /// Give a sent line free (NC) or charge it again. Kept at its real item,
    /// quantity and value so reports show what was given away.
    pub fn set_running_line_complimentary(
        &self,
        line_id: &str,
        on: bool,
        reason: &str,
        actor: &StaffMember,
        approval_pin: Option<&str>,
    ) -> Result<(), String> {
        self.assert_server_writer("Change a bill")?;
        let reason = reason.trim();
        if on && reason.is_empty() {
            return Err("Say why it is complimentary (e.g. owner's guest, late food)".into());
        }
        let approver = self.check_right(actor, Right::Complimentary, approval_pin)?;
        let line = self.running_line(line_id)?;
        if line.cancelled {
            return Err("A cancelled item is not on the bill".into());
        }
        let approved_by = approver.map(|a| a.name).unwrap_or_default();
        self.conn
            .execute(
                "UPDATE running_lines
                    SET complimentary = ?2, reason = ?3, changed_by = ?4, approved_by = ?5
                  WHERE id = ?1",
                params![
                    line.id,
                    on as i64,
                    if on { reason } else { "" },
                    actor.name,
                    approved_by
                ],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            if on { "bill.complimentary" } else { "bill.charge_again" },
            &format!(
                "{} · {} · value {}{}",
                line.name,
                reason,
                crate::locale::format_money(line.value_paise(), &self.locale_settings()?.currency),
                if approved_by.is_empty() { String::new() } else { format!(" · approved by {approved_by}") }
            ),
        )?;
        Ok(())
    }

    /// Move some sent lines to another table's order (guest moved, wrong
    /// table punched). The kitchen tickets stay as they were.
    pub fn move_running_lines(
        &self,
        line_ids: &[String],
        to_table_id: &str,
        actor: &StaffMember,
        approval_pin: Option<&str>,
    ) -> Result<(), String> {
        self.assert_server_writer("Move items")?;
        if line_ids.is_empty() {
            return Err("Pick the items to move".into());
        }
        self.check_right(actor, Right::MoveTable, approval_pin)?;
        self.move_lines(line_ids, to_table_id, actor)
    }

    /// The move itself, after the caller decided it may happen.
    fn move_lines(&self, line_ids: &[String], to_table_id: &str, actor: &StaffMember) -> Result<(), String> {
        let first = self.running_line(&line_ids[0])?;
        let from_table: String = self
            .conn
            .query_row(
                "SELECT table_id FROM running_orders WHERE id = ?1",
                params![first.order_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if from_table == to_table_id {
            return Err("Pick a different table".into());
        }
        let to_order = self.ensure_running_order(to_table_id, actor)?;
        let mut seq = self.next_line_seq(&to_order)?;
        for id in line_ids {
            let line = self.running_line(id)?;
            if line.order_id != first.order_id {
                return Err("Move items from one table at a time".into());
            }
            self.conn
                .execute(
                    "UPDATE running_lines SET order_id = ?2, seq = ?3 WHERE id = ?1",
                    params![id, to_order, seq],
                )
                .map_err(|e| e.to_string())?;
            seq += 1;
        }
        self.release_if_empty(&from_table, actor)?;
        self.audit(
            actor,
            "table.move_items",
            &format!(
                "{} item(s) {} → {}",
                line_ids.len(),
                self.table_label(&from_table)?,
                self.table_label(to_table_id)?
            ),
        )?;
        Ok(())
    }

    /// The kitchen's all-day view: how much of each dish is still to cook
    /// over every ticket not yet ready (tables' running orders and counter
    /// orders), most first. Quantities in thousandths.
    pub fn kitchen_all_day(&self) -> Result<Vec<(String, i64)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, SUM(q) FROM (
                     SELECT name, qty_milli AS q FROM running_lines
                      WHERE cancelled = 0 AND kot_id IN (
                            SELECT id FROM kot_tickets
                             WHERE status IN ('new', 'preparing') AND kind = 'kot')
                     UNION ALL
                     SELECT name, qty_milli AS q FROM sale_lines
                      WHERE sale_id IN (
                            SELECT DISTINCT sale_id FROM kot_tickets
                             WHERE status IN ('new', 'preparing') AND kind = 'kot'
                               AND COALESCE(order_id, '') = '')
                 ) GROUP BY name ORDER BY SUM(q) DESC, name",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Split a table's bill: the picked lines (what one guest pays for) go
    /// to a separate bill on a companion table beside it — "D3/2" in D3's
    /// area — which is printed and paid on its own and disappears once paid
    /// ([`Self::tidy_split_tables`]). Nothing changes for the kitchen or the
    /// money, so no right is needed. Returns the companion table.
    pub fn split_running_lines(&self, line_ids: &[String], actor: &StaffMember) -> Result<DiningTable, String> {
        self.assert_server_writer("Split the bill")?;
        if line_ids.is_empty() {
            return Err("Pick the items for the separate bill".into());
        }
        let first = self.running_line(&line_ids[0])?;
        let from_table: String = self
            .conn
            .query_row(
                "SELECT table_id FROM running_orders WHERE id = ?1",
                params![first.order_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let order = self.running_order(&from_table)?.ok_or("That table has no open bill")?;
        let live = order.live_lines().count();
        if line_ids.len() >= live {
            return Err("Leave at least one item on this bill".into());
        }
        let tables = self.list_tables()?;
        let from = tables.iter().find(|t| t.id == from_table).ok_or("Table not found")?;
        let base = from.name.split('/').next().unwrap_or(&from.name).trim().to_string();
        let name = (2..)
            .map(|i| format!("{base}/{i}"))
            .find(|n| !tables.iter().any(|t| t.name.eq_ignore_ascii_case(n)))
            .unwrap_or_default();
        let companion = self.add_table_in_area(&name, &from.area)?;
        self.conn
            .execute(
                "INSERT INTO split_tables(table_id, parent_id) VALUES(?1, ?2)",
                params![companion.id, from_table],
            )
            .map_err(|e| e.to_string())?;
        self.move_lines(line_ids, &companion.id, actor)?;
        self.audit(actor, "table.split", &format!("{} item(s) {} → {}", line_ids.len(), from.name, name))?;
        Ok(companion)
    }

    /// Remove companion tables from a split once their bill is closed.
    pub fn tidy_split_tables(&self) -> Result<(), String> {
        let ids: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT table_id FROM split_tables")
                .map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |r| r.get(0)).map_err(|e| e.to_string())?;
            rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?
        };
        for id in ids {
            if self.running_order(&id)?.is_some() {
                continue;
            }
            let _ = self.conn.execute("DELETE FROM tables_dining WHERE id = ?1", params![id]);
            self.conn
                .execute("DELETE FROM split_tables WHERE table_id = ?1", params![id])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// The whole party moves to a free table: same order, new table.
    pub fn move_table(
        &self,
        from_table_id: &str,
        to_table_id: &str,
        actor: &StaffMember,
        approval_pin: Option<&str>,
    ) -> Result<(), String> {
        self.assert_server_writer("Move a table")?;
        self.check_right(actor, Right::MoveTable, approval_pin)?;
        if from_table_id == to_table_id {
            return Err("Pick a different table".into());
        }
        let order = self
            .running_order(from_table_id)?
            .ok_or_else(|| "That table has no running order".to_string())?;
        if self.running_order(to_table_id)?.is_some() {
            return Err("That table is running — use Merge to join two tables".into());
        }
        let (status, occupied_at): (String, String) = self
            .conn
            .query_row(
                "SELECT status, COALESCE(occupied_at, '') FROM tables_dining WHERE id = ?1",
                params![from_table_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE running_orders SET table_id = ?2 WHERE id = ?1",
                params![order.id, to_table_id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE kot_tickets SET table_id = ?2 WHERE order_id = ?1",
                params![order.id, to_table_id],
            )
            .map_err(|e| e.to_string())?;
        // The seating keeps its clock: minutes seated travel with the guests.
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?2, occupied_at = ?3, active_sale_id = NULL
                  WHERE id = ?1",
                params![to_table_id, status, occupied_at],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?2, occupied_at = '', active_sale_id = NULL
                  WHERE id = ?1",
                params![from_table_id, TableStatus::Available.as_str()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "table.move",
            &format!("{} → {}", self.table_label(from_table_id)?, self.table_label(to_table_id)?),
        )?;
        Ok(())
    }

    /// Two tables become one bill: `from`'s lines join `into`'s order, `from`
    /// is freed. The merged order is kept (status merged) for the record.
    pub fn merge_tables(
        &self,
        from_table_id: &str,
        into_table_id: &str,
        actor: &StaffMember,
        approval_pin: Option<&str>,
    ) -> Result<(), String> {
        self.assert_server_writer("Merge tables")?;
        self.check_right(actor, Right::MoveTable, approval_pin)?;
        if from_table_id == into_table_id {
            return Err("Pick a different table".into());
        }
        let from = self
            .running_order(from_table_id)?
            .ok_or_else(|| "That table has no running order".to_string())?;
        let into = self
            .running_order(into_table_id)?
            .ok_or_else(|| "Merge into a running table — or Move to a free one".to_string())?;
        let mut seq = self.next_line_seq(&into.id)?;
        for line in &from.lines {
            self.conn
                .execute(
                    "UPDATE running_lines SET order_id = ?2, seq = ?3 WHERE id = ?1",
                    params![line.id, into.id, seq],
                )
                .map_err(|e| e.to_string())?;
            seq += 1;
        }
        self.conn
            .execute(
                "UPDATE running_orders SET status = 'merged', merged_into = ?2, closed_at = ?3
                  WHERE id = ?1",
                params![from.id, into.id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE kot_tickets SET table_id = ?2, order_id = ?3 WHERE order_id = ?1",
                params![from.id, into_table_id, into.id],
            )
            .map_err(|e| e.to_string())?;
        // Merged bill needs a fresh print.
        self.conn
            .execute(
                "UPDATE running_orders SET status = 'running', printed_at = '' WHERE id = ?1",
                params![into.id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?2 WHERE id = ?1",
                params![into_table_id, TableStatus::Occupied.as_str()],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?2, occupied_at = '', active_sale_id = NULL
                  WHERE id = ?1",
                params![from_table_id, TableStatus::Available.as_str()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            "table.merge",
            &format!("{} + {}", self.table_label(into_table_id)?, self.table_label(from_table_id)?),
        )?;
        Ok(())
    }

    /// A table whose order has nothing live left (everything moved or
    /// cancelled) is released; its order closes with no sale.
    fn release_if_empty(&self, table_id: &str, actor: &StaffMember) -> Result<(), String> {
        let Some(order) = self.running_order(table_id)? else {
            return Ok(());
        };
        if order.live_lines().next().is_some() {
            return Ok(());
        }
        self.close_order_without_sale(&order, actor, "table.release_empty")
    }

    fn close_order_without_sale(
        &self,
        order: &RunningOrder,
        actor: &StaffMember,
        audit_action: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE running_orders SET status = 'settled', closed_at = ?2 WHERE id = ?1",
                params![order.id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?2, occupied_at = '', active_sale_id = NULL
                  WHERE id = ?1",
                params![order.table_id, TableStatus::Available.as_str()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(actor, audit_action, &self.table_label(&order.table_id)?)?;
        Ok(())
    }

    /// Free a table whose open order charges nothing (all complimentary or
    /// cancelled) — there is no bill to settle, so release is the close.
    pub fn release_free_order(&self, table_id: &str, actor: &StaffMember) -> Result<(), String> {
        self.assert_server_writer("Release a table")?;
        let order = self
            .running_order(table_id)?
            .ok_or_else(|| "That table has no running order".to_string())?;
        if order.total_paise() > 0 {
            return Err("This table has an open bill — settle it first".into());
        }
        self.close_order_without_sale(&order, actor, "table.release_no_charge")
    }

    /// Accidental Print bill: put the table back to open (Occupied) so staff
    /// can keep ordering or free it — not stuck on "Bill printed".
    pub fn undo_bill_print(
        &self,
        table_id: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        self.assert_server_writer("Undo bill print")?;
        let Some(order) = self.running_order(table_id)? else {
            // Ghost "Bill printed" with no order — just free the tile.
            self.conn
                .execute(
                    "UPDATE tables_dining
                        SET status = ?1, active_sale_id = NULL, occupied_at = '', paid_at = ''
                      WHERE id = ?2",
                    params![TableStatus::Available.as_str(), table_id],
                )
                .map_err(|e| e.to_string())?;
            self.audit(actor, "bill.undo_print", &self.table_label(table_id)?)?;
            return Ok(());
        };
        self.conn
            .execute(
                "UPDATE running_orders SET status = 'running', printed_at = '' WHERE id = ?1",
                params![order.id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?2 WHERE id = ?1",
                params![table_id, TableStatus::Occupied.as_str()],
            )
            .map_err(|e| e.to_string())?;
        if order.live_lines().next().is_none() {
            self.close_order_without_sale(&order, actor, "bill.undo_print_empty")?;
        } else if order.total_paise() == 0 {
            self.release_free_order(table_id, actor)?;
        } else {
            self.audit(actor, "bill.undo_print", &self.table_label(table_id)?)?;
        }
        Ok(())
    }

    /// Print the bill for the guests (before payment). The table turns
    /// "Printed" on every device. Printing again needs the reprint right.
    /// Returns the bill text.
    pub fn print_running_bill(
        &self,
        table_id: &str,
        actor: &StaffMember,
        approval_pin: Option<&str>,
    ) -> Result<String, String> {
        self.assert_server_writer("Print the bill")?;
        let order = self
            .running_order(table_id)?
            .ok_or_else(|| "That table has no running order".to_string())?;
        if order.live_lines().next().is_none() {
            return Err("Nothing on this table's bill".into());
        }
        if order.status == RunningOrderStatus::Printed {
            self.check_right(actor, Right::Reprint, approval_pin)?;
        }
        let text = self.format_running_bill_text(&order)?;
        self.conn
            .execute(
                "UPDATE running_orders SET status = 'printed', printed_at = ?2 WHERE id = ?1",
                params![order.id, Local::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE tables_dining SET status = ?2 WHERE id = ?1",
                params![table_id, TableStatus::PaymentPending.as_str()],
            )
            .map_err(|e| e.to_string())?;
        self.audit(
            actor,
            if order.status == RunningOrderStatus::Printed { "bill.reprint" } else { "bill.print" },
            &self.table_label(table_id)?,
        )?;
        Ok(text)
    }

    /// The guest's bill for an open order: live lines, NC shown at ₹0 with
    /// its value, totals with the shop's tax wording. Cancelled lines are
    /// not on the guest's bill (they are on the order and in reports).
    pub fn format_running_bill_text(&self, order: &RunningOrder) -> Result<String, String> {
        let fiscal = self.fiscal_settings()?;
        let currency = self.locale_settings()?.currency;
        let money = |p: i64| Money(p).format_localized(&currency, &fiscal);
        let mut out = String::new();
        let venue = self.venue_title()?;
        if !venue.trim().is_empty() {
            out.push_str(&format!("{}\n", venue.trim()));
        }
        for line in self.bill_header_lines()? {
            out.push_str(&format!("{line}\n"));
        }
        out.push_str(&format!("Table: {}\n", self.table_label(&order.table_id)?));
        out.push_str(&format!(
            "Date: {}\n",
            Local::now().format("%Y-%m-%d %H:%M")
        ));
        out.push_str("--------------------------------\n");
        for line in order.live_lines() {
            let qty = crate::domain::format_qty_display(
                line.qty_milli,
                crate::domain::milli_to_whole_qty(line.qty_milli),
                &line.unit_label,
            );
            if line.complimentary {
                out.push_str(&format!(
                    "{qty} {}  NC ({}) {}\n",
                    line.name,
                    money(line.value_paise()),
                    money(0)
                ));
            } else {
                out.push_str(&format!("{qty} {}  {}\n", line.name, money(line.billed_paise())));
            }
        }
        out.push_str("--------------------------------\n");
        let subtotal = order.total_paise();
        out.push_str(&format!("Sub total: {}\n", money(subtotal)));
        let (draft, scheme_notes) = self.order_bill_draft_schemed(order);
        for name in &scheme_notes {
            out.push_str(&format!("Scheme: {name}\n"));
        }
        if let Some(draft) = draft {
            let total = self.preview_sale_total(&draft)?;
            if total != subtotal {
                out.push_str(&format!(
                    "{}: {}\n",
                    if fiscal.tax_pricing.is_inclusive() { "Total (incl. tax)" } else { "Tax + charges" },
                    money(total - subtotal)
                ));
            }
            out.push_str(&format!("TOTAL: {}\n", money(total)));
        } else {
            out.push_str(&format!("TOTAL: {}\n", money(0)));
        }
        let footer = self.bill_footer()?;
        if !footer.trim().is_empty() {
            out.push_str(&format!("{}\n", footer.trim()));
        }
        Ok(out)
    }

    /// Lines printed under the restaurant's name on every bill: address,
    /// tax id, food licence — whatever the owner typed (one per line).
    pub fn bill_header_lines(&self) -> Result<Vec<String>, String> {
        let mut lines: Vec<String> = self
            .meta_get("bill_header")?
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        let fssai = self.meta_get("fssai_no")?.unwrap_or_default();
        if !fssai.trim().is_empty() {
            lines.push(format!("FSSAI Lic. No: {}", fssai.trim()));
        }
        Ok(lines)
    }

    pub fn bill_footer(&self) -> Result<String, String> {
        Ok(self.meta_get("bill_footer")?.unwrap_or_default())
    }

    pub fn fssai_no(&self) -> Result<String, String> {
        Ok(self.meta_get("fssai_no")?.unwrap_or_default())
    }

    /// Owner's bill format: header lines (address, phone…), India's FSSAI
    /// food-licence number, and a footer ("Thank you, visit again").
    pub fn set_bill_format(&self, header: &str, fssai_no: &str, footer: &str) -> Result<(), String> {
        self.set_bill_format_ex(header, fssai_no, footer, None, None)
    }

    /// Bill format plus optional UPI VPA / payee for the customer-facing QR.
    pub fn set_bill_format_ex(
        &self,
        header: &str,
        fssai_no: &str,
        footer: &str,
        upi_vpa: Option<&str>,
        upi_payee: Option<&str>,
    ) -> Result<(), String> {
        self.assert_server_writer("Change the bill format")?;
        self.meta_set("bill_header", header.trim())?;
        self.meta_set("fssai_no", fssai_no.trim())?;
        self.meta_set("bill_footer", footer.trim())?;
        if let Some(v) = upi_vpa {
            self.meta_set("upi_vpa", v.trim())?;
        }
        if let Some(p) = upi_payee {
            self.meta_set("upi_payee", p.trim())?;
        }
        Ok(())
    }

    pub fn upi_vpa(&self) -> Result<String, String> {
        Ok(self.meta_get("upi_vpa")?.unwrap_or_default())
    }

    pub fn upi_payee(&self) -> Result<String, String> {
        Ok(self.meta_get("upi_payee")?.unwrap_or_default())
    }

    /// Settle & Save details on a sale: cash the guest handed over, change
    /// returned, and a tip (owed to staff — not revenue, BUSINESS_BOOKS.md §
    /// Shift and day-end cash).
    pub fn record_settlement(
        &self,
        sale_id: &str,
        tendered_paise: i64,
        change_paise: i64,
        tip_paise: i64,
        tip_method: &str,
        actor: &StaffMember,
    ) -> Result<(), String> {
        if tendered_paise < 0 || change_paise < 0 || tip_paise < 0 {
            return Err("Amounts cannot be negative".into());
        }
        self.conn
            .execute(
                "UPDATE sales SET tendered_paise = ?2, change_paise = ?3, tip_paise = ?4,
                        tip_method = ?5
                  WHERE id = ?1",
                params![sale_id, tendered_paise, change_paise, tip_paise, tip_method],
            )
            .map_err(|e| e.to_string())?;
        if tip_paise > 0 {
            let currency = self.locale_settings()?.currency;
            self.audit(
                actor,
                "bill.tip",
                &format!("{sale_id} · {}", crate::locale::format_money(tip_paise, &currency)),
            )?;
        }
        Ok(())
    }

    /// Tips taken today (paise) — shown on the day close, kept out of sales.
    pub fn today_tips_paise(&self) -> Result<i64, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(tip_paise), 0) FROM sales
                  WHERE created_at >= ?1 AND created_at < ?1 || 'Z' AND status != 'void'",
                params![today],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    /// Close the table's running order with the sale that paid it. Called
    /// inside finalize's transaction.
    pub(crate) fn settle_running_order_tx(
        tx: &rusqlite::Transaction<'_>,
        table_id: &str,
        sale_id: &str,
    ) -> Result<(), String> {
        tx.execute(
            "UPDATE running_orders SET status = 'settled', sale_id = ?2, closed_at = ?3
              WHERE table_id = ?1 AND status IN ('running', 'printed')",
            params![table_id, sale_id, Local::now().to_rfc3339()],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---- Floor areas -----------------------------------------------------

    /// Add a table in a floor area ("Dine In", "Parcel", "AC" …).
    pub fn add_table_in_area(&self, name: &str, area: &str) -> Result<DiningTable, String> {
        let mut table = self.add_table(name)?;
        let area = area.trim();
        if !area.is_empty() {
            self.set_table_area(&table.id, area)?;
            table.area = area.to_string();
        }
        Ok(table)
    }

    /// Add `prefix{from}..=prefix{to}` in one go (D1–D20, P1–P20) — how
    /// restaurants number their floor. Names already on the floor are skipped.
    pub fn add_tables_range(&self, prefix: &str, from: u32, to: u32, area: &str) -> Result<usize, String> {
        self.assert_server_writer("Add tables")?;
        if from == 0 || to < from {
            return Err("Use numbers like 1 to 20".into());
        }
        if to - from >= 200 {
            return Err("Add at most 200 tables at a time".into());
        }
        let existing: Vec<String> = self
            .list_tables()?
            .into_iter()
            .map(|t| t.name.trim().to_lowercase())
            .collect();
        let mut added = 0;
        for n in from..=to {
            let name = format!("{}{n}", prefix.trim());
            if existing.contains(&name.to_lowercase()) {
                continue;
            }
            self.add_table_in_area(&name, area)?;
            added += 1;
        }
        Ok(added)
    }

    pub fn set_table_area(&self, table_id: &str, area: &str) -> Result<(), String> {
        self.assert_server_writer("Change tables")?;
        let area = area.trim();
        if !area.is_empty() {
            self.ensure_floor_area(area)?;
        }
        self.conn
            .execute(
                "UPDATE tables_dining SET area = ?2 WHERE id = ?1",
                params![table_id, area],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Floor areas in use, in first-seen order of the sorted floor.
    pub fn list_areas(&self) -> Result<Vec<String>, String> {
        let mut out: Vec<String> = Vec::new();
        for t in self.list_tables()? {
            if !out.contains(&t.area) {
                out.push(t.area);
            }
        }
        Ok(out)
    }

    // ---- Reports -----------------------------------------------------------

    /// Today's cancelled-after-KOT and complimentary lines, for the owner:
    /// what, how much, why, cooked or not, who and who approved.
    pub fn cancellations_and_nc_text(&self) -> Result<String, String> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {LINE_COLUMNS} FROM running_lines
                  WHERE (cancelled = 1 OR complimentary = 1)
                    AND created_at >= ?1 AND created_at < ?1 || 'Z'
                  ORDER BY created_at"
            ))
            .map_err(|e| e.to_string())?;
        let lines = stmt
            .query_map(params![today], row_to_line)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let fiscal = self.fiscal_settings()?;
        let currency = self.locale_settings()?.currency;
        let money = |p: i64| Money(p).format_localized(&currency, &fiscal);
        let (cancelled, nc): (Vec<_>, Vec<_>) = lines.iter().partition(|l| l.cancelled);
        let wastage: i64 = cancelled.iter().filter(|l| l.prepared).map(|l| l.value_paise()).sum();
        let mut out = format!(
            "Cancelled after KOT: {} (wastage {})\nComplimentary (NC): {} (value {})\n",
            cancelled.len(),
            money(wastage),
            nc.len(),
            money(nc.iter().map(|l| l.value_paise()).sum())
        );
        for l in &lines {
            let qty = crate::domain::format_qty_display(
                l.qty_milli,
                crate::domain::milli_to_whole_qty(l.qty_milli),
                &l.unit_label,
            );
            let who = if l.approved_by.is_empty() {
                l.changed_by.clone()
            } else {
                format!("{} (approved {})", l.changed_by, l.approved_by)
            };
            out.push_str(&format!(
                "- {} {qty} {} {} · {} · {who}{}\n",
                if l.cancelled { "CANCEL" } else { "NC" },
                l.name,
                money(l.value_paise()),
                l.reason,
                if l.cancelled && l.prepared { " · cooked" } else { "" }
            ));
        }
        Ok(out)
    }
}

impl Store {
    /// Bill draft with active schemes applied (ticket "Scheme: …" + settle total).
    pub fn order_bill_draft_schemed(
        &self,
        order: &RunningOrder,
    ) -> (Option<SaleDraft>, Vec<String>) {
        let mut cart = crate::domain::Cart::default();
        cart.lines = order.to_cart_lines();
        let notes = self.apply_schemes_to_cart(&mut cart).unwrap_or_default();
        (cart.to_sale_draft().ok(), notes)
    }
}

fn right_words(right: Right) -> &'static str {
    match right {
        Right::Settle => "Taking payment",
        Right::CancelSentItem => "Cancelling an item the kitchen has",
        Right::Complimentary => "Giving an item free (NC)",
        Right::MoveTable => "Moving or merging tables",
        Right::Reprint => "Printing the bill again",
        Right::Discount => "Giving a discount",
    }
}

#[cfg(test)]
mod tests {
    use super::natural_key;
    use crate::domain::{
        new_id, Cart, Money, PayMethod, Product, RightLevel, RunningOrderStatus, StaffMember,
        StaffRole, TableStatus,
    };
    use crate::store::Store;

    #[test]
    fn tables_sort_by_number_not_text() {
        let mut names = vec!["D10", "D2", "P1", "D1", "AC 12", "AC 3"];
        names.sort_by_key(|n| natural_key(n));
        assert_eq!(names, vec!["AC 3", "AC 12", "D1", "D2", "D10", "P1"]);
    }

    struct Shop {
        store: Store,
        path: std::path::PathBuf,
        owner: StaffMember,
        cashier: StaffMember,
        waiter: StaffMember,
        thali: Product,
        chaas: Product,
    }

    impl Drop for Shop {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    /// A restaurant with an owner, a manager (PIN 9999), a cashier, a
    /// waiter, two dishes, and Dine In D1–D3 + Parcel P1–P2.
    fn shop() -> Shop {
        let path = std::env::temp_dir().join(format!("khaynjo-running-{}.sqlite3", new_id()));
        let store = Store::open_path(&path).unwrap();
        let owner = store.list_staff().unwrap().into_iter().next().unwrap();
        store.add_staff("Mona", StaffRole::Manager, "Manager", "9999", &owner).unwrap();
        store.add_staff("Cash", StaffRole::Cashier, "Cashier", "1111", &owner).unwrap();
        store.add_staff("Wasim", StaffRole::Waiter, "Captain", "2222", &owner).unwrap();
        let staff = store.list_staff().unwrap();
        let find = |n: &str| staff.iter().find(|s| s.name == n).unwrap().clone();
        for (name, rupees) in [("Thali", 200), ("Chaas", 30)] {
            store
                .upsert_product(&Product::new_basic(new_id(), name, Money::from_rupees(rupees)))
                .unwrap();
        }
        let products = store.list_products(false).unwrap();
        let product = |n: &str| products.iter().find(|p| p.name == n).unwrap().clone();
        store.add_tables_range("D", 1, 3, "Dine In").unwrap();
        store.add_tables_range("P", 1, 2, "Parcel").unwrap();
        Shop {
            cashier: find("Cash"),
            waiter: find("Wasim"),
            thali: product("Thali"),
            chaas: product("Chaas"),
            owner,
            store,
            path,
        }
    }

    impl Shop {
        fn table(&self, name: &str) -> crate::domain::DiningTable {
            self.store
                .list_tables()
                .unwrap()
                .into_iter()
                .find(|t| t.name == name)
                .unwrap()
        }

        /// Waiter punches `items` on `table` and sends the KOT.
        fn kot(&self, table: &str, items: &[(&Product, usize)]) -> Vec<String> {
            let mut cart = Cart::default();
            for (p, n) in items {
                for _ in 0..*n {
                    cart.add_or_bump(p);
                }
            }
            let draft = cart.to_kot_draft().unwrap();
            self.store
                .send_kot(&draft, Some(&self.table(table).id), &self.waiter)
                .unwrap()
        }

        /// The counter opens the table: its running order as a cart.
        fn bill_cart(&self, table: &str) -> Cart {
            let order = self.store.running_order(&self.table(table).id).unwrap().unwrap();
            let mut cart = Cart::default();
            cart.lines = order.to_cart_lines();
            cart
        }
    }

    /// Punch pending on D1, open D2, reopen D1 — pending still there; Send KOT
    /// promotes; clearing the last pending frees the table.
    #[test]
    fn pending_lines_survive_table_bounce_and_clear_frees_the_table() {
        let shop = shop();
        let d1 = shop.table("D1");
        let waiter = shop.waiter.clone();
        shop.store
            .bump_pending_product(&d1.id, &shop.thali, 2000, "", &waiter)
            .unwrap();
        shop.store
            .bump_pending_product(&d1.id, &shop.chaas, 1000, "", &waiter)
            .unwrap();
        assert_eq!(shop.table("D1").status, TableStatus::Occupied);
        assert_eq!(
            shop.table("D1").running_total_paise,
            Money::from_rupees(430).paise()
        );
        // Open D2 (another table) — D1's pending stays on the Server.
        shop.store
            .bump_pending_product(&shop.table("D2").id, &shop.chaas, 1000, "", &waiter)
            .unwrap();
        let order = shop.store.running_order(&d1.id).unwrap().unwrap();
        let pending: Vec<_> = order.live_lines().filter(|l| l.kot_id.is_empty()).collect();
        assert_eq!(pending.len(), 2);
        assert!(pending.iter().all(|l| l.kot_id.is_empty()));

        // Send KOT promotes pending → kot_id set; no duplicate unsent rows.
        let mut cart = Cart::default();
        cart.lines = order.to_cart_lines();
        let draft = cart.to_kot_draft().unwrap();
        shop.store.send_kot(&draft, Some(&d1.id), &waiter).unwrap();
        let order = shop.store.running_order(&d1.id).unwrap().unwrap();
        assert!(order.live_lines().all(|l| !l.kot_id.is_empty()));
        assert_eq!(order.live_lines().count(), 2);

        // Fresh pending then remove it — table frees when nothing live remains
        // only after settle; dropping the last *pending-only* order frees via
        // release_if_empty when qty goes to zero before any KOT.
        let d3 = shop.table("D3");
        shop.store
            .bump_pending_product(&d3.id, &shop.chaas, 1000, "", &waiter)
            .unwrap();
        assert_eq!(shop.table("D3").status, TableStatus::Occupied);
        shop.store
            .bump_pending_product(&d3.id, &shop.chaas, -1000, "", &waiter)
            .unwrap();
        assert!(shop.store.running_order(&d3.id).unwrap().is_none());
        assert_eq!(shop.table("D3").status, TableStatus::Available);
    }

    /// Petpooja's core loop: KOT, another KOT, bill — the counter bills
    /// exactly what the waiter sent, and the kitchen is not told twice.
    #[test]
    fn table_runs_kot_by_kot_and_the_counter_bills_what_was_sent() {
        let mut shop = shop();
        shop.kot("D1", &[(&shop.thali, 2)]);
        shop.kot("D1", &[(&shop.chaas, 1)]);
        let d1 = shop.table("D1");
        assert_eq!(d1.status, TableStatus::Occupied);
        assert_eq!(d1.area, "Dine In");
        assert_eq!(d1.running_total_paise, Money::from_rupees(430).paise());
        let kots_before = shop.store.list_open_kots().unwrap().len();
        assert_eq!(kots_before, 2);

        // A new tap on a sent dish is a new line, never a bump of history.
        let mut cart = shop.bill_cart("D1");
        cart.add_or_bump(&shop.thali);
        assert_eq!(cart.lines.len(), 3);
        cart.bump_qty(0, -1); // a sent line does not shrink without a reason
        assert_eq!(cart.lines[0].qty, 2);
        cart.lines.pop();

        let draft = cart.to_sale_draft().unwrap();
        assert!(draft.lines.iter().all(|l| l.kot_sent));
        let owner = shop.owner.clone();
        let sale = shop
            .store
            .finalize_sale(&draft, PayMethod::Cash, Some(&d1.id), None, &owner)
            .unwrap();
        assert_eq!(sale.total_paise, Money::from_rupees(430).paise());
        assert_eq!(shop.store.list_open_kots().unwrap().len(), kots_before, "no second KOT");
        let d1 = shop.table("D1");
        assert_eq!(d1.status, TableStatus::Available);
        assert!(shop.store.running_order(&d1.id).unwrap().is_none());
    }

    /// The kitchen sees how much of each dish is still to cook, over every
    /// open ticket; a ready ticket drops out.
    #[test]
    fn the_kitchen_counts_each_dish_still_to_cook() {
        let shop = shop();
        let first = shop.kot("D1", &[(&shop.thali, 2)]);
        shop.kot("D2", &[(&shop.thali, 1), (&shop.chaas, 3)]);
        let counts = shop.store.kitchen_all_day().unwrap();
        assert_eq!(counts, vec![("Chaas".to_string(), 3000), ("Thali".to_string(), 3000)]);
        let owner = shop.owner.clone();
        shop.store.advance_kot(&first[0], &owner).unwrap(); // preparing: still to cook
        shop.store.advance_kot(&first[0], &owner).unwrap(); // ready: done cooking
        let counts = shop.store.kitchen_all_day().unwrap();
        assert!(counts.contains(&("Thali".to_string(), 1000)), "{counts:?}");
    }

    /// Two guests on D1 pay separately: the chaas goes to a separate bill on
    /// D1/2 (same area), each bill is paid on its own, and D1/2 goes away
    /// once paid.
    #[test]
    fn a_split_bill_is_paid_separately_and_its_companion_table_goes() {
        let mut shop = shop();
        shop.kot("D1", &[(&shop.thali, 2), (&shop.chaas, 1)]);
        let d1 = shop.table("D1");
        let order = shop.store.running_order(&d1.id).unwrap().unwrap();
        let all: Vec<String> = order.live_lines().map(|l| l.id.clone()).collect();
        let owner = shop.owner.clone();
        assert!(shop.store.split_running_lines(&all, &owner).is_err(), "something stays on D1");
        let chaas: Vec<String> = order.live_lines().filter(|l| l.name == "Chaas").map(|l| l.id.clone()).collect();
        let companion = shop.store.split_running_lines(&chaas, &owner).unwrap();
        assert_eq!(companion.name, "D1/2");
        assert_eq!(companion.area, "Dine In");
        assert_eq!(shop.table("D1").running_total_paise, Money::from_rupees(400).paise());
        assert_eq!(shop.table("D1/2").running_total_paise, Money::from_rupees(30).paise());

        let cart = shop.bill_cart("D1/2");
        let sale = shop
            .store
            .finalize_sale(&cart.to_sale_draft().unwrap(), PayMethod::Upi, Some(&companion.id), None, &owner)
            .unwrap();
        assert_eq!(sale.total_paise, Money::from_rupees(30).paise());
        shop.store.tidy_split_tables().unwrap();
        assert!(shop.store.list_tables().unwrap().iter().all(|t| t.name != "D1/2"));
        // D1's own bill is untouched and still open.
        assert!(shop.store.running_order(&d1.id).unwrap().is_some());
        let _ = crate::print_share::take_captured();
    }

    /// Guests sit before any dish is punched: Free → Seated (no order) →
    /// In kitchen on first KOT; an empty seated table frees again.
    #[test]
    fn seating_marks_occupied_without_an_order_then_frees_or_cooks() {
        use crate::domain::FloorState;
        let shop = shop();
        let d2 = shop.table("D2");
        assert_eq!(d2.floor_state, FloorState::Free);
        shop.store.seat_table(&d2.id, &shop.owner).unwrap();
        let d2 = shop.table("D2");
        assert_eq!(d2.status, TableStatus::Occupied);
        assert_eq!(d2.floor_state, FloorState::Seated);
        assert!(shop.store.running_order(&d2.id).unwrap().is_none());
        // Leave without ordering.
        shop.store.free_table(&d2.id).unwrap();
        assert_eq!(shop.table("D2").floor_state, FloorState::Free);
        // Sit again, then send a KOT — Seated becomes In kitchen.
        let d2 = shop.table("D2");
        shop.store.seat_table(&d2.id, &shop.owner).unwrap();
        assert_eq!(shop.table("D2").floor_state, FloorState::Seated);
        shop.kot("D2", &[(&shop.thali, 1)]);
        assert_eq!(shop.table("D2").floor_state, FloorState::InKitchen);
        assert!(shop.store.running_order(&shop.table("D2").id).unwrap().is_some());
    }

    /// The floor follows the guests' visit, the same on every device:
    /// free → in kitchen → food ready → eating → bill printed → paid → free.
    #[test]
    fn the_floor_follows_the_visit_from_free_to_paid_and_back() {
        use crate::domain::FloorState;
        let mut shop = shop();
        let state = |shop: &Shop| shop.table("D2").floor_state;
        assert_eq!(state(&shop), FloorState::Free);
        let kots = shop.kot("D2", &[(&shop.thali, 1)]);
        assert_eq!(state(&shop), FloorState::InKitchen);
        let owner = shop.owner.clone();
        shop.store.advance_kot(&kots[0], &owner).unwrap(); // preparing
        assert_eq!(state(&shop), FloorState::InKitchen);
        shop.store.advance_kot(&kots[0], &owner).unwrap(); // ready
        assert_eq!(state(&shop), FloorState::FoodReady);
        shop.store.advance_kot(&kots[0], &owner).unwrap(); // served
        assert_eq!(state(&shop), FloorState::Eating);
        let d2 = shop.table("D2");
        shop.store.print_running_bill(&d2.id, &owner, None).unwrap();
        assert_eq!(state(&shop), FloorState::BillPrinted);
        let draft = shop.bill_cart("D2").to_sale_draft().unwrap();
        shop.store
            .finalize_sale(&draft, PayMethod::Cash, Some(&d2.id), None, &owner)
            .unwrap();
        // Payment frees the table at once — no stuck Paid / Ready step.
        assert_eq!(state(&shop), FloorState::Free);
        // …and seating new guests on a paid table starts a fresh visit.
        shop.kot("D2", &[(&shop.chaas, 1)]);
        let d2 = shop.table("D2");
        let draft = shop.bill_cart("D2").to_sale_draft().unwrap();
        shop.store
            .finalize_sale(&draft, PayMethod::Cash, Some(&d2.id), None, &owner)
            .unwrap();
        shop.kot("D2", &[(&shop.thali, 1)]);
        let d2 = shop.table("D2");
        assert_eq!(d2.floor_state, FloorState::InKitchen);
        assert_eq!(d2.running_total_paise, Money::from_rupees(200).paise(), "only the new visit");
        // Accidental Print bill can be undone; empty bill-print frees.
        let d3 = shop.table("D3");
        shop.kot("D3", &[(&shop.chaas, 1)]);
        shop.store
            .print_running_bill(&d3.id, &owner, None)
            .unwrap();
        assert_eq!(shop.table("D3").floor_state, FloorState::BillPrinted);
        shop.store.undo_bill_print(&d3.id, &owner).unwrap();
        // KOT still open in the kitchen — back to In kitchen, not stuck Bill printed.
        assert_eq!(shop.table("D3").floor_state, FloorState::InKitchen);
        shop.store
            .print_running_bill(&d3.id, &owner, None)
            .unwrap();
        let line = shop
            .store
            .running_order(&d3.id)
            .unwrap()
            .unwrap()
            .live_lines()
            .next()
            .unwrap()
            .id
            .clone();
        shop.store
            .cancel_running_line(&line, None, "Mistake", false, &owner, None)
            .unwrap();
        assert_eq!(shop.table("D3").floor_state, FloorState::Free);
    }

    #[test]
    fn a_waiter_takes_orders_but_does_not_settle() {
        let mut shop = shop();
        shop.kot("D2", &[(&shop.thali, 1)]);
        let d2 = shop.table("D2");
        let draft = shop.bill_cart("D2").to_sale_draft().unwrap();
        let waiter = shop.waiter.clone();
        let err = shop
            .store
            .finalize_sale(&draft, PayMethod::Cash, Some(&d2.id), None, &waiter)
            .unwrap_err();
        assert!(err.contains("not allowed"), "{err}");
        let cashier = shop.cashier.clone();
        shop.store
            .finalize_sale(&draft, PayMethod::Cash, Some(&d2.id), None, &cashier)
            .unwrap();
    }

    /// Paying (or freeing) a table must not leave open KOTs that paint the
    /// *next* seating as "In kitchen" with nothing on the ticket.
    #[test]
    fn paying_clears_in_kitchen_so_the_next_guests_start_free() {
        use crate::domain::FloorState;
        let mut shop = shop();
        shop.kot("D2", &[(&shop.thali, 1)]);
        assert_eq!(shop.table("D2").floor_state, FloorState::InKitchen);
        let d2 = shop.table("D2");
        let draft = shop.bill_cart("D2").to_sale_draft().unwrap();
        let owner = shop.owner.clone();
        shop.store
            .finalize_sale(&draft, PayMethod::Cash, Some(&d2.id), None, &owner)
            .unwrap();
        assert_eq!(shop.table("D2").floor_state, FloorState::Free);
        assert!(
            shop.store
                .list_open_kots()
                .unwrap()
                .iter()
                .all(|k| k.table_id != d2.id),
            "no open KOT may still name the freed table"
        );
        // Next guests sit with no order — must be Seated, not In kitchen.
        shop.store.seat_table(&d2.id, &owner).unwrap();
        assert_eq!(shop.table("D2").floor_state, FloorState::Seated);
        shop.store.free_table(&d2.id).unwrap();
        assert_eq!(shop.table("D2").floor_state, FloorState::Free);
    }

    /// A cashier can bump a kitchen ticket (one-device shops clear "In kitchen"
    /// without a Kitchen role login).
    #[test]
    fn a_cashier_can_advance_a_kitchen_ticket() {
        let shop = shop();
        let kots = shop.kot("D1", &[(&shop.thali, 1)]);
        shop.store
            .advance_kot(&kots[0], &shop.cashier)
            .expect("cashier advances New → Preparing");
    }

    /// Cancel after KOT: needs a reason and (for a cashier) a manager's PIN;
    /// part of a line can go; the kitchen gets a cancel slip; nothing is
    /// deleted and the report shows the wastage.
    #[test]
    fn cancelling_a_sent_item_is_reasoned_approved_and_kept() {
        let shop = shop();
        shop.kot("D1", &[(&shop.thali, 3)]);
        let d1 = shop.table("D1");
        let line = shop.store.running_order(&d1.id).unwrap().unwrap().lines[0].clone();
        let s = &shop.store;
        assert!(s.cancel_running_line(&line.id, Some(1000), "", true, &shop.cashier, None).is_err());
        let err = s
            .cancel_running_line(&line.id, Some(1000), "Guest changed mind", true, &shop.cashier, None)
            .unwrap_err();
        assert!(err.contains("approval"), "{err}");
        assert!(s
            .cancel_running_line(&line.id, Some(1000), "Guest changed mind", true, &shop.cashier, Some("1111"))
            .is_err(), "a cashier cannot approve");
        let slip = s
            .cancel_running_line(&line.id, Some(1000), "Guest changed mind", true, &shop.cashier, Some("9999"))
            .unwrap();
        let order = s.running_order(&d1.id).unwrap().unwrap();
        assert_eq!(order.lines.len(), 2, "the cancelled part is its own kept line");
        assert_eq!(order.lines[0].qty_milli, 2000);
        assert!(order.lines[1].cancelled && order.lines[1].prepared);
        assert_eq!(order.lines[1].approved_by, "Mona");
        assert_eq!(shop.table("D1").running_total_paise, Money::from_rupees(400).paise());
        let text = s.format_kot_text(&slip).unwrap();
        assert!(text.contains("CANCEL") && text.contains("Thali"), "{text}");
        let report = s.cancellations_and_nc_text().unwrap();
        assert!(report.contains("Guest changed mind") && report.contains("cooked"), "{report}");
    }

    #[test]
    fn complimentary_items_are_free_on_the_bill_and_visible_in_reports() {
        let mut shop = shop();
        shop.kot("D3", &[(&shop.thali, 1), (&shop.chaas, 2)]);
        let d3 = shop.table("D3");
        let chaas = shop
            .store
            .running_order(&d3.id)
            .unwrap()
            .unwrap()
            .lines
            .into_iter()
            .find(|l| l.name == "Chaas")
            .unwrap();
        let waiter = shop.waiter.clone();
        assert!(shop
            .store
            .set_running_line_complimentary(&chaas.id, true, "Regular guest", &waiter, Some("9999"))
            .is_err(), "a waiter may not give food away, even with a PIN");
        let owner = shop.owner.clone();
        shop.store
            .set_running_line_complimentary(&chaas.id, true, "Regular guest", &owner, None)
            .unwrap();
        assert_eq!(shop.table("D3").running_total_paise, Money::from_rupees(200).paise());
        let bill = shop
            .store
            .print_running_bill(&d3.id, &owner, None)
            .unwrap();
        assert!(bill.contains("NC") && bill.contains("Chaas"), "{bill}");
        let draft = shop.bill_cart("D3").to_sale_draft().unwrap();
        let sale = shop
            .store
            .finalize_sale(&draft, PayMethod::Upi, Some(&d3.id), None, &owner)
            .unwrap();
        assert_eq!(sale.total_paise, Money::from_rupees(200).paise());
        let report = shop.store.cancellations_and_nc_text().unwrap();
        assert!(report.contains("Regular guest"), "{report}");
    }

    #[test]
    fn a_printed_bill_with_nothing_to_charge_still_frees_the_table() {
        // A dish still at ₹0 (fresh from the template) or an all-NC order:
        // Pay cannot close a zero sale, so list_tables heals Bill-printed
        // with nothing to collect (same end as Ready on the Floor).
        let shop = shop();
        let water = Product::new_basic(new_id(), "Water", Money(0));
        shop.store.upsert_product(&water).unwrap();
        shop.kot("D2", &[(&water, 1)]);
        let d2 = shop.table("D2");
        shop.store.print_running_bill(&d2.id, &shop.owner, None).unwrap();
        assert_eq!(shop.table("D2").status, TableStatus::Available);
        assert!(shop.store.running_order(&d2.id).unwrap().is_none());
    }

    #[test]
    fn cancelling_everything_sent_frees_the_table() {
        let shop = shop();
        shop.kot("D3", &[(&shop.thali, 1)]);
        let d3 = shop.table("D3");
        let line = shop.store.running_order(&d3.id).unwrap().unwrap().live_lines().next().unwrap().id.clone();
        shop.store
            .cancel_running_line(&line, None, "Guest left", false, &shop.owner, None)
            .unwrap();
        // Nothing left to cook or to bill: the table is free again, not
        // "Eating" (or "Bill printed") with an empty order.
        assert_eq!(shop.table("D3").status, TableStatus::Available);
        assert!(shop.store.running_order(&d3.id).unwrap().is_none());
    }

    #[test]
    fn printing_the_bill_marks_the_table_and_reprint_is_a_right() {
        let shop = shop();
        shop.kot("D1", &[(&shop.thali, 1)]);
        let d1 = shop.table("D1");
        shop.store.print_running_bill(&d1.id, &shop.cashier, None).unwrap();
        assert_eq!(shop.table("D1").status, TableStatus::PaymentPending);
        assert_eq!(
            shop.store.running_order(&d1.id).unwrap().unwrap().status,
            RunningOrderStatus::Printed
        );
        assert!(shop.store.print_running_bill(&d1.id, &shop.waiter, None).is_err());
        shop.store.print_running_bill(&d1.id, &shop.cashier, None).unwrap();
        // More food after the print: back to running, bill must print again.
        shop.kot("D1", &[(&shop.chaas, 1)]);
        assert_eq!(shop.table("D1").status, TableStatus::Occupied);
        // Leaving is not paying.
        assert!(shop.store.free_table(&d1.id).is_err());
    }

    #[test]
    fn moving_and_merging_keeps_one_order_and_frees_the_old_table() {
        let shop = shop();
        shop.kot("D1", &[(&shop.thali, 1)]);
        let (d1, d2, d3) = (shop.table("D1"), shop.table("D2"), shop.table("D3"));
        let order_id = shop.store.running_order(&d1.id).unwrap().unwrap().id;
        let s = &shop.store;
        s.move_table(&d1.id, &d2.id, &shop.waiter, None).unwrap();
        assert_eq!(shop.table("D1").status, TableStatus::Available);
        let moved = s.running_order(&d2.id).unwrap().unwrap();
        assert_eq!(moved.id, order_id, "same order, new table");
        assert_eq!(shop.table("D2").occupied_at, d1.occupied_at, "minutes travel");

        shop.kot("D3", &[(&shop.chaas, 2)]);
        assert!(s.move_table(&d3.id, &d2.id, &shop.waiter, None).is_err(), "busy: merge instead");
        s.merge_tables(&d3.id, &d2.id, &shop.waiter, None).unwrap();
        assert_eq!(shop.table("D3").status, TableStatus::Available);
        assert_eq!(
            shop.table("D2").running_total_paise,
            Money::from_rupees(260).paise()
        );

        // Move one item to a parcel: the parcel opens, D2 keeps the rest.
        let thali = s.running_order(&d2.id).unwrap().unwrap().lines[0].id.clone();
        let p1 = shop.table("P1");
        s.move_running_lines(&[thali], &p1.id, &shop.waiter, None).unwrap();
        assert_eq!(shop.table("P1").running_total_paise, Money::from_rupees(200).paise());
        assert_eq!(shop.table("D2").running_total_paise, Money::from_rupees(60).paise());
    }

    #[test]
    fn a_table_with_nothing_to_charge_releases_without_a_bill() {
        let shop = shop();
        shop.kot("D1", &[(&shop.chaas, 1)]);
        let d1 = shop.table("D1");
        let line = shop.store.running_order(&d1.id).unwrap().unwrap().lines[0].id.clone();
        shop.store
            .set_running_line_complimentary(&line, true, "Water for driver", &shop.owner, None)
            .unwrap();
        shop.store.free_table(&d1.id).unwrap();
        assert_eq!(shop.table("D1").status, TableStatus::Available);
        assert!(shop.store.running_order(&d1.id).unwrap().is_none());
    }

    /// Each variant is an item of its own (UUID, price, station, stock);
    /// quick entry makes them, the group only labels them; an AC table pays
    /// the AC price.
    #[test]
    fn variants_are_items_of_their_own_and_area_prices_price_the_line() {
        use crate::domain::ModifierOption;
        let mut shop = shop();
        let mut base = Product::new_basic(new_id(), "Paneer Tikka", Money::from_rupees(200));
        base.set_area_prices(&[ModifierOption { name: "AC".into(), price_paise: Money::from_rupees(150).paise() , group: String::new() }]);
        let owner = shop.owner.clone();
        let made = shop
            .store
            .add_variant_items(
                &base,
                &[
                    ModifierOption { name: "Half".into(), price_paise: Money::from_rupees(120).paise() , group: String::new() },
                    ModifierOption { name: "Full".into(), price_paise: Money::from_rupees(200).paise() , group: String::new() },
                ],
                None,
                &owner,
            )
            .unwrap();
        assert_eq!(made.len(), 2);
        assert_ne!(made[0].id, made[1].id, "each variant has its own UUID");
        assert!(made.iter().all(|p| p.id != base.id));
        let half = shop.store.get_product(&made[0].id).unwrap().unwrap();
        let full = shop.store.get_product(&made[1].id).unwrap().unwrap();
        assert_eq!((half.name.as_str(), half.price), ("Paneer Tikka (Half)", Money::from_rupees(120)));
        assert_eq!(full.group_name, "Paneer Tikka");
        // Each variant is edited on its own: sold out Full leaves Half on sale.
        shop.store.set_product_available(&full.id, false).unwrap();
        let mut cart = Cart::default();
        cart.add_or_bump_in_stock(&half).unwrap();
        cart.add_or_bump_in_stock(&half).unwrap();
        let full = shop.store.get_product(&full.id).unwrap().unwrap();
        assert!(cart.add_or_bump_in_stock(&full).is_err());
        // The base's AC price was for the full dish — it is not copied; each
        // variant gets its own area prices.
        assert_eq!(half.price_in_area("ac"), Money::from_rupees(120));
        assert!(half.aliases().is_empty());
        assert_eq!(cart.lines.len(), 1);
        assert_eq!(cart.applied_total(), Money::from_rupees(240));
        let d1 = shop.table("D1");
        let draft = cart.to_sale_draft().unwrap();
        shop.store
            .finalize_sale(&draft, PayMethod::Cash, Some(&d1.id), None, &owner)
            .unwrap();
        let report = shop.store.restaurant_day_report_text().unwrap();
        assert!(report.contains("By dish group") && report.contains("Paneer Tikka · ×2"), "{report}");
    }

    #[test]
    fn owner_set_rights_override_the_role_preset() {
        let shop = shop();
        let mut features = shop.waiter.features.clone();
        let mut rights = shop.waiter.rights();
        rights.settle = RightLevel::Allowed;
        features.rights = Some(rights);
        shop.store
            .set_staff_features(&shop.waiter.id, &features, &shop.owner)
            .unwrap();
        let waiter = shop
            .store
            .list_staff()
            .unwrap()
            .into_iter()
            .find(|s| s.name == "Wasim")
            .unwrap();
        assert!(shop.store.check_right(&waiter, crate::domain::Right::Settle, None).is_ok());
    }

    #[test]
    fn floor_ranges_skip_existing_tables_and_settlement_keeps_tips_apart() {
        let mut shop = shop();
        assert_eq!(shop.store.add_tables_range("D", 1, 5, "Dine In").unwrap(), 2);
        // Fresh shops seed T1–T6 in the unnamed area; ours follow it.
        assert_eq!(shop.store.list_areas().unwrap(), vec!["", "Dine In", "Parcel"]);
        let names: Vec<String> = shop
            .store
            .list_tables()
            .unwrap()
            .into_iter()
            .filter(|t| !t.area.is_empty())
            .map(|t| t.name)
            .collect();
        assert_eq!(names, vec!["D1", "D2", "D3", "D4", "D5", "P1", "P2"]);

        shop.kot("P1", &[(&shop.thali, 1)]);
        let p1 = shop.table("P1");
        let draft = shop.bill_cart("P1").to_sale_draft().unwrap();
        let owner = shop.owner.clone();
        let sale = shop
            .store
            .finalize_sale(&draft, PayMethod::Cash, Some(&p1.id), None, &owner)
            .unwrap();
        shop.store
            .record_settlement(&sale.id, Money::from_rupees(500).paise(), Money::from_rupees(250).paise(), Money::from_rupees(50).paise(), "cash", &owner)
            .unwrap();
        assert_eq!(shop.store.today_tips_paise().unwrap(), Money::from_rupees(50).paise());
        assert_eq!(shop.store.today_totals().unwrap().revenue_paise, Money::from_rupees(200).paise());
    }

    /// Punch without Send KOT, open another table, come back — pending still
    /// on D1. Send KOT promotes; clearing the last pending frees the table.
    #[test]
    fn pending_ticket_bounces_promotes_and_frees_when_empty() {
        let shop = shop();
        let d1 = shop.table("D1");
        let waiter = shop.waiter.clone();
        shop.store
            .bump_pending_product(&d1.id, &shop.thali, 1000, "", &waiter)
            .unwrap();
        shop.store
            .bump_pending_product(&d1.id, &shop.chaas, 1000, "", &waiter)
            .unwrap();
        assert_eq!(shop.table("D1").status, TableStatus::Occupied);
        assert_eq!(
            shop.table("D1").running_total_paise,
            Money::from_rupees(230).paise()
        );
        // Bounce: look at D2, then D1 again — pending still there.
        assert!(shop.store.running_order(&shop.table("D2").id).unwrap().is_none());
        let order = shop.store.running_order(&d1.id).unwrap().unwrap();
        let cart = {
            let mut c = Cart::default();
            c.lines = order.to_cart_lines();
            c
        };
        assert_eq!(cart.lines.len(), 2);
        assert!(cart.lines.iter().all(|l| !l.is_sent()));

        let draft = cart.to_kot_draft().unwrap();
        shop.store.send_kot(&draft, Some(&d1.id), &waiter).unwrap();
        let order = shop.store.running_order(&d1.id).unwrap().unwrap();
        assert!(order.live_lines().all(|l| !l.kot_id.is_empty()));
        let cart2 = {
            let mut c = Cart::default();
            c.lines = order.to_cart_lines();
            c
        };
        assert!(cart2.lines.iter().all(|l| l.is_sent()));

        // Fresh pending only, then remove — table frees.
        let d2 = shop.table("D2");
        shop.store
            .bump_pending_product(&d2.id, &shop.chaas, 1000, "", &waiter)
            .unwrap();
        assert_eq!(shop.table("D2").status, TableStatus::Occupied);
        shop.store
            .bump_pending_product(&d2.id, &shop.chaas, -1000, "", &waiter)
            .unwrap();
        assert_eq!(shop.table("D2").status, TableStatus::Available);
        assert!(shop.store.running_order(&d2.id).unwrap().is_none());
    }

    /// Change table: pending moves to the new table; sent stays on the old.
    #[test]
    fn reseat_pending_moves_unsent_and_leaves_sent() {
        let shop = shop();
        let d1 = shop.table("D1");
        let d2 = shop.table("D2");
        let waiter = shop.waiter.clone();
        shop.kot("D1", &[(&shop.thali, 1)]);
        shop.store
            .bump_pending_product(&d1.id, &shop.chaas, 1000, "", &waiter)
            .unwrap();
        shop.store
            .reseat_pending(&d1.id, &d2.id, &waiter)
            .unwrap();
        let on_d1 = shop.store.running_order(&d1.id).unwrap().unwrap();
        assert_eq!(on_d1.live_lines().count(), 1);
        assert!(on_d1.live_lines().all(|l| !l.kot_id.is_empty() && l.name == "Thali"));
        let on_d2 = shop.store.running_order(&d2.id).unwrap().unwrap();
        assert_eq!(on_d2.live_lines().count(), 1);
        assert!(on_d2.live_lines().all(|l| l.kot_id.is_empty() && l.name == "Chaas"));
        assert_eq!(shop.table("D2").status, TableStatus::Occupied);
    }
}
