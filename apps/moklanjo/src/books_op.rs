//! Person-level book writes. A Client sends one of these to the shop over
//! `POST /books`; the shop applies it. Device exceptions (backup folder,
//! extra tills, pairing code, forget device, restore) are not in this enum.

use crate::booking;
use crate::courier_ops_view;
use crate::domain::{
    self, AttachmentEntity, BookingMeasureSettings, ModifierOption, Money, MoneyMoveKind, PayMethod,
    Product, StaffFeatures, StaffMember, StaffRole,
};
use crate::fiscal::TaxPricing;
use crate::manifest::TrackingStatus;
use crate::precise;
use crate::store::{StaffHire, Store};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Splits an expense reply into the status line and the new expense id.
pub const EXPENSE_ID_MARK: &str = "\n---expense---\n";

/// Splits a hire reply into the status line and the new staff id.
pub const HIRED_STAFF_MARK: &str = "\n---hired---\n";

pub fn hired_staff_parts(message: &str) -> (&str, Option<&str>) {
    match message.split_once(HIRED_STAFF_MARK) {
        Some((status, id)) => (status, Some(id.trim())),
        None => (message, None),
    }
}

/// One person-level change to the shop's books.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum BooksOp {
    SetPlatformLanguage,
    SetUserLanguage {
        code: String,
    },
    SetCurrency {
        code: String,
    },
    CommitDayClose {
        counted_cash_paise: Option<i64>,
    },
    UpdateParty {
        id: String,
        name: String,
        phone: String,
        address: String,
        kind: String,
        image_bytes: Vec<u8>,
        image_filename: String,
        /// `None` leaves the logo. `Some("")` clears it. Any other key is stored.
        image_key: Option<String>,
    },
    AddParty {
        name: String,
        phone: String,
        address: String,
        kind: String,
        image_bytes: Vec<u8>,
        image_filename: String,
        image_key: Option<String>,
    },
    DeleteParty {
        id: String,
    },
    SetPartyField {
        id: String,
        key: String,
        value: String,
    },
    RemovePartyField {
        id: String,
        key: String,
    },
    SetCreditLimit {
        party_name: String,
        limit_paise: i64,
    },
    WriteOffCredit {
        customer_id: String,
        amount_paise: i64,
        reason: String,
    },
    SettlePayable {
        party_id: String,
        amount_paise: i64,
        pay: PayMethod,
        /// 1-based index into the open payable bills, when the desk typed a bill #.
        bill_number: Option<usize>,
    },
    WriteOffPayable {
        party_id: String,
        amount_paise: i64,
        reason: String,
    },
    PeriodBill {
        party_name: String,
    },
    RecordPurchase {
        supplier: String,
        party_kind: String,
        on_account: bool,
        product_id: String,
        qty_milli: i64,
        unit_cost_paise: i64,
        note: String,
    },
    SaveItem {
        updating: bool,
        product_id: String,
        name: String,
        price_paise: i64,
        unit_label: String,
        aliases: Vec<String>,
        tax_text: String,
        category_name: String,
        modifiers: Vec<ModifierOption>,
        first_kg: String,
        first_paise: i64,
        extra_per_kg_paise: i64,
        zone_name: String,
        carrier_name: String,
        party_name: String,
        image_bytes: Vec<u8>,
        image_filename: String,
        existing_image_key: Option<String>,
    },
    ArchiveProduct {
        id: String,
        name: String,
    },
    SetProductAvailable {
        id: String,
        name: String,
        available: bool,
    },
    ImportCsv {
        kind: String,
        filename: String,
        bytes: Vec<u8>,
    },
    AdjustStock {
        product_id: String,
        delta_qty: i64,
        note: String,
    },
    StartStockTracking {
        product_id: String,
        opening: i64,
    },
    HireStaff {
        name: String,
        role: StaffRole,
        role_title: String,
        pin: String,
        login_id: String,
        password: String,
        features: StaffFeatures,
    },
    SetStaffLogin {
        staff_id: String,
        login_id: String,
        password: String,
    },
    DeactivateStaff {
        staff_id: String,
        name: String,
    },
    SetStaffFeatures {
        staff_id: String,
        features: StaffFeatures,
    },
    SetDuressPin {
        pin: String,
        confirm: String,
    },
    ClearDuressPin,
    ApplyCountry {
        code: String,
    },
    SetTaxPricing {
        pricing: TaxPricing,
    },
    SetFiscal {
        business_tax_id: String,
        service_bps: i64,
    },
    /// Date order (`dmy`/`mdy`/`ymd`) + clock (`24h`/`12h`) — appearance only.
    SetDateTimeAppearance {
        date_order: String,
        time_format: String,
    },
    ImportReceiptStamp {
        filename: String,
        bytes: Vec<u8>,
    },
    ImportPaymentQr {
        filename: String,
        bytes: Vec<u8>,
    },
    ClearReceiptStamp,
    ClearPaymentQr,
    AdvanceTracking {
        awbs: Vec<String>,
        bulk: bool,
        status: String,
        note: String,
        pod_filename: String,
        pod_bytes: Vec<u8>,
    },
    FailedDelivery {
        awb: String,
        note: String,
    },
    Reattempt {
        awb: String,
    },
    SettleRiderCod {
        rider: String,
        handed: String,
    },
    DispatchFleet {
        rider: String,
        note: String,
    },
    CarrierHandover {
        carrier: String,
        note: String,
    },
    CollectCod {
        awb: String,
    },
    RemitCod {
        party: String,
    },
    AddRider {
        name: String,
        phone: String,
        vehicle: String,
    },
    PostMoney {
        kind: String,
        amount_paise: i64,
        note: String,
    },
    SetMovementCleared {
        number: usize,
        cleared: bool,
    },
    RenameBusiness {
        name: String,
    },
    SetVenueBrand {
        title: String,
        /// Empty bytes = leave the icon alone.
        icon_filename: String,
        icon_bytes: Vec<u8>,
    },
    ClearVenueIcon,
    AddAttachmentNote {
        entity: AttachmentEntity,
        entity_id: String,
        label: String,
        body: String,
    },
    AddAttachmentImage {
        entity: AttachmentEntity,
        entity_id: String,
        label: String,
        filename: String,
        bytes: Vec<u8>,
    },
    AddAttachmentFile {
        entity: AttachmentEntity,
        entity_id: String,
        label: String,
        filename: String,
        bytes: Vec<u8>,
    },
    SetBookingMeasures {
        settings: BookingMeasureSettings,
    },
    ImportBranchSnapshot {
        json: String,
    },
    AddUnit {
        code: String,
        name: String,
    },
    AddUnitConversion {
        from_code: String,
        to_code: String,
        factor_milli: i64,
    },
    AddAwbPool {
        carrier: String,
        prefix: String,
        start_n: i64,
        end_n: i64,
        label: String,
    },
    LogExpense {
        amount_paise: i64,
        note: String,
        category: String,
        parts: Vec<(PayMethod, i64)>,
    },
    /// Print a test page on the Server's printers (`receipt` / `label`).
    PrintTestPage {
        route_key: String,
    },
    SaveBackupCopy,
}

impl BooksOp {
    pub fn label(&self) -> String {
        match self {
            Self::SetPlatformLanguage | Self::SetUserLanguage { .. } | Self::SetCurrency { .. } => {
                "Change language"
            }
            Self::CommitDayClose { .. } => "Record the day close",
            Self::UpdateParty { .. }
            | Self::SetPartyField { .. }
            | Self::RemovePartyField { .. } => "Edit party",
            Self::AddParty { .. } => "Add customers",
            Self::DeleteParty { .. } => "Delete party",
            Self::SetCreditLimit { .. } => "Set credit limit",
            Self::WriteOffCredit { .. } => "Write off credit",
            Self::SettlePayable { .. } => "Settle payable",
            Self::WriteOffPayable { .. } => "Write off payable",
            Self::PeriodBill { .. } => "Period bill",
            Self::RecordPurchase { .. } => "Record purchases",
            Self::SaveItem { updating: true, .. } => "Change items",
            Self::SaveItem { .. } => "Add items",
            Self::ArchiveProduct { .. } => "Remove items",
            Self::SetProductAvailable { .. } => "Mark items sold out",
            Self::ImportCsv { .. } => "Import from CSV",
            Self::AdjustStock { .. } => "Adjust stock",
            Self::StartStockTracking { .. } => "Start stock tracking",
            Self::HireStaff { .. } => "Add staff",
            Self::SetStaffLogin { .. } => "Change staff logins",
            Self::DeactivateStaff { .. } => "Remove staff",
            Self::SetStaffFeatures { .. } => "Change staff permissions",
            Self::SetDuressPin { .. } => "Set emergency unlock PIN",
            Self::ClearDuressPin => "Clear emergency unlock PIN",
            Self::ApplyCountry { .. } => "Change the country pack",
            Self::SetTaxPricing { .. } => "Change tax pricing",
            Self::SetFiscal { .. } => "Set the business id",
            Self::SetDateTimeAppearance { .. } => "Change date and time appearance",
            Self::ImportReceiptStamp { .. } | Self::ClearReceiptStamp => "Change the receipt stamp",
            Self::ImportPaymentQr { .. } | Self::ClearPaymentQr => "Change the payment QR",
            Self::AdvanceTracking { status, .. } if status == "delivered" => "Mark delivered",
            Self::AdvanceTracking { .. } => "Advance tracking",
            Self::FailedDelivery { .. } => "Failed delivery",
            Self::Reattempt { .. } => "Reattempt",
            Self::SettleRiderCod { .. } => "Settle rider COD",
            Self::DispatchFleet { .. } => "Dispatch own fleet",
            Self::CarrierHandover { .. } => "Carrier handover",
            Self::CollectCod { .. } => "Collect COD",
            Self::RemitCod { .. } => "Remit COD",
            Self::AddRider { .. } => "Add rider",
            Self::PostMoney { .. } => "Post bank / cashbook",
            Self::SetMovementCleared { .. } => "Reconcile bank movements",
            Self::RenameBusiness { .. } => "Rename the business",
            Self::SetVenueBrand { .. } | Self::ClearVenueIcon => "Change in-app brand",
            Self::AddAttachmentNote { .. } => "Add note",
            Self::AddAttachmentImage { .. } => "Attach photo",
            Self::AddAttachmentFile { .. } => "Attach file",
            Self::SetBookingMeasures { .. } => "Save booking measures",
            Self::ImportBranchSnapshot { .. } => "Import a branch day",
            Self::AddUnit { .. } => "Add units",
            Self::AddUnitConversion { .. } => "Add unit conversion",
            Self::AddAwbPool { .. } => "Add AWB pool",
            Self::LogExpense { .. } => "Log expenses",
            Self::PrintTestPage { .. } => "Print a test page",
            Self::SaveBackupCopy => "Save a backup copy",
        }
        .to_string()
    }

    pub fn done_message(&self) -> String {
        match self {
            Self::SetPlatformLanguage => "Language follows this device".into(),
            Self::SetUserLanguage { code } => format!("Language: {code}"),
            Self::SetCurrency { code } => format!("Currency: {code}"),
            Self::CommitDayClose { .. } => "Day close recorded".into(),
            Self::UpdateParty { name, .. } => format!("Party updated: {}", name.trim()),
            Self::AddParty { name, .. } => format!("Added {}", name.trim()),
            Self::DeleteParty { .. } => "Party deleted".into(),
            Self::SetPartyField { key, .. } => format!("Saved field {}", key.trim()),
            Self::RemovePartyField { key, .. } => format!("Removed field {}", key.trim()),
            Self::SetCreditLimit { .. } => "Credit limit saved".into(),
            Self::WriteOffCredit { .. } => "Credit written off".into(),
            Self::SettlePayable { .. } => "Payable settled".into(),
            Self::WriteOffPayable { .. } => "Payable written off".into(),
            Self::PeriodBill { .. } => "Period bill saved".into(),
            Self::RecordPurchase { .. } => "Purchase recorded".into(),
            Self::SaveItem { name, updating, .. } => {
                if *updating {
                    format!("Updated {}", name.trim())
                } else {
                    format!("Added {}", name.trim())
                }
            }
            Self::ArchiveProduct { name, .. } => format!("Removed {name}"),
            Self::SetProductAvailable { name, available, .. } => {
                if *available {
                    format!("{name} restocked")
                } else {
                    format!("{name} marked sold out")
                }
            }
            Self::ImportCsv { kind, .. } => match kind.as_str() {
                "parties" => "Parties imported".into(),
                "pincodes" => "Pincodes imported".into(),
                "carrier_bill" => "Carrier bill imported".into(),
                _ => "Items imported".into(),
            },
            Self::AdjustStock { .. } => "Stock adjusted".into(),
            Self::StartStockTracking { .. } => "Stock tracking started".into(),
            Self::HireStaff { name, .. } => format!("Added {}", name.trim()),
            Self::SetStaffLogin { .. } => "Login saved".into(),
            Self::DeactivateStaff { name, .. } => format!("Removed {name}"),
            Self::SetStaffFeatures { .. } => "Staff features saved".into(),
            Self::SetDuressPin { .. } => "Emergency unlock PIN saved".into(),
            Self::ClearDuressPin => "Emergency unlock PIN turned off".into(),
            Self::ApplyCountry { .. } => "Country pack applied".into(),
            Self::SetTaxPricing { .. } => "Tax pricing saved".into(),
            Self::SetFiscal { .. } => "Business id saved".into(),
            Self::SetDateTimeAppearance { .. } => "Date and time appearance saved".into(),
            Self::ImportReceiptStamp { .. } => "Receipt stamp saved".into(),
            Self::ImportPaymentQr { .. } => "Payment QR saved".into(),
            Self::ClearReceiptStamp => "Stamp cleared".into(),
            Self::ClearPaymentQr => "Payment QR cleared".into(),
            Self::AdvanceTracking { status, .. } if status == "delivered" => "Marked delivered".into(),
            Self::AdvanceTracking { .. } => "Tracking advanced".into(),
            Self::FailedDelivery { .. } => "Failed delivery recorded".into(),
            Self::Reattempt { .. } => "Reattempt recorded".into(),
            Self::SettleRiderCod { .. } => "Rider COD settled".into(),
            Self::DispatchFleet { .. } => "Dispatched to rider".into(),
            Self::CarrierHandover { .. } => "Handed to carrier".into(),
            Self::CollectCod { .. } => "COD collected".into(),
            Self::RemitCod { .. } => "COD remitted".into(),
            Self::AddRider { name, .. } => format!("Rider added: {}", name.trim()),
            Self::PostMoney { .. } => "Bank movement posted".into(),
            Self::SetMovementCleared { cleared: true, .. } => "Marked cleared".into(),
            Self::SetMovementCleared { .. } => "Unmarked".into(),
            Self::RenameBusiness { .. } => "Business name saved".into(),
            Self::SetVenueBrand { .. } => "Brand saved".into(),
            Self::ClearVenueIcon => "Icon cleared".into(),
            Self::AddAttachmentNote { .. } => "Note added".into(),
            Self::AddAttachmentImage { .. } => "Photo attached (stored as WebP)".into(),
            Self::AddAttachmentFile { .. } => "File attached".into(),
            Self::SetBookingMeasures { .. } => "Booking measures saved".into(),
            Self::ImportBranchSnapshot { .. } => "Branch day imported".into(),
            Self::AddUnit { code, .. } => format!("Unit {code} added"),
            Self::AddUnitConversion { .. } => "Unit conversion added".into(),
            Self::AddAwbPool { .. } => "AWB pool added".into(),
            Self::LogExpense { .. } => "Expense logged".into(),
            Self::PrintTestPage { .. } => "Test page sent".into(),
            Self::SaveBackupCopy => "Backup saved on the shop device".into(),
        }
    }
}

struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

impl TempFile {
    fn path(&self) -> &Path {
        &self.0
    }
}

fn write_upload(filename: &str, bytes: &[u8]) -> Result<TempFile, String> {
    if bytes.is_empty() {
        return Err("Empty file".into());
    }
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty())
        .unwrap_or("bin");
    let path = std::env::temp_dir().join(format!("moklanjo-upload-{}.{}", domain::new_id(), ext));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(TempFile(path))
}

fn csv_summary(added: usize, skipped: usize, problems: &[String]) -> String {
    let mut msg = format!("Imported {added} new, skipped {skipped} (already there or unusable)");
    if let Some(p) = problems.first() {
        msg.push_str(&format!(" · first problem: {p}"));
    }
    msg
}

fn tracking_status(raw: &str) -> Result<TrackingStatus, String> {
    match raw {
        "out_for_delivery" => Ok(TrackingStatus::OutForDelivery),
        "delivered" => Ok(TrackingStatus::Delivered),
        other => Err(format!("Unknown tracking status '{other}'")),
    }
}

impl Store {
    fn show_money(&self, paise: i64) -> String {
        if let (Ok(loc), Ok(fiscal)) = (self.locale_settings(), self.fiscal_settings()) {
            Money(paise).format_localized(&loc.currency, &fiscal)
        } else {
            self.money_display(paise)
        }
    }

    fn apply_party_logo(
        &self,
        id: &str,
        image_bytes: &[u8],
        image_filename: &str,
        image_key: &Option<String>,
    ) -> Result<(), String> {
        if !image_bytes.is_empty() {
            let file = write_upload(image_filename, image_bytes)?;
            let imported = self.import_party_image(id, file.path())?;
            self.set_customer_image_key(id, Some(&imported))?;
        } else if let Some(key) = image_key {
            if key.is_empty() {
                self.set_customer_image_key(id, None)?;
            } else if !key.starts_with("pending_custom:") {
                self.set_customer_image_key(id, Some(key))?;
            }
        }
        Ok(())
    }

    fn save_item(
        &self,
        actor: &StaffMember,
        updating: bool,
        product_id: &str,
        name: &str,
        price_paise: i64,
        unit_label: &str,
        aliases: &[String],
        tax_text: &str,
        category_name: &str,
        modifiers: &[ModifierOption],
        first_kg: &str,
        first_paise: i64,
        extra_per_kg_paise: i64,
        zone_name: &str,
        carrier_name: &str,
        party_name: &str,
        image_bytes: &[u8],
        image_filename: &str,
        existing_image_key: Option<&str>,
    ) -> Result<String, String> {
        if !actor.is_owner() && !actor.can_access_items() {
            return Err("Not allowed to manage items".into());
        }
        let name = name.trim();
        if name.is_empty() {
            return Err("Name required".into());
        }
        if price_paise < 0 {
            return Err("Price cannot be negative".into());
        }
        let product_id = if product_id.is_empty() {
            domain::new_id()
        } else {
            product_id.to_string()
        };
        let mut image_key = existing_image_key.map(|s| s.to_string());
        if !image_bytes.is_empty() {
            let file = write_upload(image_filename, image_bytes)?;
            image_key = Some(self.import_item_image(&product_id, file.path())?);
        }
        let tax_rate = self.resolve_tax_rate(tax_text)?;
        let category_id = match category_name.trim() {
            "" => String::new(),
            wanted => match self
                .list_categories()?
                .into_iter()
                .find(|c| c.name.eq_ignore_ascii_case(wanted))
            {
                Some(c) => c.id,
                None => self.add_category(wanted)?.id,
            },
        };
        let mut product = if updating {
            self.get_product(&product_id)?
                .ok_or_else(|| "That item is no longer in Items".to_string())?
        } else {
            Product::new_basic(&product_id, name, Money(price_paise))
        };
        let was = if updating { Some(product.price) } else { None };
        product.name = name.to_string();
        product.price = Money(price_paise);
        product.unit_label = unit_label.trim().to_string();
        product.image_key = image_key;
        product.set_aliases(aliases);
        if let Some(rate) = &tax_rate {
            product.tax_rate_id = rate.id.clone();
            product.tax_bps = rate.bps;
        } else {
            product.tax_rate_id.clear();
            product.tax_bps = 0;
        }
        product.category_id = category_id;
        product.set_modifiers(modifiers);
        product.first_kg = booking::parse_positive(first_kg).unwrap_or(precise::ZERO);
        product.first_paise = first_paise;
        product.extra_per_kg_paise = extra_per_kg_paise;
        let zone_name = zone_name.trim();
        if zone_name.is_empty() {
            product.zone_id.clear();
        } else {
            let zones = self.list_zones()?;
            product.zone_id = if let Some(z) = zones.iter().find(|z| z.name.eq_ignore_ascii_case(zone_name))
            {
                z.id.clone()
            } else {
                self.add_zone(zone_name)?.id
            };
        }
        let parties = self.list_customers()?;
        let resolve_party = |typed: &str| -> Result<String, String> {
            let typed = typed.trim();
            if typed.is_empty() {
                return Ok(String::new());
            }
            if let Some(p) = parties
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case(typed) || p.id == typed)
            {
                return Ok(p.id.clone());
            }
            Err(format!("No party named “{typed}” — add them on Parties first"))
        };
        product.carrier_party_id = resolve_party(carrier_name)?;
        product.party_id = resolve_party(party_name)?;
        self.upsert_product(&product)?;
        let verb = if updating { "menu.update" } else { "menu.add" };
        let loc = self.locale_settings()?;
        let fiscal = self.fiscal_settings()?;
        let detail = match was {
            Some(old) if old != product.price => format!(
                "{} · {} → {}",
                product.name,
                old.format_localized(&loc.currency, &fiscal),
                product.price.format_localized(&loc.currency, &fiscal)
            ),
            _ => product.name.clone(),
        };
        self.audit(actor, verb, &detail)?;
        Ok(if updating {
            format!("Updated {}", product.name)
        } else {
            format!("Added {}", product.name)
        })
    }

    /// Apply one person-level write. Callers keep [`Self::assert_server_writer`]
    /// on the store methods this dispatches to.
    pub fn apply_books_op(&self, op: BooksOp, actor: &StaffMember) -> Result<String, String> {
        match op {
            BooksOp::SetPlatformLanguage => {
                self.set_platform_language()?;
                Ok("Language follows this device".into())
            }
            BooksOp::SetUserLanguage { code } => {
                self.set_user_language(&code)?;
                Ok(format!("Language: {code}"))
            }
            BooksOp::SetCurrency { code } => {
                if !actor.is_owner() {
                    return Err("Only the Owner can change currency".into());
                }
                let mut settings = self.locale_settings()?;
                settings.currency = code.clone();
                self.set_locale_settings(&settings)?;
                Ok(format!("Currency: {code}"))
            }
            BooksOp::CommitDayClose { counted_cash_paise } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to close the day".into());
                }
                let at = self.commit_day_close_with_count(actor, counted_cash_paise)?;
                let shown = at[..at.len().min(19)].replace('T', " ");
                Ok(format!("Day close recorded at {shown}"))
            }
            BooksOp::UpdateParty {
                id,
                name,
                phone,
                address,
                kind,
                image_bytes,
                image_filename,
                image_key,
            } => {
                if !actor.is_owner() && !actor.can_access_credit() {
                    return Err("Not allowed to edit parties".into());
                }
                self.update_customer(&id, &name, &phone, &address)?;
                self.set_customer_kind_label(&id, kind.trim())?;
                self.apply_party_logo(&id, &image_bytes, &image_filename, &image_key)?;
                Ok(format!("Party updated: {}", name.trim()))
            }
            BooksOp::AddParty {
                name,
                phone,
                address,
                kind,
                image_bytes,
                image_filename,
                image_key,
            } => {
                if !actor.is_owner() && !actor.can_access_credit() && !actor.can_access_sell() {
                    return Err("Not allowed to add parties".into());
                }
                let customer = self.add_customer_kind(&name, &phone, kind.trim())?;
                if !address.trim().is_empty() {
                    self.update_customer(&customer.id, &name, &phone, &address)?;
                }
                self.apply_party_logo(&customer.id, &image_bytes, &image_filename, &image_key)?;
                Ok(format!("Added {}", name.trim()))
            }
            BooksOp::DeleteParty { id } => {
                if !actor.is_owner() && !actor.can_access_credit() {
                    return Err("Not allowed to delete parties".into());
                }
                self.delete_customer(&id)
            }
            BooksOp::SetPartyField { id, key, value } => {
                if !actor.is_owner() && !actor.can_access_credit() {
                    return Err("Not allowed to edit parties".into());
                }
                self.set_customer_custom_field(&id, &key, &value)?;
                Ok(format!("Saved field {}", key.trim()))
            }
            BooksOp::RemovePartyField { id, key } => {
                if !actor.is_owner() && !actor.can_access_credit() {
                    return Err("Not allowed to edit parties".into());
                }
                self.remove_customer_custom_field(&id, &key)?;
                Ok(format!("Removed field {}", key.trim()))
            }
            BooksOp::SetCreditLimit {
                party_name,
                limit_paise,
            } => {
                if !actor.is_owner() && !actor.can_access_credit() {
                    return Err("Not allowed to set credit limits".into());
                }
                courier_ops_view::set_credit_limit(self, &party_name, limit_paise)
            }
            BooksOp::WriteOffCredit {
                customer_id,
                amount_paise,
                reason,
            } => {
                let written = self.write_off_credit(&customer_id, amount_paise, &reason, actor)?;
                Ok(format!("Wrote off {}", self.show_money(written)))
            }
            BooksOp::SettlePayable {
                party_id,
                amount_paise,
                pay,
                bill_number,
            } => {
                if !actor.is_owner()
                    && !actor.can_access_credit()
                    && !actor.can_access_inventory()
                {
                    return Err("Not allowed to settle payables".into());
                }
                let against = match bill_number {
                    None => None,
                    Some(n) => {
                        let bills = self.open_payable_bills(&party_id)?;
                        Some(
                            bills
                                .get(n.wrapping_sub(1))
                                .map(|b| b.bill_id.clone())
                                .ok_or_else(|| "No such open bill".to_string())?,
                        )
                    }
                };
                self.settle_payable_against(
                    &party_id,
                    amount_paise,
                    pay,
                    actor,
                    "",
                    against.as_deref(),
                )?;
                Ok(format!("AP paid {}", self.show_money(amount_paise)))
            }
            BooksOp::WriteOffPayable {
                party_id,
                amount_paise,
                reason,
            } => {
                let written = self.write_off_payable(&party_id, amount_paise, &reason, actor)?;
                Ok(format!("AP wrote off {}", self.show_money(written)))
            }
            BooksOp::PeriodBill { party_name } => {
                if !actor.is_owner()
                    && !actor.can_access_credit()
                    && !actor.can_access_sales()
                {
                    return Err("Not allowed to create period bills".into());
                }
                courier_ops_view::period_bill_party(self, &party_name, actor)
            }
            BooksOp::RecordPurchase {
                supplier,
                party_kind,
                on_account,
                product_id,
                qty_milli,
                unit_cost_paise,
                note,
            } => {
                let product = self
                    .get_product(&product_id)?
                    .ok_or_else(|| "Item not found".to_string())?;
                let mut party_id = String::new();
                if on_account {
                    let kind = if party_kind.trim().is_empty() {
                        "Supplier"
                    } else {
                        party_kind.trim()
                    };
                    party_id = self.add_customer_kind(supplier.trim(), "", kind)?.id;
                }
                let purchase = self.record_purchase_ex(
                    supplier.trim(),
                    &party_id,
                    on_account,
                    &[(product.id.clone(), qty_milli, unit_cost_paise)],
                    note.trim(),
                    actor,
                )?;
                let qty_show = domain::format_qty_display(qty_milli, 0, &product.unit_label);
                Ok(format!(
                    "Received {qty_show} {} · {}{}",
                    product.name,
                    self.show_money(purchase.total_paise),
                    if on_account { " · on account" } else { "" }
                ))
            }
            BooksOp::SaveItem {
                updating,
                product_id,
                name,
                price_paise,
                unit_label,
                aliases,
                tax_text,
                category_name,
                modifiers,
                first_kg,
                first_paise,
                extra_per_kg_paise,
                zone_name,
                carrier_name,
                party_name,
                image_bytes,
                image_filename,
                existing_image_key,
            } => self.save_item(
                actor,
                updating,
                &product_id,
                &name,
                price_paise,
                &unit_label,
                &aliases,
                &tax_text,
                &category_name,
                &modifiers,
                &first_kg,
                first_paise,
                extra_per_kg_paise,
                &zone_name,
                &carrier_name,
                &party_name,
                &image_bytes,
                &image_filename,
                existing_image_key.as_deref(),
            ),
            BooksOp::ArchiveProduct { id, name } => {
                if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to manage items".into());
                }
                self.archive_product(&id)?;
                self.audit(actor, "menu.remove", &name)?;
                Ok(format!("Removed {name}"))
            }
            BooksOp::SetProductAvailable { id, name, available } => {
                if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to manage items".into());
                }
                self.set_product_available(&id, available)?;
                Ok(if available {
                    format!("{name} restocked")
                } else {
                    format!("{name} marked sold out")
                })
            }
            BooksOp::ImportCsv { kind, bytes, .. } => {
                match kind.as_str() {
                    "parties" => {
                        if !actor.is_owner() && !actor.can_access_credit() {
                            return Err("Not allowed to import parties".into());
                        }
                    }
                    "items" | "" => {
                        if !actor.is_owner() && !actor.can_access_items() {
                            return Err("Not allowed to import menu CSV".into());
                        }
                    }
                    "pincodes" | "carrier_bill" => {
                        if !actor.is_owner() && !actor.can_access_sales() {
                            return Err("Not allowed to import CSV".into());
                        }
                    }
                    _ => {}
                }
                let text = String::from_utf8(bytes)
                    .map_err(|_| "CSV must be UTF-8 text".to_string())?;
                match kind.as_str() {
                    "parties" => {
                        let (added, skipped, problems) = self.import_parties_csv(&text)?;
                        Ok(csv_summary(added, skipped, &problems))
                    }
                    "pincodes" => {
                        let (added, skipped, problems) = self.import_pincodes_csv(&text)?;
                        Ok(csv_summary(added, skipped, &problems))
                    }
                    "carrier_bill" => {
                        let report = self.reconcile_carrier_bill_csv(&text)?;
                        let diffs = report
                            .lines()
                            .skip(1)
                            .filter(|l| {
                                l.contains("charge_diff")
                                    || l.contains("weight_diff")
                                    || l.contains("missing")
                            })
                            .count();
                        let (applied, skipped) = self.apply_carrier_bill_costs(&text)?;
                        Ok(format!(
                            "Carrier bill: {diffs} flagged · applied cost on {applied} · skipped {skipped}. Copy report from Reports."
                        ))
                    }
                    "items" | "" => {
                        let (added, skipped, problems) = self.import_items_csv(&text)?;
                        Ok(csv_summary(added, skipped, &problems))
                    }
                    other => Err(format!("Unknown CSV import '{other}'")),
                }
            }
            BooksOp::AdjustStock {
                product_id,
                delta_qty,
                note,
            } => {
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                let product = self
                    .get_product(&product_id)?
                    .ok_or_else(|| "Item not found".to_string())?;
                if !product.tracks_stock() {
                    return Err(format!(
                        "{} is not stock tracked — tap Start tracking first",
                        product.name
                    ));
                }
                let reason = if note.trim().is_empty() {
                    "adjust"
                } else {
                    note.trim()
                };
                let on_hand = self.adjust_stock(&product.id, delta_qty, reason, note.trim(), actor)?;
                Ok(format!("{} · {on_hand} on hand", product.name))
            }
            BooksOp::StartStockTracking { product_id, opening } => {
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                if opening < 0 {
                    return Err("Quantity cannot be negative".into());
                }
                let product = self
                    .get_product(&product_id)?
                    .ok_or_else(|| "Item not found".to_string())?;
                self.set_product_stock_qty(&product.id, 0)?;
                if opening > 0 {
                    self.adjust_stock(&product.id, opening, "count", "opening count", actor)?;
                }
                Ok(format!("{} now tracked · {opening} on hand", product.name))
            }
            BooksOp::HireStaff {
                name,
                role,
                role_title,
                pin,
                login_id,
                password,
                features,
            } => {
                let id = self.add_staff_with_features(
                    &StaffHire {
                        name: &name,
                        role,
                        role_title: &role_title,
                        pin: &pin,
                        login_id: &login_id,
                        password: &password,
                        features,
                    },
                    actor,
                )?;
                Ok(format!("Added {}{HIRED_STAFF_MARK}{id}", name.trim()))
            }
            BooksOp::SetStaffLogin {
                staff_id,
                login_id,
                password,
            } => {
                self.set_staff_login(&staff_id, &login_id, &password, actor)?;
                Ok("Login saved".into())
            }
            BooksOp::DeactivateStaff { staff_id, name } => {
                self.deactivate_staff(&staff_id, actor)?;
                Ok(format!("Removed {name}"))
            }
            BooksOp::SetStaffFeatures { staff_id, features } => {
                self.set_staff_features(&staff_id, &features, actor)?;
                Ok("Staff features saved".into())
            }
            BooksOp::SetDuressPin { pin, confirm } => {
                self.set_duress_pin(&pin, &confirm, actor)?;
                Ok("Emergency unlock PIN saved".into())
            }
            BooksOp::ClearDuressPin => {
                self.clear_duress_pin(actor)?;
                Ok("Emergency unlock PIN turned off".into())
            }
            BooksOp::ApplyCountry { code } => {
                if !actor.is_owner() {
                    return Err("Only the Owner can change the country pack".into());
                }
                let settings = self.apply_country_pack(&code)?;
                self.audit(
                    actor,
                    "fiscal.country",
                    &format!(
                        "{} {} {}",
                        settings.country_code,
                        settings.profile().tax_system.as_str(),
                        settings.tax_pricing.as_str()
                    ),
                )?;
                let name = settings.profile().name_en;
                Ok(format!("{name} pack applied — check rates on Items"))
            }
            BooksOp::SetTaxPricing { pricing } => {
                if !actor.is_owner() {
                    return Err("Only the Owner can change tax pricing".into());
                }
                self.set_tax_pricing(pricing)?;
                let tax = self.fiscal_settings()?.tax_label().to_string();
                Ok(pricing.label_for(&tax))
            }
            BooksOp::SetFiscal {
                business_tax_id,
                service_bps,
            } => {
                if !actor.is_owner() {
                    return Err("Only the Owner can set the business id".into());
                }
                self.set_business_tax_id(&business_tax_id)?;
                self.set_service_charge_bps(service_bps)?;
                let fiscal = self.fiscal_settings()?;
                let id_status = if fiscal.business_tax_id.is_empty() {
                    format!("{} cleared", fiscal.business_id_label)
                } else {
                    format!(
                        "{} saved — it prints on every bill",
                        fiscal.business_id_label
                    )
                };
                Ok(if fiscal.service_charge_bps > 0 {
                    format!(
                        "{id_status} · service {}% added to every bill",
                        crate::locale::format_percent_bps(
                            fiscal.service_charge_bps,
                            fiscal.decimal_sep()
                        )
                    )
                } else {
                    id_status
                })
            }
            BooksOp::SetDateTimeAppearance {
                date_order,
                time_format,
            } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change date and time appearance".into());
                }
                let settings = self.set_datetime_appearance(&date_order, &time_format)?;
                Ok(format!(
                    "Date and time appearance saved — example {ex}",
                    ex = settings.format_now()
                ))
            }
            BooksOp::ImportReceiptStamp { filename, bytes } => {
                // Stamp: Settings. Payment QR is Owner-only (below).
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change the receipt stamp".into());
                }
                let file = write_upload(&filename, &bytes)?;
                let _ = self.import_receipt_stamp(file.path())?;
                Ok("Receipt stamp saved".into())
            }
            BooksOp::ImportPaymentQr { filename, bytes } => {
                if !actor.is_owner() {
                    return Err("Only the Owner can change the payment QR".into());
                }
                let file = write_upload(&filename, &bytes)?;
                let _ = self.import_payment_qr(file.path())?;
                Ok("Payment QR saved".into())
            }
            BooksOp::ClearReceiptStamp => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change the receipt stamp".into());
                }
                self.set_receipt_stamp_key(None)?;
                Ok("Stamp cleared".into())
            }
            BooksOp::ClearPaymentQr => {
                if !actor.is_owner() {
                    return Err("Only the Owner can change the payment QR".into());
                }
                self.set_payment_qr_key(None)?;
                Ok("Payment QR cleared".into())
            }
            BooksOp::AdvanceTracking {
                awbs,
                bulk,
                status,
                note,
                pod_filename,
                pod_bytes,
            } => {
                // Same gate as Ops / Cod / Riders chrome (CollectCod pattern).
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to advance tracking".into());
                }
                let status = tracking_status(&status)?;
                if bulk {
                    let (ok, errs) = self.bulk_advance_tracking(&awbs, status, &note, actor)?;
                    let prefix = if status == TrackingStatus::Delivered {
                        "Bulk delivered"
                    } else {
                        "Bulk OFD"
                    };
                    let mut m = format!("{prefix} · ok {ok}");
                    for e in errs.iter().take(5) {
                        m.push('\n');
                        m.push_str(e);
                    }
                    Ok(m)
                } else {
                    let awb = awbs.first().map(String::as_str).unwrap_or("");
                    let pod = if pod_bytes.is_empty() {
                        None
                    } else {
                        Some(write_upload(&pod_filename, &pod_bytes)?)
                    };
                    courier_ops_view::advance_awb_with_pod(
                        self,
                        awb,
                        status,
                        &note,
                        pod.as_ref().map(|f| f.path()),
                        actor,
                    )
                }
            }
            BooksOp::FailedDelivery { awb, note } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to record failed delivery".into());
                }
                let (reason, next) = {
                    let raw = note.trim();
                    if let Some((r, d)) = raw.split_once('|') {
                        (r.trim().to_string(), d.trim().to_string())
                    } else {
                        (raw.to_string(), String::new())
                    }
                };
                let reason = if reason.is_empty() {
                    "other".into()
                } else {
                    reason
                };
                courier_ops_view::record_failed_delivery(self, &awb, &reason, &next, actor)
            }
            BooksOp::Reattempt { awb } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to reattempt delivery".into());
                }
                courier_ops_view::reattempt_awb(self, &awb, actor)
            }
            BooksOp::SettleRiderCod { rider, handed } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to settle rider COD".into());
                }
                courier_ops_view::settle_rider_cod_line(self, &rider, &handed, actor)
            }
            BooksOp::DispatchFleet { rider, note } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to dispatch fleet".into());
                }
                courier_ops_view::dispatch_booked_to_rider(self, &rider, &note, actor)
            }
            BooksOp::CarrierHandover { carrier, note } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to hand over to carrier".into());
                }
                courier_ops_view::handover_booked_to_carrier(self, &carrier, &note, actor)
            }
            BooksOp::CollectCod { awb } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to collect COD".into());
                }
                courier_ops_view::collect_cod_for_awb(self, &awb)
            }
            BooksOp::RemitCod { party } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to remit COD".into());
                }
                courier_ops_view::remit_collected_for_party(self, &party)
            }
            BooksOp::AddRider { name, phone, vehicle } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to add riders".into());
                }
                courier_ops_view::add_rider_line(self, &name, &phone, &vehicle)
            }
            BooksOp::PostMoney {
                kind,
                amount_paise,
                note,
            } => {
                let kind = MoneyMoveKind::parse(&kind)
                    .ok_or_else(|| "Unknown bank movement".to_string())?;
                self.post_money_movement(kind, amount_paise, &note, actor)?;
                Ok(format!("{} posted", kind.label()))
            }
            BooksOp::SetMovementCleared { number, cleared } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to reconcile bank movements".into());
                }
                let uncleared: Vec<String> = self
                    .list_money_movements(50)?
                    .into_iter()
                    .filter(|m| !m.cleared)
                    .map(|m| m.id)
                    .collect();
                let id = uncleared
                    .get(number.wrapping_sub(1))
                    .cloned()
                    .ok_or_else(|| "No such uncleared movement".to_string())?;
                self.set_money_movement_cleared(&id, cleared, actor)?;
                Ok(if cleared {
                    "Marked cleared — seen on the statement".into()
                } else {
                    "Unmarked".into()
                })
            }
            BooksOp::RenameBusiness { name } => {
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can rename the business".into());
                }
                self.set_org_name(&name)?;
                Ok("Business name saved".into())
            }
            BooksOp::SetVenueBrand {
                title,
                icon_filename,
                icon_bytes,
            } => {
                if !actor.is_owner() {
                    return Err("Only Owner can change in-app brand".into());
                }
                self.set_venue_title(&title)?;
                if !icon_bytes.is_empty() {
                    let file = write_upload(&icon_filename, &icon_bytes)?;
                    let _ = self.import_venue_icon(file.path())?;
                }
                Ok("Brand saved".into())
            }
            BooksOp::ClearVenueIcon => {
                if !actor.is_owner() {
                    return Err("Only Owner can change in-app brand".into());
                }
                self.set_venue_icon_key(None)?;
                Ok("Icon cleared".into())
            }
            BooksOp::AddAttachmentNote {
                entity,
                entity_id,
                label,
                body,
            } => {
                self.add_attachment_note(entity, &entity_id, &label, &body, actor)?;
                Ok("Note added".into())
            }
            BooksOp::AddAttachmentImage {
                entity,
                entity_id,
                label,
                filename,
                bytes,
            } => {
                let file = write_upload(&filename, &bytes)?;
                self.add_attachment_image(entity, &entity_id, file.path(), &label, actor)?;
                Ok("Photo attached (stored as WebP)".into())
            }
            BooksOp::AddAttachmentFile {
                entity,
                entity_id,
                label,
                filename,
                bytes,
            } => {
                let file = write_upload(&filename, &bytes)?;
                self.add_attachment_file(entity, &entity_id, file.path(), &label, actor)?;
                Ok("File attached".into())
            }
            BooksOp::SetBookingMeasures { settings } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change booking measures".into());
                }
                self.set_booking_measure_settings(&settings)?;
                Ok("Booking measures saved".into())
            }
            BooksOp::ImportBranchSnapshot { json } => {
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can import a branch day".into());
                }
                let branch = self.import_branch_snapshot(&json)?;
                Ok(format!("Imported day for {}", branch.label()))
            }
            BooksOp::AddUnit { code, name } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to add units".into());
                }
                self.add_unit(&code, &name)?;
                Ok(format!("Unit {code} added"))
            }
            BooksOp::AddUnitConversion {
                from_code,
                to_code,
                factor_milli,
            } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage units".into());
                }
                self.add_unit_conversion(&from_code, &to_code, factor_milli, actor)?;
                Ok(match self.resolve_unit(&from_code) {
                    Some((family, factor)) => {
                        let anchor = domain::unit_family_anchor(&family).unwrap_or("");
                        format!(
                            "Added — now 1 {from_code} = {} {anchor} ({family})",
                            precise::format_kg(factor)
                        )
                    }
                    None => format!(
                        "1 {from_code} → {to_code} added — '{from_code}' is not linked to a standard unit yet"
                    ),
                })
            }
            BooksOp::AddAwbPool {
                carrier,
                prefix,
                start_n,
                end_n,
                label,
            } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to add AWB pools".into());
                }
                let party = self
                    .list_customers()?
                    .into_iter()
                    .find(|c| c.name.eq_ignore_ascii_case(carrier.trim()) || c.id == carrier.trim())
                    .ok_or_else(|| "Carrier party not found — add them on Parties first".to_string())?;
                let pool = self.add_awb_pool(&party.id, &prefix, start_n, end_n, &label)?;
                Ok(format!(
                    "AWB pool {}{}–{} for {}",
                    pool.prefix, pool.start_n, pool.end_n, party.name
                ))
            }
            BooksOp::LogExpense {
                amount_paise,
                note,
                category,
                parts,
            } => {
                let id = self.add_expense_tenders(amount_paise, &category, &note, actor, &parts)?;
                Ok(format!("Expense logged{EXPENSE_ID_MARK}{id}"))
            }
            BooksOp::PrintTestPage { route_key } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to print a test page".into());
                }
                self.print_test_page_for_route(&route_key)
            }
            BooksOp::SaveBackupCopy => {
                if !actor.is_owner() && !actor.can_access_backup() {
                    return Err("Not allowed to save a backup copy".into());
                }
                let stamp = chrono::Local::now().format("%Y%m%d-%H%M");
                let file = self.auto_backup_dir().join(format!("copy-{stamp}.sqlite3"));
                self.backup_to(&file)?;
                self.note_backup_copy_saved()?;
                Ok(format!("Backup saved on the shop device ({})", file.display()))
            }
        }
    }
}
