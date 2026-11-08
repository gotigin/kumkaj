//! Person-level book writes. A Client sends one of these to the shop over
//! `POST /books`; the shop applies it. Device exceptions (backup folder,
//! extra tills, pairing code, forget device, restore) are not in this enum.

use crate::domain::{
    self, AttachmentEntity, ModifierGroup, ModifierOption, Money, MoneyMoveKind, PayMethod,
    Product, ScheduleWindow, StaffFeatures, StaffMember, StaffRole,
};
use crate::fiscal::TaxPricing;
use crate::print_share::KotSlipFormat;
use crate::schemes::Scheme;
use crate::store::{CrmSegment, StaffHire, Store};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Splits a shift-close reply into the status line and the handover slip.
pub const SHIFT_SLIP_MARK: &str = "\n---slip---\n";

/// Splits an expense reply into the status line and the new expense id.
pub const EXPENSE_ID_MARK: &str = "\n---expense---\n";

/// Splits a hire reply into the status line and the new staff id.
pub const HIRED_STAFF_MARK: &str = "\n---hired---\n";

pub fn shift_slip_parts(message: &str) -> (&str, Option<&str>) {
    match message.split_once(SHIFT_SLIP_MARK) {
        Some((status, slip)) => (status, Some(slip)),
        None => (message, None),
    }
}

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
    SetOpeningFloat {
        amount_paise: i64,
    },
    CloseShift {
        counted_paise: i64,
    },
    CommitDayClose {
        counted_cash_paise: Option<i64>,
    },
    SetPlatformLanguage,
    SetUserLanguage {
        code: String,
    },
    SetCurrency {
        code: String,
    },
    UpdateParty {
        id: String,
        name: String,
        phone: String,
        birthday: String,
        kind: String,
    },
    AddParty {
        name: String,
        phone: String,
        birthday: String,
        kind: String,
    },
    DeleteParty {
        id: String,
    },
    SetCreditLimit {
        customer_id: String,
        name: String,
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
        against_bill_id: Option<String>,
    },
    WriteOffPayable {
        party_id: String,
        amount_paise: i64,
        reason: String,
    },
    CrmCampaign {
        kind: String,
        inactive_days: i64,
        top_n: usize,
        body: String,
        channel: String,
    },
    WalletTopUp {
        customer_id: String,
        amount_paise: i64,
    },
    IssueVoucher {
        code: String,
        amount_paise: i64,
    },
    RecordPurchase {
        supplier: String,
        party_id: String,
        party_name: String,
        party_kind: String,
        on_account: bool,
        product_id: String,
        new_item_name: String,
        new_item_unit: String,
        qty_milli: i64,
        unit_cost_paise: i64,
        note: String,
        batch_code: String,
        expires_on: String,
    },
    SaveMenuItem {
        updating: bool,
        product_id: String,
        name: String,
        price_paise: i64,
        unit_label: String,
        resale: bool,
        aliases: Vec<String>,
        tax_text: String,
        category_id: String,
        category_name: String,
        category_photo_bytes: Vec<u8>,
        category_photo_filename: String,
        station_name: String,
        modifier_groups: Vec<ModifierGroup>,
        variations: Vec<ModifierOption>,
        area_prices: Vec<ModifierOption>,
        schedule: Vec<ScheduleWindow>,
        group_name: String,
        recipe_text: String,
        image_bytes: Vec<u8>,
        image_filename: String,
        existing_image_key: Option<String>,
        barcode: String,
        composition: String,
        schedule_h: bool,
        rx_required: bool,
        rx_note: String,
        hsn_code: String,
    },
    ArchiveProduct {
        id: String,
        name: String,
    },
    RestoreProduct {
        id: String,
        name: String,
    },
    SetProductAvailable {
        id: String,
        name: String,
        available: bool,
    },
    ImportShelfTemplate,
    ImportClientTemplate {
        template_id: String,
    },
    ImportCsv {
        parties: bool,
        filename: String,
        bytes: Vec<u8>,
    },
    ImportAggregator,
    ImportCategoryImage {
        category_id: String,
        filename: String,
        bytes: Vec<u8>,
    },
    BulkPrice {
        category_id: String,
        percent: Option<i64>,
        set_paise: Option<i64>,
    },
    AdjustStock {
        product_id: String,
        new_item_name: String,
        new_item_unit: String,
        delta_qty: i64,
        note: String,
    },
    SetReorder {
        product_id: String,
        level: i64,
        name: String,
        unit_label: String,
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
    RenameStaff {
        staff_id: String,
        name: String,
    },
    ApplyStaffConfig {
        staff_id: String,
        role: StaffRole,
        role_title: String,
        features: StaffFeatures,
        from_name: String,
        to_name: String,
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
    SetZeroSales {
        allow: bool,
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
    SetBillFormat {
        header: String,
        fssai: String,
        footer: String,
        /// `None` = leave Server UPI unchanged (footer-only / Sales edits).
        #[serde(default)]
        upi_vpa: Option<String>,
        #[serde(default)]
        upi_payee: Option<String>,
        /// Field show / size / weight / align. Empty leaves the saved styles.
        #[serde(default)]
        styles: String,
    },
    ImportPrintStamp {
        filename: String,
        bytes: Vec<u8>,
    },
    ImportPaymentQr {
        filename: String,
        bytes: Vec<u8>,
    },
    ClearPrintStamp,
    ClearPaymentQr,
    SetKotFormat {
        format: KotSlipFormat,
    },
    SetKotStationPrefs {
        priority_csv: String,
        selected_csv: String,
        default_id: String,
    },
    UpsertScheme {
        scheme: Scheme,
    },
    EnsureFloorArea {
        name: String,
    },
    AddTablesRange {
        prefix: String,
        from: u32,
        to: u32,
        area: String,
    },
    AddTable {
        name: String,
        area: String,
    },
    RemoveTables {
        names: Vec<String>,
    },
    AddStation {
        name: String,
    },
    RemoveStation {
        id: String,
    },
    AddReservation {
        name: String,
        phone: String,
        guests: i64,
        at: String,
        table_name: String,
    },
    SetReservationStatus {
        id: String,
        status: String,
        guest_name: String,
        table_name: String,
    },
    AddWaitlist {
        name: String,
        party_size: i64,
        quoted_mins: i64,
    },
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
    PostMoney {
        kind: String,
        amount_paise: i64,
        note: String,
    },
    SetMovementCleared {
        id: String,
        cleared: bool,
    },
    AddBranch {
        name: String,
        code: String,
    },
    ImportBranchSnapshot {
        json: String,
    },
    SwitchBranch {
        id: String,
    },
    SetUnitPicker {
        code: String,
        enabled: bool,
    },
    SetAreaPicker {
        name: String,
        enabled: bool,
    },
    SetStationPicker {
        id: String,
        enabled: bool,
    },
    SetTaxPicker {
        id: String,
        enabled: bool,
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
    SetFrequent {
        days: u32,
        limit: u32,
    },
    SetVenueBrand {
        title: String,
        /// Empty leaves the business name alone.
        org: String,
        /// Empty bytes leave the icon alone.
        icon_filename: String,
        icon_bytes: Vec<u8>,
    },
    ClearVenueIcon,
    SetKotAutoPrint {
        on: bool,
    },
    /// Bill print size (Standard Portrait/Landscape em) — shop printers.json on Server.
    SetStandardEm {
        em: f32,
    },
    /// Content width in millimetres for the active slip (bill or KOT).
    SetContentWidth {
        mm: u32,
        /// `true` = kitchen ticket; `false` = bill.
        kot: bool,
    },
    /// Print a test page on the Server's printers for `route_key` (`bill` / `kot:`…).
    PrintTestPage {
        route_key: String,
    },
    /// Keep / on day close / after N hours — trading rows only.
    SetHistoryRetention {
        mode: String,
        hours: u32,
    },
    /// Wipe sales / KOTs / ledgers; keep catalogue, staff, printers, settings.
    ClearTradingHistory,
    LogExpense {
        amount_paise: i64,
        note: String,
        category: String,
        parts: Vec<(PayMethod, i64)>,
    },
    SaveBackupCopy,
}

impl BooksOp {
    pub fn label(&self) -> String {
        match self {
            Self::SetOpeningFloat { .. } => "Set the opening cash",
            Self::CloseShift { .. } => "Close the shift",
            Self::CommitDayClose { .. } => "Record the day close",
            Self::SetPlatformLanguage | Self::SetUserLanguage { .. } | Self::SetCurrency { .. } => {
                "Change language"
            }
            Self::UpdateParty { .. } => "Edit party",
            Self::AddParty { .. } => "Add customers",
            Self::DeleteParty { .. } => "Delete party",
            Self::SetCreditLimit { .. } => "Set credit limit",
            Self::WriteOffCredit { .. } => "Write off credit",
            Self::SettlePayable { .. } => "Settle payable",
            Self::WriteOffPayable { .. } => "Write off payable",
            Self::CrmCampaign { .. } => "CRM campaign",
            Self::WalletTopUp { .. } => "Wallet top-up",
            Self::IssueVoucher { .. } => "Issue voucher",
            Self::RecordPurchase { .. } => "Record purchases",
            Self::SaveMenuItem { updating: true, .. } => "Change items",
            Self::SaveMenuItem { .. } => "Add items",
            Self::ArchiveProduct { .. } => "Remove items",
            Self::RestoreProduct { .. } => "Restore items",
            Self::SetProductAvailable { .. } => "Mark items sold out",
            Self::ImportShelfTemplate => "Import photo shelf",
            Self::ImportClientTemplate { .. } => "Import a menu template",
            Self::ImportCsv { .. } => "Import from CSV",
            Self::ImportAggregator => "Import aggregator orders",
            Self::ImportCategoryImage { .. } => "Change items",
            Self::BulkPrice { .. } => "Bulk price",
            Self::AdjustStock { .. } => "Adjust stock",
            Self::SetReorder { .. } => "Change reorder levels",
            Self::HireStaff { .. } => "Add staff",
            Self::SetStaffLogin { .. } => "Change staff logins",
            Self::DeactivateStaff { .. } => "Remove staff",
            Self::RenameStaff { .. } => "Rename staff",
            Self::ApplyStaffConfig { .. } => "Copy staff setup",
            Self::SetStaffFeatures { .. } => "Change staff permissions",
            Self::SetDuressPin { .. } => "Set emergency unlock PIN",
            Self::ClearDuressPin => "Clear emergency unlock PIN",
            Self::ApplyCountry { .. } => "Change the country pack",
            Self::SetTaxPricing { .. } => "Change tax pricing",
            Self::SetZeroSales { .. } => "Change zero-value sales",
            Self::SetFiscal { .. } => "Set the business id",
            Self::SetDateTimeAppearance { .. } => "Change date and time appearance",
            Self::SetBillFormat { .. }
            | Self::ImportPrintStamp { .. }
            | Self::ImportPaymentQr { .. }
            | Self::ClearPrintStamp
            | Self::ClearPaymentQr => "Change the bill format",
            Self::SetKotFormat { .. } | Self::SetKotStationPrefs { .. } => "Change the KOT format",
            Self::UpsertScheme { .. } => "Change schemes",
            Self::EnsureFloorArea { .. } => "Add floor area",
            Self::AddTablesRange { .. } | Self::AddTable { .. } => "Add tables",
            Self::RemoveTables { .. } => "Remove tables",
            Self::AddStation { .. } => "Add kitchen stations",
            Self::RemoveStation { .. } => "Remove kitchen stations",
            Self::AddReservation { .. } => "Book tables",
            Self::SetReservationStatus { .. } => "Change bookings",
            Self::AddWaitlist { .. } => "Waitlist",
            Self::AddAttachmentNote { .. } => "Add note",
            Self::AddAttachmentImage { .. } => "Attach photo",
            Self::AddAttachmentFile { .. } => "Attach file",
            Self::PostMoney { .. } => "Post bank / cashbook",
            Self::SetMovementCleared { .. } => "Reconcile bank movements",
            Self::AddBranch { .. } => "Add branches",
            Self::ImportBranchSnapshot { .. } => "Import a branch day",
            Self::SwitchBranch { .. } => "Switch branch",
            Self::SetUnitPicker { .. } => "Change unit pickers",
            Self::SetAreaPicker { .. } | Self::SetStationPicker { .. } | Self::SetTaxPicker { .. } => {
                "Change pickers"
            }
            Self::AddUnit { .. } => "Add units",
            Self::AddUnitConversion { .. } => "Add unit conversion",
            Self::SetFrequent { .. } => "Frequent items settings",
            Self::SetVenueBrand { .. } | Self::ClearVenueIcon => "Change in-app brand",
            Self::SetKotAutoPrint { .. } => "Change KOT auto-print",
            Self::SetStandardEm { .. } => "Change the bill print size",
            Self::SetContentWidth { .. } => "Change the content width",
            Self::PrintTestPage { .. } => "Print a test page",
            Self::SetHistoryRetention { .. } => "Change history clearing",
            Self::ClearTradingHistory => "Clear trading history",
            Self::LogExpense { .. } => "Log expenses",
            Self::SaveBackupCopy => "Save a backup copy",
        }
        .to_string()
    }

    pub fn done_message(&self) -> String {
        match self {
            Self::SetOpeningFloat { .. } => "Opening cash saved".into(),
            Self::CloseShift { .. } => "Shift closed".into(),
            Self::CommitDayClose { .. } => "Day close recorded".into(),
            Self::SetPlatformLanguage => "Language follows this device".into(),
            Self::SetUserLanguage { code } => format!("Language: {code}"),
            Self::SetCurrency { code } => format!("Currency: {code}"),
            Self::UpdateParty { name, .. } => format!("Party updated: {}", name.trim()),
            Self::AddParty { name, .. } => format!("Added {}", name.trim()),
            Self::DeleteParty { .. } => "Party deleted".into(),
            Self::SetCreditLimit { .. } => "Credit limit saved".into(),
            Self::WriteOffCredit { .. } => "Credit written off".into(),
            Self::SettlePayable { .. } => "Payable settled".into(),
            Self::WriteOffPayable { .. } => "Payable written off".into(),
            Self::CrmCampaign { channel, .. } => format!("Campaign queued ({channel})"),
            Self::WalletTopUp { .. } => "Wallet topped up".into(),
            Self::IssueVoucher { code, .. } => format!("Voucher {} issued", code.trim()),
            Self::RecordPurchase { .. } => "Purchase recorded".into(),
            Self::SaveMenuItem { name, updating, .. } => {
                if *updating {
                    format!("Updated {}", name.trim())
                } else {
                    format!("Added {}", name.trim())
                }
            }
            Self::ArchiveProduct { name, .. } => format!("Removed {name}"),
            Self::RestoreProduct { name, .. } => format!("{name} back on the menu"),
            Self::SetProductAvailable { name, available, .. } => {
                if *available {
                    format!("{name} restocked")
                } else {
                    format!("{name} marked sold out")
                }
            }
            Self::ImportShelfTemplate | Self::ImportClientTemplate { .. } => {
                "Menu template imported".into()
            }
            Self::ImportCsv { parties: true, .. } => "Parties imported".into(),
            Self::ImportCsv { .. } => "Items imported".into(),
            Self::ImportAggregator => "Aggregator orders imported".into(),
            Self::ImportCategoryImage { .. } => "Category photo saved".into(),
            Self::BulkPrice { .. } => "Prices updated".into(),
            Self::AdjustStock { .. } => "Stock adjusted".into(),
            Self::SetReorder { .. } => "Reorder level saved".into(),
            Self::HireStaff { name, .. } => format!("Added {}", name.trim()),
            Self::SetStaffLogin { .. } => "Login saved".into(),
            Self::DeactivateStaff { name, .. } => format!("Removed {name}"),
            Self::RenameStaff { .. } => "Staff renamed".into(),
            Self::ApplyStaffConfig { .. } => "Staff setup copied".into(),
            Self::SetStaffFeatures { .. } => "Staff features saved".into(),
            Self::SetDuressPin { .. } => "Emergency unlock PIN saved".into(),
            Self::ClearDuressPin => "Emergency unlock PIN turned off".into(),
            Self::ApplyCountry { .. } => "Country pack applied".into(),
            Self::SetTaxPricing { .. } => "Tax pricing saved".into(),
            Self::SetZeroSales { allow: true } => "Zero-value sales on".into(),
            Self::SetZeroSales { .. } => "Zero-value sales off".into(),
            Self::SetFiscal { .. } => "Business id saved".into(),
            Self::SetDateTimeAppearance { .. } => "Date and time appearance saved".into(),
            Self::SetBillFormat { .. } => "Bill format saved".into(),
            Self::ImportPrintStamp { .. } => "Stamp saved for bill faces".into(),
            Self::ImportPaymentQr { .. } => "Payment QR saved for bill faces".into(),
            Self::ClearPrintStamp => "Stamp cleared".into(),
            Self::ClearPaymentQr => "Payment QR cleared".into(),
            Self::SetKotFormat { .. } => "KOT format saved".into(),
            Self::SetKotStationPrefs { .. } => "Kitchen order saved".into(),
            Self::UpsertScheme { .. } => "Scheme saved".into(),
            Self::EnsureFloorArea { name } => format!("Area {name} added"),
            Self::AddTablesRange { .. } | Self::AddTable { .. } => "Tables added".into(),
            Self::RemoveTables { .. } => "Tables removed".into(),
            Self::AddStation { .. } => "Kitchen station added".into(),
            Self::RemoveStation { .. } => "Kitchen station removed".into(),
            Self::AddReservation { .. } => "Table booked".into(),
            Self::SetReservationStatus { status, .. } => format!("Booking {status}"),
            Self::AddWaitlist { name, .. } => format!("Waitlist: {}", name.trim()),
            Self::AddAttachmentNote { .. } => "Note added".into(),
            Self::AddAttachmentImage { .. } => "Photo attached (stored as WebP)".into(),
            Self::AddAttachmentFile { .. } => "File attached".into(),
            Self::PostMoney { .. } => "Bank movement posted".into(),
            Self::SetMovementCleared { cleared: true, .. } => "Marked cleared".into(),
            Self::SetMovementCleared { .. } => "Unmarked".into(),
            Self::AddBranch { .. } => "Branch added".into(),
            Self::ImportBranchSnapshot { .. } => "Branch day imported".into(),
            Self::SwitchBranch { .. } => "Branch switched".into(),
            Self::SetUnitPicker { .. } => "Unit picker updated".into(),
            Self::SetAreaPicker { .. } | Self::SetStationPicker { .. } | Self::SetTaxPicker { .. } => {
                "Picker updated".into()
            }
            Self::AddUnit { code, .. } => format!("Unit {code} added"),
            Self::AddUnitConversion { .. } => "Unit conversion added".into(),
            Self::SetFrequent { .. } => "Frequent items saved".into(),
            Self::SetVenueBrand { org, .. } => {
                if org.trim().is_empty() {
                    "Brand saved".into()
                } else {
                    "Brand and business name saved".into()
                }
            }
            Self::ClearVenueIcon => "Icon cleared".into(),
            Self::SetKotAutoPrint { on: true } => "KOT auto-print on".into(),
            Self::SetKotAutoPrint { on: false } => "KOT auto-print off".into(),
            Self::SetStandardEm { .. } => "Bill print size saved".into(),
            Self::SetContentWidth { mm, kot: true } => format!("KOT content width {mm} mm"),
            Self::SetContentWidth { mm, kot: false } => format!("Bill content width {mm} mm"),
            Self::PrintTestPage { .. } => "Test page sent".into(),
            Self::SetHistoryRetention { .. } => "History clearing saved".into(),
            Self::ClearTradingHistory => "Trading history cleared — setup kept".into(),
            Self::LogExpense { .. } => "Expense logged".into(),
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
    let path = std::env::temp_dir().join(format!("tabiyat-upload-{}.{}", domain::new_id(), ext));
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

fn crm_segment(kind: &str, inactive_days: i64, top_n: usize) -> Result<CrmSegment, String> {
    Ok(match kind {
        "birthday_week" => CrmSegment::BirthdayThisWeek,
        "birthday_month" => CrmSegment::BirthdayThisMonth,
        "phone" => CrmSegment::WithPhone,
        "loyalty" => CrmSegment::WithLoyalty,
        "inactive" => CrmSegment::InactiveDays(if inactive_days > 0 { inactive_days } else { 30 }),
        "top" => CrmSegment::TopSpenders(if top_n == 0 { 20 } else { top_n }),
        other => return Err(format!("Unknown guest list '{other}'")),
    })
}

impl Store {
    fn show_money(&self, paise: i64) -> String {
        if let (Ok(loc), Ok(fiscal)) = (self.locale_settings(), self.fiscal_settings()) {
            Money(paise).format_localized(&loc.currency, &fiscal)
        } else {
            self.money_display(paise)
        }
    }

    fn resolve_category_id(&self, id: &str, name: &str) -> Result<String, String> {
        let name = name.trim();
        let cats = self.list_categories()?;
        if !id.is_empty() && cats.iter().any(|c| c.id == id) {
            return Ok(id.to_string());
        }
        if name.is_empty() {
            return Ok(String::new());
        }
        if let Some(c) = cats.iter().find(|c| c.name.eq_ignore_ascii_case(name)) {
            return Ok(c.id.clone());
        }
        Ok(self.add_category(name)?.id)
    }

    fn resolve_station_id(&self, name: &str) -> Result<String, String> {
        let name = name.trim();
        if name.is_empty() {
            return Ok(String::new());
        }
        let stations = self.list_stations()?;
        if let Some(s) = stations.iter().find(|s| s.name.eq_ignore_ascii_case(name)) {
            return Ok(s.id.clone());
        }
        Ok(self.add_station(name)?.id)
    }

    /// Apply one person-level write. Callers keep [`Self::assert_server_writer`]
    /// on the store methods this dispatches to.
    pub fn apply_books_op(&self, op: BooksOp, actor: &StaffMember) -> Result<String, String> {
        match op {
            BooksOp::SetOpeningFloat { amount_paise } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to set opening cash".into());
                }
                self.set_opening_float(amount_paise, actor)?;
                Ok(format!("Opening cash {} saved", self.show_money(amount_paise)))
            }
            BooksOp::CloseShift { counted_paise } => {
                let (shift, slip) = self.close_shift(actor, counted_paise)?;
                let v = shift.variance_paise();
                let status = if v == 0 {
                    "Shift closed — drawer matches".to_string()
                } else if v < 0 {
                    format!("Shift closed — short {}", self.show_money(-v))
                } else {
                    format!("Shift closed — over {}", self.show_money(v))
                };
                Ok(format!("{status}{SHIFT_SLIP_MARK}{slip}"))
            }
            BooksOp::CommitDayClose { counted_cash_paise } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to close the day".into());
                }
                let at = self.commit_day_close_with_count(actor, counted_cash_paise)?;
                let shown = at[..at.len().min(19)].replace('T', " ");
                Ok(format!("Day close recorded at {shown}"))
            }
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
            BooksOp::UpdateParty { id, name, phone, birthday, kind } => {
                if !actor.is_owner() && !actor.can_access_credit() {
                    return Err("Not allowed to edit parties".into());
                }
                self.update_customer_details(&id, &name, &phone, Some(&birthday))?;
                self.set_customer_kind_label(&id, kind.trim())?;
                Ok(format!("Party updated: {}", name.trim()))
            }
            BooksOp::AddParty { name, phone, birthday, kind } => {
                if !actor.is_owner() && !actor.can_access_credit() && !actor.can_access_sell() {
                    return Err("Not allowed to add parties".into());
                }
                let customer = self.add_customer_kind(&name, &phone, kind.trim())?;
                if !birthday.trim().is_empty() {
                    self.update_customer_details(
                        &customer.id,
                        &customer.name,
                        &customer.phone,
                        Some(&birthday),
                    )?;
                }
                Ok(format!(
                    "Added {} — now editing it; tap New party to add another",
                    name.trim()
                ))
            }
            BooksOp::DeleteParty { id } => {
                if !actor.is_owner() && !actor.can_access_credit() {
                    return Err("Not allowed to delete parties".into());
                }
                self.delete_customer(&id)
            }
            BooksOp::SetCreditLimit { customer_id, name, limit_paise } => {
                if !actor.is_owner() && !actor.can_access_credit() {
                    return Err("Not allowed to set credit limits".into());
                }
                let customer = if !customer_id.is_empty() {
                    self.get_customer(&customer_id)?
                        .ok_or_else(|| "Party not found".to_string())?
                } else {
                    self.find_or_create_customer(name.trim())?
                };
                self.set_customer_credit_limit(&customer.id, limit_paise)?;
                Ok(format!(
                    "Credit limit for {} set to {}",
                    customer.name,
                    self.show_money(limit_paise)
                ))
            }
            BooksOp::WriteOffCredit { customer_id, amount_paise, reason } => {
                let written = self.write_off_credit(&customer_id, amount_paise, &reason, actor)?;
                Ok(format!("Wrote off {}", self.show_money(written)))
            }
            BooksOp::SettlePayable { party_id, amount_paise, pay, against_bill_id } => {
                if !actor.is_owner()
                    && !actor.can_access_credit()
                    && !actor.can_access_inventory()
                {
                    return Err("Not allowed to settle payables".into());
                }
                self.settle_payable_against(
                    &party_id,
                    amount_paise,
                    pay,
                    actor,
                    "",
                    against_bill_id.as_deref(),
                )?;
                Ok(format!("AP paid {}", self.show_money(amount_paise)))
            }
            BooksOp::WriteOffPayable { party_id, amount_paise, reason } => {
                let written = self.write_off_payable(&party_id, amount_paise, &reason, actor)?;
                Ok(format!("AP wrote off {}", self.show_money(written)))
            }
            BooksOp::CrmCampaign { kind, inactive_days, top_n, body, channel } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to run CRM campaigns".into());
                }
                let seg = crm_segment(&kind, inactive_days, top_n)?;
                let n = self.queue_segment_campaign(seg, &body, &channel, actor)?;
                Ok(format!("Queued {n} {channel} message(s) for {}", seg.label()))
            }
            BooksOp::WalletTopUp { customer_id, amount_paise } => {
                if !actor.is_owner() {
                    return Err("Not allowed to top up wallets".into());
                }
                let bal = self.wallet_top_up(&customer_id, amount_paise, actor)?;
                Ok(format!("Wallet now {}", self.show_money(bal)))
            }
            BooksOp::IssueVoucher { code, amount_paise } => {
                if !actor.is_owner() {
                    return Err("Not allowed to issue vouchers".into());
                }
                let v = self.issue_voucher(&code, amount_paise, actor)?;
                Ok(format!(
                    "Voucher {} issued for {}",
                    v.code,
                    self.show_money(v.amount_paise)
                ))
            }
            BooksOp::RecordPurchase {
                supplier,
                party_id,
                party_name,
                party_kind,
                on_account,
                product_id,
                new_item_name,
                new_item_unit,
                qty_milli,
                unit_cost_paise,
                note,
                batch_code,
                expires_on,
            } => {
                let mut party_id = party_id;
                if on_account && party_id.is_empty() {
                    let kind = if party_kind.trim().is_empty() {
                        "Supplier"
                    } else {
                        party_kind.trim()
                    };
                    party_id = self.add_customer_kind(party_name.trim(), "", kind)?.id;
                }
                let product = if !product_id.is_empty() {
                    self.get_product(&product_id)?
                        .ok_or_else(|| "Item not found".to_string())?
                } else {
                    let created = Product::new_resale_stock(&new_item_name, &new_item_unit);
                    self.upsert_product(&created)?;
                    created
                };
                let purchase = self.record_purchase_ex(
                    supplier.trim(),
                    &party_id,
                    on_account,
                    &[(product.id.clone(), qty_milli, unit_cost_paise)],
                    note.trim(),
                    actor,
                )?;
                if !batch_code.trim().is_empty() || !expires_on.trim().is_empty() {
                    let _ = self.set_purchase_batch_expiry(
                        &purchase.id,
                        batch_code.trim(),
                        expires_on.trim(),
                    );
                }
                let qty_show = domain::format_qty_display(qty_milli, 0, &product.unit_label);
                Ok(format!(
                    "Received {qty_show} {} · {}{}",
                    product.name,
                    self.show_money(purchase.total_paise),
                    if on_account { " · on account" } else { "" }
                ))
            }
            BooksOp::SaveMenuItem {
                updating,
                product_id,
                name,
                price_paise,
                unit_label,
                resale,
                aliases,
                tax_text,
                category_id,
                category_name,
                category_photo_bytes,
                category_photo_filename,
                station_name,
                modifier_groups,
                variations,
                area_prices,
                schedule,
                group_name,
                recipe_text,
                image_bytes,
                image_filename,
                existing_image_key,
                barcode,
                composition,
                schedule_h,
                rx_required,
                rx_note,
                hsn_code,
            } => self.save_menu_item(
                actor,
                updating,
                &product_id,
                &name,
                price_paise,
                &unit_label,
                resale,
                &aliases,
                &tax_text,
                &category_id,
                &category_name,
                &category_photo_bytes,
                &category_photo_filename,
                &station_name,
                &modifier_groups,
                &variations,
                &area_prices,
                &schedule,
                &group_name,
                &recipe_text,
                &image_bytes,
                &image_filename,
                existing_image_key.as_deref(),
                &barcode,
                &composition,
                schedule_h,
                rx_required,
                &rx_note,
                &hsn_code,
            ),
            BooksOp::ArchiveProduct { id, name } => {
                if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to manage items".into());
                }
                self.archive_product(&id)?;
                self.audit(actor, "menu.remove", &name)?;
                Ok(format!("Removed {name}"))
            }
            BooksOp::RestoreProduct { id, name } => {
                if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to manage items".into());
                }
                self.restore_product(&id)?;
                self.audit(actor, "menu.restore", &name)?;
                Ok(format!("{name} back on the menu"))
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
            BooksOp::ImportShelfTemplate => {
                if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to manage items".into());
                }
                let done = self.import_catalogue(crate::menu_images::shelf())?;
                Ok(format!(
                    "Imported {} new dishes in {} categories ({} already on the menu)",
                    done.items_added, done.categories_added, done.items_kept
                ))
            }
            BooksOp::ImportClientTemplate { template_id } => {
                if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to manage items".into());
                }
                let tmpl = crate::menu_templates::find(&template_id)
                    .ok_or_else(|| format!("Unknown menu template '{template_id}'"))?;
                let done = self.import_menu_template(tmpl)?;
                Ok(format!(
                    "Imported {} — {} new dishes in {} categories",
                    tmpl.title, done.items_added, done.categories_added
                ))
            }
            BooksOp::ImportCsv { parties, bytes, .. } => {
                if parties {
                    if !actor.is_owner() && !actor.can_access_credit() {
                        return Err("Not allowed to import parties".into());
                    }
                } else if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to import menu CSV".into());
                }
                let text = String::from_utf8(bytes)
                    .map_err(|_| "CSV must be UTF-8 text".to_string())?;
                let (added, skipped, problems) = if parties {
                    self.import_parties_csv(&text)?
                } else {
                    self.import_items_csv(&text)?
                };
                Ok(csv_summary(added, skipped, &problems))
            }
            BooksOp::ImportAggregator => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to import aggregator orders".into());
                }
                let n = self.import_aggregator_orders_drop_file()?;
                Ok(format!("Imported {n} aggregator order(s) into the inbox"))
            }
            BooksOp::ImportCategoryImage { category_id, filename, bytes } => {
                if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to manage items".into());
                }
                let file = write_upload(&filename, &bytes)?;
                let key = self.import_category_image(&category_id, file.path())?;
                self.set_category_image(&category_id, &key)?;
                Ok("Category photo saved".into())
            }
            BooksOp::BulkPrice { category_id, percent, set_paise } => {
                if !actor.is_owner() && !actor.can_access_items() {
                    return Err("Not allowed to manage items".into());
                }
                let n = self.bulk_price_category(&category_id, percent, set_paise, actor)?;
                Ok(format!("Updated {n} item(s)"))
            }
            BooksOp::AdjustStock { product_id, new_item_name, new_item_unit, delta_qty, note } => {
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                let product = if !product_id.is_empty() {
                    self.get_product(&product_id)?
                        .ok_or_else(|| "Item not found".to_string())?
                } else if delta_qty > 0 {
                    let created = Product::new_resale_stock(&new_item_name, &new_item_unit);
                    self.upsert_product(&created)?;
                    created
                } else {
                    return Err(format!("Nothing called \"{}\" in stock", new_item_name.trim()));
                };
                if !product.tracks_stock() {
                    self.set_product_stock_qty(&product.id, 0)?;
                }
                let reason = if note.trim().is_empty() { "adjust" } else { note.trim() };
                let on_hand = self.adjust_stock(&product.id, delta_qty, reason, note.trim(), actor)?;
                Ok(format!("{} · {on_hand} on hand", product.name))
            }
            BooksOp::SetReorder { product_id, level, name, unit_label } => {
                if !actor.can_access_inventory() && !actor.is_owner() {
                    return Err("Not allowed to change stock".into());
                }
                self.set_reorder_level(&product_id, level, actor)?;
                let unit = unit_label.trim();
                Ok(format!("{name} · buy more at {level} {unit}").trim_end().to_string())
            }
            BooksOp::HireStaff { name, role, role_title, pin, login_id, password, features } => {
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
            BooksOp::SetStaffLogin { staff_id, login_id, password } => {
                self.set_staff_login(&staff_id, &login_id, &password, actor)?;
                Ok("Login saved".into())
            }
            BooksOp::DeactivateStaff { staff_id, name } => {
                self.deactivate_staff(&staff_id, actor)?;
                Ok(format!("Removed {name}"))
            }
            BooksOp::RenameStaff { staff_id, name } => {
                self.rename_staff(&staff_id, &name, actor)?;
                Ok("Staff renamed".into())
            }
            BooksOp::ApplyStaffConfig { staff_id, role, role_title, features, from_name, to_name } => {
                self.apply_staff_config(&staff_id, role, &role_title, &features, actor)?;
                Ok(format!("Copied setup from {from_name} onto {to_name}"))
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
                Ok(format!("{name} pack applied — check taxes on Menu items"))
            }
            BooksOp::SetTaxPricing { pricing } => {
                if !actor.is_owner() {
                    return Err("Only the Owner can change tax pricing".into());
                }
                self.set_tax_pricing(pricing)?;
                let tax = self.fiscal_settings()?.tax_label().to_string();
                Ok(pricing.label_for(&tax))
            }
            BooksOp::SetZeroSales { allow } => {
                self.set_allow_zero_sales(allow, actor)?;
                Ok(if allow {
                    "Zero-value sales on".into()
                } else {
                    "Zero-value sales off".into()
                })
            }
            BooksOp::SetFiscal { business_tax_id, service_bps } => {
                if !actor.is_owner() {
                    return Err("Only the Owner can set the business id".into());
                }
                self.set_business_tax_id(&business_tax_id)?;
                self.set_service_charge_bps(service_bps)?;
                Ok("Business id saved".into())
            }
            BooksOp::SetDateTimeAppearance {
                date_order,
                time_format,
            } => {
                if !actor.is_owner() && !actor.can_access_settings() && !actor.role.is_admin() {
                    return Err("Not allowed to change date and time appearance".into());
                }
                let settings = self.set_datetime_appearance(&date_order, &time_format)?;
                Ok(format!(
                    "Date and time appearance saved — example {ex}",
                    ex = settings.format_now()
                ))
            }
            BooksOp::SetBillFormat { header, fssai, footer, upi_vpa, upi_payee, styles } => {
                // Header / FSSAI / footer: Settings. UPI VPA / payee: Owner-only.
                // `None` UPI fields leave Server values alone (Client footer-only).
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change bill format".into());
                }
                let current_vpa = self.upi_vpa()?;
                let current_payee = self.upi_payee()?;
                let upi_changed = upi_vpa
                    .as_ref()
                    .is_some_and(|v| v.trim() != current_vpa.trim())
                    || upi_payee
                        .as_ref()
                        .is_some_and(|p| p.trim() != current_payee.trim());
                if upi_changed && !actor.is_owner() {
                    return Err("Only the Owner can change UPI details".into());
                }
                self.set_bill_format_ex(
                    &header,
                    &fssai,
                    &footer,
                    upi_vpa.as_deref(),
                    upi_payee.as_deref(),
                )?;
                if !styles.trim().is_empty() {
                    let parsed = crate::print_share::BillSlipFormat::parse(styles.trim());
                    // Full draft styles (layout, piece toggles, width, Cashier…).
                    // Address / UPI columns were just written — keep those.
                    let columns = self.bill_slip_format().unwrap_or(parsed.clone());
                    let mut kept = parsed;
                    kept.header = columns.header;
                    kept.fssai = columns.fssai;
                    kept.footer = columns.footer;
                    kept.upi_vpa = columns.upi_vpa;
                    kept.upi_payee = columns.upi_payee;
                    self.set_bill_slip_format(&kept)?;
                }
                Ok("Bill format saved".into())
            }
            BooksOp::ImportPrintStamp { filename, bytes } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change bill format".into());
                }
                let file = write_upload(&filename, &bytes)?;
                let _ = self.import_print_stamp(file.path())?;
                Ok("Stamp saved for bill faces".into())
            }
            BooksOp::ImportPaymentQr { filename, bytes } => {
                if !actor.is_owner() {
                    return Err("Only the Owner can change the payment QR".into());
                }
                let file = write_upload(&filename, &bytes)?;
                let _ = self.import_payment_qr(file.path())?;
                Ok("Payment QR saved for bill faces".into())
            }
            BooksOp::ClearPrintStamp => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change bill format".into());
                }
                self.set_print_stamp_key(None)?;
                Ok("Stamp cleared".into())
            }
            BooksOp::ClearPaymentQr => {
                if !actor.is_owner() {
                    return Err("Only the Owner can change the payment QR".into());
                }
                self.set_payment_qr_key(None)?;
                Ok("Payment QR cleared".into())
            }
            BooksOp::SetKotFormat { format } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change KOT format".into());
                }
                let _ = self.set_kot_slip_format(&format)?;
                Ok("KOT format saved".into())
            }
            BooksOp::SetKotStationPrefs { priority_csv, selected_csv, default_id } => {
                self.set_kot_station_priority_csv(&priority_csv)?;
                self.set_kot_stations_selected_csv(&selected_csv)?;
                self.set_kot_station_default(&default_id)?;
                Ok("Kitchen order saved".into())
            }
            BooksOp::UpsertScheme { scheme } => {
                if !actor.is_owner() && !actor.can_access_settings() && !actor.can_access_sales() {
                    return Err("Not allowed to manage schemes".into());
                }
                let name = scheme.name.clone();
                self.upsert_scheme(&scheme)?;
                Ok(format!("Added scheme: {name}"))
            }
            BooksOp::EnsureFloorArea { name } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage tables".into());
                }
                self.ensure_floor_area(&name)?;
                Ok(format!("Area {name} added"))
            }
            BooksOp::AddTablesRange { prefix, from, to, area } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage tables".into());
                }
                let added = self.add_tables_range(&prefix, from, to, &area)?;
                Ok(format!("{added} table(s) added ({prefix}{from}–{prefix}{to})"))
            }
            BooksOp::AddTable { name, area } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage tables".into());
                }
                let table = self.add_table_in_area(&name, &area)?;
                Ok(format!("Table {} added", table.name))
            }
            BooksOp::RemoveTables { names } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage tables".into());
                }
                let (removed, busy) = self.remove_tables_named(&names)?;
                let mut msg = format!("{removed} table(s) removed");
                if !busy.is_empty() {
                    msg.push_str(&format!(" · kept, open order: {}", busy.join(", ")));
                }
                Ok(msg)
            }
            BooksOp::AddStation { name } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage kitchen stations".into());
                }
                let station = self.add_station(&name)?;
                Ok(format!(
                    "Station {} added — attach its printers: Kitchen → Kitchen stations → Printers for this station",
                    station.name
                ))
            }
            BooksOp::RemoveStation { id } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage kitchen stations".into());
                }
                let name = self.remove_station(&id)?;
                Ok(format!("Station {name} removed"))
            }
            BooksOp::AddReservation { name, phone, guests, at, table_name } => {
                let r = self.add_reservation(&name, &phone, guests, &at, &table_name, actor)?;
                Ok(format!("Booked {} · {}", r.name, r.at))
            }
            BooksOp::SetReservationStatus { id, status, guest_name, table_name } => {
                self.set_reservation_status(&id, &status, actor)?;
                Ok(if status == "seated" {
                    if table_name.is_empty() {
                        format!("{guest_name} seated — pick their table")
                    } else {
                        format!("{guest_name} seated at {table_name} — tap {table_name} to order")
                    }
                } else {
                    format!("Booking for {guest_name} cancelled")
                })
            }
            BooksOp::AddWaitlist {
                name,
                party_size,
                quoted_mins,
            } => {
                if !actor.is_owner() && !actor.can_access_floor() {
                    return Err("Not allowed to manage the waitlist".into());
                }
                let entry = self.add_waitlist(&name, party_size, quoted_mins, actor)?;
                Ok(format!("Waitlist: {}", entry.name))
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
            BooksOp::PostMoney { kind, amount_paise, note } => {
                let kind = MoneyMoveKind::parse(&kind).ok_or_else(|| "Unknown bank movement".to_string())?;
                self.post_money_movement(kind, amount_paise, &note, actor)?;
                Ok(format!("{} posted", kind.label()))
            }
            BooksOp::SetMovementCleared { id, cleared } => {
                if !actor.is_owner() && !actor.can_access_sales() {
                    return Err("Not allowed to reconcile bank movements".into());
                }
                self.set_money_movement_cleared(&id, cleared, actor)?;
                Ok(if cleared {
                    "Marked cleared — seen on the statement".into()
                } else {
                    "Unmarked".into()
                })
            }
            BooksOp::AddBranch { name, code } => {
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can add branches".into());
                }
                let branch = self.add_branch(&name, &code)?;
                Ok(format!("Added {}", branch.label()))
            }
            BooksOp::ImportBranchSnapshot { json } => {
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can import a branch day".into());
                }
                let branch = self.import_branch_snapshot(&json)?;
                Ok(format!("Imported day for {}", branch.label()))
            }
            BooksOp::SwitchBranch { id } => {
                if !actor.can_access_branches() && !actor.is_owner() {
                    return Err("Only Owner can switch branch".into());
                }
                self.set_active_branch_id(&id)?;
                let branch = self
                    .list_branches()?
                    .into_iter()
                    .find(|b| b.id == id)
                    .ok_or_else(|| "Branch not found".to_string())?;
                Ok(format!("Now selling for {}", branch.label()))
            }
            BooksOp::SetUnitPicker { code, enabled } => {
                self.set_unit_picker_enabled(&code, enabled, actor)?;
                Ok(if enabled {
                    format!("{code} shown in pickers")
                } else {
                    format!("{code} hidden from pickers")
                })
            }
            BooksOp::SetAreaPicker { name, enabled } => {
                self.set_area_picker_enabled(&name, enabled, actor)?;
                Ok(format!("Area {name} picker {}", if enabled { "on" } else { "off" }))
            }
            BooksOp::SetStationPicker { id, enabled } => {
                self.set_station_picker_enabled(&id, enabled, actor)?;
                Ok(format!("Station picker {}", if enabled { "on" } else { "off" }))
            }
            BooksOp::SetTaxPicker { id, enabled } => {
                self.set_tax_rate_picker_enabled(&id, enabled, actor)?;
                Ok(format!("Tax picker {}", if enabled { "on" } else { "off" }))
            }
            BooksOp::AddUnit { code, name } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage units".into());
                }
                self.add_unit(&code, &name)?;
                Ok(format!("Unit {code} added"))
            }
            BooksOp::AddUnitConversion { from_code, to_code, factor_milli } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to manage units".into());
                }
                self.add_unit_conversion(&from_code, &to_code, factor_milli)?;
                Ok(match self.resolve_unit(&from_code) {
                    Some((family, factor)) => {
                        let anchor = domain::unit_family_anchor(&family).unwrap_or("");
                        format!(
                            "Added — now 1 {from_code} = {} {anchor} ({family})",
                            crate::precise::format_kg(factor)
                        )
                    }
                    None => format!(
                        "1 {from_code} → {to_code} added — '{from_code}' is not linked to a standard unit yet"
                    ),
                })
            }
            BooksOp::SetFrequent { days, limit } => {
                self.set_frequent_config(days, limit, actor)?;
                let count = self.frequent_product_ids(days, limit)?.len();
                Ok(format!("Frequent items: {days} days, {limit} shown, {count} on the list"))
            }
            BooksOp::SetVenueBrand {
                title,
                org,
                icon_filename,
                icon_bytes,
            } => {
                if !actor.is_owner() {
                    return Err("Only Owner can change in-app brand".into());
                }
                self.set_venue_title(&title)?;
                let org = org.trim();
                if !org.is_empty() {
                    self.set_org_name(org)?;
                }
                if !icon_bytes.is_empty() {
                    let file = write_upload(&icon_filename, &icon_bytes)?;
                    let _ = self.import_venue_icon(file.path())?;
                }
                Ok(if org.is_empty() {
                    "Brand saved".into()
                } else {
                    "Brand and business name saved".into()
                })
            }
            BooksOp::ClearVenueIcon => {
                if !actor.is_owner() {
                    return Err("Only Owner can change in-app brand".into());
                }
                self.set_venue_icon_key(None)?;
                Ok("Icon cleared".into())
            }
            BooksOp::SetKotAutoPrint { on } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change KOT auto-print".into());
                }
                self.set_kot_auto_print(on)?;
                Ok(if on {
                    "KOT auto-print on".into()
                } else {
                    "KOT auto-print off".into()
                })
            }
            BooksOp::SetStandardEm { em } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change bill print size".into());
                }
                let _ = self.set_standard_print_em(em)?;
                Ok("Bill print size saved".into())
            }
            BooksOp::SetContentWidth { mm, kot } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to change the content width".into());
                }
                if kot {
                    let saved = self.set_kot_content_width_mm(mm)?;
                    Ok(format!("KOT content width {saved} mm"))
                } else {
                    let saved = self.set_bill_content_width_mm(mm)?;
                    Ok(format!("Bill content width {saved} mm"))
                }
            }
            BooksOp::PrintTestPage { route_key } => {
                if !actor.is_owner() && !actor.can_access_settings() {
                    return Err("Not allowed to print a test page".into());
                }
                self.print_test_page_for_route(&route_key)
            }
            BooksOp::SetHistoryRetention { mode, hours } => {
                if !actor.is_owner() && !actor.role.is_admin() && !actor.can_access_backup() {
                    return Err("Not allowed to change history clearing".into());
                }
                let retention = crate::business_day::HistoryRetention::parse(&mode, hours);
                self.set_history_retention(retention)?;
                Ok("History clearing saved".into())
            }
            BooksOp::ClearTradingHistory => {
                if !actor.is_owner() && !actor.role.is_admin() {
                    return Err("Only Owner or Admin can clear history".into());
                }
                self.purge_trading_history_keep_setup()?;
                self.audit(actor, "history.clear", "Trading history cleared; setup kept")?;
                Ok("Trading history cleared — setup kept".into())
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

    fn save_menu_item(
        &self,
        actor: &StaffMember,
        updating: bool,
        product_id: &str,
        name: &str,
        price_paise: i64,
        unit_label: &str,
        resale: bool,
        aliases: &[String],
        tax_text: &str,
        category_id: &str,
        category_name: &str,
        category_photo_bytes: &[u8],
        category_photo_filename: &str,
        station_name: &str,
        modifier_groups: &[ModifierGroup],
        variations: &[ModifierOption],
        area_prices: &[ModifierOption],
        schedule: &[ScheduleWindow],
        group_name: &str,
        recipe_text: &str,
        image_bytes: &[u8],
        image_filename: &str,
        existing_image_key: Option<&str>,
        barcode: &str,
        composition: &str,
        schedule_h: bool,
        rx_required: bool,
        rx_note: &str,
        hsn_code: &str,
    ) -> Result<String, String> {
        if !actor.is_owner() && !actor.can_access_items() {
            return Err("Not allowed to manage items".into());
        }
        let name = name.trim();
        if name.is_empty() {
            return Err("Name required".into());
        }
        let sizes_only = !variations.is_empty() && !updating;
        let product_id = if updating {
            product_id.to_string()
        } else {
            domain::new_id()
        };
        let was = if updating {
            self.get_product(&product_id)?.map(|p| p.price)
        } else {
            None
        };
        let photo = if image_bytes.is_empty() {
            None
        } else {
            Some(write_upload(image_filename, image_bytes)?)
        };
        let mut image_key = existing_image_key
            .filter(|k| !k.starts_with("pending_custom:"))
            .map(|k| k.to_string());
        if let Some(src) = photo.as_ref()
            && !sizes_only
        {
            image_key = Some(self.import_item_image(&product_id, src.path())?);
        }
        let tax_rate = self.resolve_tax_rate(tax_text)?;
        let category_id = self.resolve_category_id(category_id, category_name)?;
        if !category_photo_bytes.is_empty() && !category_id.is_empty() {
            let photo = write_upload(category_photo_filename, category_photo_bytes)?;
            self.set_category_photo_if_missing(&category_id, photo.path())?;
        }
        let station_id = self.resolve_station_id(station_name)?;
        let mut product = if updating {
            self.get_product(&product_id)?
                .ok_or_else(|| "That item is no longer in Items".to_string())?
        } else {
            Product::new_basic(product_id, name, Money(price_paise))
        };
        product.name = name.to_string();
        product.price = Money(price_paise);
        product.unit_label = unit_label.trim().to_string();
        product.kind = if resale {
            domain::ITEM_KIND_RESALE
        } else {
            domain::ITEM_KIND_DISH
        }
        .into();
        if resale && !product.tracks_stock() {
            product.stock_qty = 0;
        }
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
        product.station_id = station_id;
        product.set_modifier_groups(modifier_groups);
        product.set_area_prices(area_prices);
        product.set_schedule(schedule);
        product.group_name = group_name.trim().to_string();
        product.barcode = barcode.trim().to_string();
        product.composition = composition.trim().to_string();
        product.schedule_h = schedule_h;
        product.rx_required = rx_required;
        // Owner prescription note lives on the item as free text in allergens
        // when provided (bookkeeping only — not compliance).
        if !rx_note.trim().is_empty() {
            product.allergens = rx_note.trim().to_string();
        }
        product.hsn_code = hsn_code.trim().to_string();
        if !sizes_only {
            self.upsert_product(&product)?;
            let verb = if updating { "menu.update" } else { "menu.add" };
            let detail = match was {
                Some(old) if old != product.price => {
                    let currency = self.locale_settings()?.currency;
                    let fiscal = self.fiscal_settings()?;
                    format!(
                        "{} · {} → {}",
                        product.name,
                        old.format_localized(&currency, &fiscal),
                        product.price.format_localized(&currency, &fiscal)
                    )
                }
                _ => product.name.clone(),
            };
            self.audit(actor, verb, &detail)?;
            if !recipe_text.trim().is_empty() || !self.recipe(&product.id)?.is_empty() {
                self.set_recipe_from_text(&product.id, recipe_text, actor)?;
            }
        }
        let photo_path = photo.as_ref().map(|f| f.path().to_path_buf()).or_else(|| {
            existing_image_key
                .and_then(|k| k.strip_prefix("custom:"))
                .and_then(|file| self.item_images_dir().ok().map(|d| d.join(file)))
        });
        let sizes = if variations.is_empty() {
            Vec::new()
        } else {
            self.add_variant_items(&product, variations, photo_path.as_deref(), actor)?
        };
        let size_names = sizes.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ");
        Ok(match (updating, sizes.is_empty()) {
            (true, true) => format!("Updated {}", product.name),
            (false, true) => format!("Added {}", product.name),
            (true, false) => format!("Updated {} · added {size_names}", product.name),
            (false, false) => format!("Added {size_names}"),
        })
    }
}
