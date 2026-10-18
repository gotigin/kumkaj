//! Core Kumkaj domain: integer money, catalog, cart, sales, staff, tables/KDS.

use crate::fiscal::{self, FiscalSettings, TaxPricing};
use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Three clocks on every books entry (sale, buy, expense, ledger, stock, bank).
///
/// 1. **user_at** — business time the owner entered (or meant); drives day books.
/// 2. **device_at** — local device clock when the row was written offline.
/// 3. **server_at** — when the cloud accepted the upload; empty until synced.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventTimes {
    pub user_at: String,
    pub device_at: String,
    /// Empty string until a cloud drain stamps it.
    pub server_at: String,
}

impl EventTimes {
    /// Capture now on the device. Optional `user_at` overrides the books time
    /// (RFC3339 or `YYYY-MM-DD`); blank → same as device time.
    pub fn capture(user_at: Option<&str>) -> Self {
        let device_at = Local::now().to_rfc3339();
        let user_at = match user_at.map(str::trim).filter(|s| !s.is_empty()) {
            Some(raw) => normalize_user_at(raw, &device_at),
            None => device_at.clone(),
        };
        Self {
            user_at,
            device_at,
            server_at: String::new(),
        }
    }

    /// Legacy rows that only had `created_at`.
    pub fn from_legacy_created(created_at: &str) -> Self {
        let t = created_at.trim();
        let t = if t.is_empty() {
            Local::now().to_rfc3339()
        } else {
            t.to_string()
        };
        Self {
            user_at: t.clone(),
            device_at: t,
            server_at: String::new(),
        }
    }

    pub fn with_server_now(mut self) -> Self {
        self.server_at = Local::now().to_rfc3339();
        self
    }
}

/// Accept a full RFC3339 stamp or a calendar day (`YYYY-MM-DD` → that day at device clock time).
fn normalize_user_at(raw: &str, device_at: &str) -> String {
    if raw.len() == 10 && raw.as_bytes().get(4) == Some(&b'-') && raw.as_bytes().get(7) == Some(&b'-')
    {
        let clock = device_at.get(11..).unwrap_or("00:00:00");
        return format!("{raw}T{clock}");
    }
    raw.to_string()
}

/// Money in minor currency units (not floats): cents, paise, fils — or whole
/// units for the currencies that have no minor unit at all.
///
/// Intermediate major-unit and × milli maths go through [`crate::precise`]
/// (exact decimal). Overflow becomes an error on checked helpers; the
/// saturating helpers remain for non-critical UI display paths.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money(pub i64);

impl Money {
    pub fn from_rupees(rupees: i64) -> Self {
        Self::from_major_units(rupees, "INR").unwrap_or(Self(0))
    }

    pub fn from_major_units(major: i64, currency_code: &str) -> Result<Self, String> {
        let decimals = crate::locale::currency_by_code(currency_code)
            .map(|c| c.decimals)
            .unwrap_or(2);
        let factor = 10i64
            .checked_pow(decimals as u32)
            .ok_or_else(|| "Currency scale overflow".to_string())?;
        let minor = major
            .checked_mul(factor)
            .ok_or_else(|| "Amount overflow".to_string())?;
        Ok(Self(minor))
    }

    pub fn paise(self) -> i64 {
        self.0
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    pub fn checked_add(self, other: Self) -> Result<Self, String> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or_else(|| "Money overflow".to_string())
    }

    pub fn saturating_mul_qty(self, qty: i64) -> Self {
        Self(self.0.saturating_mul(qty.max(0)))
    }

    /// `unit × qty_milli / 1000` — 1000 milli = one unit (piece or kg).
    /// Exact decimal intermediate; saturates only if the result does not fit i64.
    pub fn saturating_mul_milli(self, qty_milli: i64) -> Self {
        self.checked_mul_milli(qty_milli).unwrap_or(Self(i64::MAX))
    }

    /// `unit × qty_milli / 1000` with overflow error (no silent wrap).
    pub fn checked_mul_milli(self, qty_milli: i64) -> Result<Self, String> {
        use rust_decimal::Decimal;
        let qty = qty_milli.max(0);
        let product = Decimal::from(self.0) * Decimal::from(qty) / Decimal::from(1000);
        Ok(Self(crate::precise::to_i64(product)?))
    }

    pub fn format_with(self, currency_code: &str) -> String {
        crate::locale::format_money(self.0, currency_code)
    }

    /// Same amount in the shop's own number style — `€1.234,50` in Berlin,
    /// `₹1,234.50` in Bengaluru.
    pub fn format_localized(self, currency_code: &str, fiscal: &FiscalSettings) -> String {
        crate::locale::format_money_ex(
            self.0,
            currency_code,
            fiscal.decimal_sep(),
            fiscal.thousands_sep(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayMethod {
    Cash,
    Upi,
    Card,
    /// Customer credit (on account).
    Credit,
    /// Multiple tenders on one bill (see sale_payments).
    Split,
}

impl PayMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::Upi => "upi",
            Self::Card => "card",
            Self::Credit => "credit",
            Self::Split => "split",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cash" => Some(Self::Cash),
            "upi" => Some(Self::Upi),
            "card" => Some(Self::Card),
            "credit" | "udhaar" => Some(Self::Credit),
            "split" => Some(Self::Split),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cash => "Cash",
            Self::Upi => "UPI",
            Self::Card => "Card",
            Self::Credit => "Credit",
            Self::Split => "Split",
        }
    }

    /// Same tender, named the way the seller's country names it: the instant
    /// rail is Pix in Brazil, PayNow in Singapore, FPS in Hong Kong, UPI in
    /// India. Which tender that is comes from the country pack rather than
    /// being hard-coded here, and storage values never change, so old sales
    /// keep reading back correctly.
    pub fn label_for(self, fiscal: &FiscalSettings) -> String {
        if self.as_str() == fiscal.digital_pay_code() {
            return fiscal.digital_pay_label().to_string();
        }
        match self {
            Self::Credit => fiscal.credit_label().to_string(),
            Self::Split => format!("Split Cash+{}", fiscal.digital_pay_label()),
            other => other.label().to_string(),
        }
    }

    pub fn is_credit(self) -> bool {
        matches!(self, Self::Credit)
    }

    pub fn is_split(self) -> bool {
        matches!(self, Self::Split)
    }
}

/// How the bill was served. Counter keeps the chai-stall flow unchanged;
/// DineIn/Takeaway matter for restaurants and KOT routing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    #[default]
    Counter,
    DineIn,
    Takeaway,
}

impl OrderType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::DineIn => "dine_in",
            Self::Takeaway => "takeaway",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "dine_in" | "dinein" | "dine-in" => Self::DineIn,
            "takeaway" | "take_away" | "parcel" => Self::Takeaway,
            _ => Self::Counter,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Counter => "Counter",
            Self::DineIn => "Dine-in",
            Self::Takeaway => "Takeaway",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaffRole {
    /// Host of this local business (server). Only one semantic owner account.
    Owner,
    /// Non-owner staff — permission preset; display name is [`StaffMember::role_title`].
    Manager,
    Cashier,
    /// Takes orders on the floor: sells, runs tables, never touches the books.
    Waiter,
    Kitchen,
}

impl StaffRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Manager => "manager",
            Self::Cashier => "cashier",
            Self::Waiter => "waiter",
            Self::Kitchen => "kitchen",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(Self::Owner),
            "manager" => Some(Self::Manager),
            "cashier" => Some(Self::Cashier),
            "waiter" => Some(Self::Waiter),
            "kitchen" => Some(Self::Kitchen),
            _ => None,
        }
    }

    pub fn is_owner(self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Default label when `role_title` is empty.
    pub fn default_title(self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Manager => "Manager",
            Self::Cashier => "Cashier",
            Self::Waiter => "Waiter",
            Self::Kitchen => "Kitchen",
        }
    }

    pub fn can_sell(self) -> bool {
        matches!(self, Self::Owner | Self::Manager | Self::Cashier | Self::Waiter)
    }

    pub fn can_manage_menu(self) -> bool {
        matches!(self, Self::Owner | Self::Manager)
    }

    pub fn can_void_refund(self) -> bool {
        matches!(self, Self::Owner | Self::Manager)
    }

    /// Amend a recorded order (live state changes; history is appended).
    /// Original sale authorship is never rewritten.
    pub fn can_amend_sale(self) -> bool {
        matches!(self, Self::Owner | Self::Manager)
    }

    pub fn can_kitchen(self) -> bool {
        matches!(self, Self::Owner | Self::Manager | Self::Kitchen)
    }

    pub fn can_admin(self) -> bool {
        matches!(self, Self::Owner)
    }
}

/// Per-staff feature flags (Owner always has all).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffFeatures {
    pub sell: bool,
    pub items: bool,
    pub sales: bool,
    pub credit: bool,
    pub floor: bool,
    pub kitchen: bool,
    pub team: bool,
    pub backup: bool,
    /// Stock, purchases and low-stock views.
    #[serde(default)]
    pub inventory: bool,
    /// Branch switching, snapshots and the consolidated org report.
    #[serde(default)]
    pub branches: bool,
}

impl Default for StaffFeatures {
    fn default() -> Self {
        Self::for_role(StaffRole::Cashier)
    }
}

impl StaffFeatures {
    pub fn all() -> Self {
        Self {
            sell: true,
            items: true,
            sales: true,
            credit: true,
            floor: true,
            kitchen: true,
            team: true,
            backup: true,
            inventory: true,
            branches: true,
        }
    }

    pub fn for_role(role: StaffRole) -> Self {
        match role {
            StaffRole::Owner => Self::all(),
            StaffRole::Manager => Self {
                sell: true,
                items: true,
                sales: true,
                credit: true,
                floor: true,
                kitchen: true,
                team: false,
                backup: false,
                inventory: true,
                branches: true,
            },
            StaffRole::Cashier => Self {
                sell: true,
                items: false,
                sales: false,
                credit: true,
                floor: false,
                kitchen: false,
                team: false,
                backup: false,
                inventory: false,
                branches: false,
            },
            // A waiter is a cashier who also works the floor.
            StaffRole::Waiter => Self {
                sell: true,
                items: false,
                sales: false,
                credit: true,
                floor: true,
                kitchen: false,
                team: false,
                backup: false,
                inventory: false,
                branches: false,
            },
            StaffRole::Kitchen => Self {
                sell: false,
                items: false,
                sales: false,
                credit: false,
                floor: false,
                kitchen: true,
                team: false,
                backup: false,
                inventory: false,
                branches: false,
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".into())
    }

    pub fn from_json(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VenueMode {
    Counter,
    Restaurant,
}

impl VenueMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Restaurant => "restaurant",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "restaurant" => Self::Restaurant,
            _ => Self::Counter,
        }
    }

    pub fn creates_kot(self) -> bool {
        matches!(self, Self::Restaurant)
    }
}

/// Bill-type tag on a sale (not separate ledgers).
///
/// Industry wording: **Tax invoice / Account booking** (shareable, GST path) vs
/// **Cash memo / Cash booking** (spot / management). Same sales table; reports
/// can total either tag or both. Labels come from the start template.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoneyLane {
    #[default]
    Declared,
    Owner,
}

impl MoneyLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Owner => "owner",
        }
    }

    pub fn parse(s: &str) -> Self {
        let key = s.trim().to_ascii_lowercase();
        match key.as_str() {
            // Prefer Account/Cash booking · Tax invoice/Cash memo — never legacy slang.
            "owner" | "cash" | "cash_booking" | "cash_memo" | "internal" => Self::Owner,
            _ => Self::Declared,
        }
    }
}

/// Milli-units helper: whole pieces use multiples of 1000.
pub const QTY_MILLI_UNIT: i64 = 1000;

pub fn qty_to_milli(qty: i64) -> i64 {
    qty.saturating_mul(QTY_MILLI_UNIT)
}

pub fn milli_to_whole_qty(qty_milli: i64) -> i64 {
    if qty_milli <= 0 {
        0
    } else {
        (qty_milli + QTY_MILLI_UNIT - 1) / QTY_MILLI_UNIT
    }
}

/// Display qty for receipts/reports: `x2` (piece) or `1.25 kg` (unit label).
pub fn format_qty_display(qty_milli: i64, qty_whole: i64, unit_label: &str) -> String {
    let milli = if qty_milli > 0 {
        qty_milli
    } else {
        qty_whole.saturating_mul(QTY_MILLI_UNIT)
    };
    let unit = unit_label.trim();
    if unit.is_empty() {
        return format!("x{}", milli_to_whole_qty(milli).max(1));
    }
    let whole = milli / QTY_MILLI_UNIT;
    let frac = (milli % QTY_MILLI_UNIT).unsigned_abs();
    if frac == 0 {
        format!("{whole} {unit}")
    } else {
        let mut frac_s = format!("{frac:03}");
        while frac_s.ends_with('0') {
            frac_s.pop();
        }
        format!("{whole}.{frac_s} {unit}")
    }
}

/// Cashbook / bank movement kinds (generic — not a trade fork).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoneyMoveKind {
    /// Cash from till into bank.
    TillToBank,
    /// Cash from bank back to till.
    BankToTill,
    /// Other money into bank (transfer in, deposit).
    BankIn,
    /// Other money out of bank (withdrawal, payment).
    BankOut,
}

impl MoneyMoveKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TillToBank => "till_to_bank",
            Self::BankToTill => "bank_to_till",
            Self::BankIn => "bank_in",
            Self::BankOut => "bank_out",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "till_to_bank" => Some(Self::TillToBank),
            "bank_to_till" => Some(Self::BankToTill),
            "bank_in" => Some(Self::BankIn),
            "bank_out" => Some(Self::BankOut),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::TillToBank => "Till → bank",
            Self::BankToTill => "Bank → till",
            Self::BankIn => "Bank in",
            Self::BankOut => "Bank out",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoneyMovement {
    pub id: String,
    /// Books day — same as [`EventTimes::user_at`] (kept for older callers).
    pub created_at: String,
    #[serde(default)]
    pub user_at: String,
    #[serde(default)]
    pub device_at: String,
    #[serde(default)]
    pub server_at: String,
    pub kind: String,
    pub amount_paise: i64,
    pub note: String,
    pub actor_name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartyAgingRow {
    pub customer_id: String,
    pub name: String,
    pub kind_label: String,
    /// Positive = they owe; negative = advance held.
    pub balance_paise: i64,
    /// Days since oldest open charge (0 if unknown / advance).
    pub days: i64,
    /// `current` (0–7) | `due` (8–30) | `overdue` (31+) | `advance`
    pub bucket: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PeriodBooks {
    pub from_day: String,
    pub to_day: String,
    pub sale_count: i64,
    pub revenue_paise: i64,
    pub declared_paise: i64,
    pub owner_lane_paise: i64,
    pub cash_paise: i64,
    pub upi_paise: i64,
    pub card_paise: i64,
    pub credit_paise: i64,
    pub tax_paise: i64,
    /// Cost of goods from purchases in the period.
    pub cogs_paise: i64,
    pub expenses_paise: i64,
    pub expense_count: i64,
    pub bank_in_paise: i64,
    pub bank_out_paise: i64,
    pub outstanding_credit_paise: i64,
    /// Whom we owe (AP).
    pub outstanding_payable_paise: i64,
}

impl PeriodBooks {
    pub fn gross_paise(&self) -> i64 {
        self.revenue_paise.saturating_sub(self.cogs_paise)
    }

    pub fn net_paise(&self) -> i64 {
        self.gross_paise().saturating_sub(self.expenses_paise)
    }
}

/// Catalogue unit (builtin or owner-defined).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: String,
    pub code: String,
    pub name: String,
    pub builtin: bool,
}

/// `1 from_code = (factor_milli / 1000) to_code` when qty is in milli-units:
/// `to_qty_milli = from_qty_milli * factor_milli / 1000`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UnitConversion {
    pub id: String,
    pub from_code: String,
    pub to_code: String,
    pub factor_milli: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaffMember {
    pub id: String,
    pub name: String,
    /// Owner-defined role label for staff (e.g. "Barista"). Owner uses "Owner".
    pub role_title: String,
    pub role: StaffRole,
    pub pin_hash: String,
    pub active: bool,
    /// Optional login id (Slice 4). Empty = PIN-only.
    #[serde(default)]
    pub login_id: String,
    /// Optional password hash. Empty = PIN-only.
    #[serde(default)]
    pub password_hash: String,
    #[serde(default)]
    pub features: StaffFeatures,
}

impl StaffMember {
    pub fn is_owner(&self) -> bool {
        self.role.is_owner()
    }

    pub fn display_role(&self) -> String {
        let t = self.role_title.trim();
        if t.is_empty() {
            self.role.default_title().to_string()
        } else {
            t.to_string()
        }
    }

    pub fn lock_label(&self) -> String {
        if self.is_owner() {
            format!("{} (Owner)", self.name)
        } else {
            format!("{} · {}", self.name, self.display_role())
        }
    }

    pub fn effective_features(&self) -> StaffFeatures {
        if self.is_owner() {
            StaffFeatures::all()
        } else {
            self.features.clone()
        }
    }

    pub fn can_access_sell(&self) -> bool {
        self.effective_features().sell
    }

    pub fn can_access_items(&self) -> bool {
        self.effective_features().items
    }

    pub fn can_access_sales(&self) -> bool {
        self.effective_features().sales
    }

    pub fn can_access_credit(&self) -> bool {
        self.effective_features().credit
    }

    pub fn can_access_floor(&self) -> bool {
        self.effective_features().floor
    }

    pub fn can_access_kitchen(&self) -> bool {
        self.effective_features().kitchen
    }

    pub fn can_access_team(&self) -> bool {
        self.effective_features().team
    }

    pub fn can_access_backup(&self) -> bool {
        self.effective_features().backup
    }

    pub fn can_access_inventory(&self) -> bool {
        self.effective_features().inventory
    }

    pub fn can_access_branches(&self) -> bool {
        self.effective_features().branches
    }
}

/// One business (org). A single device always belongs to exactly one org;
/// branches are the shops/outlets under it.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Branch {
    pub id: String,
    pub org_id: String,
    pub name: String,
    /// Short code for receipts / bill numbers (e.g. `MAIN`, `MG02`).
    pub code: String,
    pub active: bool,
    /// True for the branch this device sells for.
    pub is_this_device: bool,
    /// Last imported [`BranchSnapshot`] JSON for remote branches (empty locally).
    pub last_snapshot_json: String,
    pub last_sync_at: String,
}

impl Branch {
    pub fn label(&self) -> String {
        if self.code.trim().is_empty() {
            self.name.clone()
        } else {
            format!("{} ({})", self.name, self.code)
        }
    }
}

/// Menu grouping for large catalogues (Drinks / Food / Starters …).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub branch_id: String,
    pub name: String,
    pub sort_order: i64,
}

/// Kitchen station a line is routed to (Hot / Cold / Tandoor …).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KitchenStation {
    pub id: String,
    pub branch_id: String,
    pub name: String,
    pub sort_order: i64,
}

/// One paid add-on for an item (`Extra cheese +₹20`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifierOption {
    pub name: String,
    #[serde(default)]
    pub price_paise: i64,
}

impl ModifierOption {
    /// Parse a `modifiers_json` column. Bad/empty JSON reads as "no modifiers".
    pub fn parse_list(json: &str) -> Vec<Self> {
        let s = json.trim();
        if s.is_empty() {
            return Vec::new();
        }
        serde_json::from_str(s).unwrap_or_default()
    }

    pub fn serialize_list(list: &[Self]) -> String {
        serde_json::to_string(list).unwrap_or_else(|_| "[]".into())
    }

    pub fn price(&self) -> Money {
        Money(self.price_paise)
    }

    /// Parse `Extra cheese=20, Less sugar=0` (major currency units for the price).
    pub fn parse_human_list(text: &str, currency_code: &str) -> Result<Vec<Self>, String> {
        let mut out = Vec::new();
        for part in text.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let (name, price_major) = match part.split_once('=') {
                Some((n, p)) => (n.trim(), p.trim()),
                None => (part, "0"),
            };
            if name.is_empty() {
                return Err("Modifier name required".into());
            }
            let price_paise = if price_major.is_empty() {
                0
            } else {
                crate::locale::parse_major_input(price_major, currency_code)
                    .map_err(|_| format!("Bad modifier price for '{name}'"))?
            };
            if price_paise < 0 {
                return Err("Modifier price cannot be negative".into());
            }
            out.push(Self {
                name: name.to_string(),
                price_paise,
            });
        }
        Ok(out)
    }
}

/// One named rate a shop charges, seeded from its country pack and editable.
/// Items point at a rate by id; `tax_bps` on the item stays in sync so every
/// historical read path keeps working without a join.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TaxRate {
    pub id: String,
    pub name: String,
    /// Short seller-facing key (`5`, `20`, `0`) used to match typed input.
    pub code: String,
    pub bps: i64,
    pub active: bool,
}

impl TaxRate {
    /// `18%`, `13.5%`, or `0%` for exempt rates.
    pub fn percent_label(&self) -> String {
        let whole = self.bps / 100;
        let frac = format!("{:02}", self.bps % 100);
        let frac = frac.trim_end_matches('0');
        if frac.is_empty() {
            format!("{whole}%")
        } else {
            format!("{whole}.{frac}%")
        }
    }

    /// True when the seller typed this rate's name or code (case-insensitive).
    pub fn matches(&self, typed: &str) -> bool {
        let t = typed.trim();
        self.name.eq_ignore_ascii_case(t) || (!self.code.is_empty() && self.code.eq_ignore_ascii_case(t))
    }
}

/// `stock_qty` value meaning "this item is not stock tracked".
pub const STOCK_UNTRACKED: i64 = -1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: Money,
    pub available: bool,
    pub archived: bool,
    /// Custom photo (`custom:…` / `pending_custom:…`), or `None` for name-initials avatar.
    pub image_key: Option<String>,
    /// Catalogue grouping id; empty = uncategorised.
    #[serde(default)]
    pub category_id: String,
    /// Optional prep-station id (food products); unused on courier rates.
    #[serde(default)]
    pub station_id: String,
    /// Tax rate in basis points (500 = 5%). Whether this sits *inside* the
    /// price or is added at the till is the shop's [`crate::fiscal::TaxPricing`].
    #[serde(default)]
    pub tax_bps: i64,
    /// [`TaxRate`] this item follows; empty = ad-hoc `tax_bps` only.
    #[serde(default)]
    pub tax_rate_id: String,
    /// On-hand quantity, or [`STOCK_UNTRACKED`] (−1) when not tracked.
    #[serde(default = "stock_untracked")]
    pub stock_qty: i64,
    /// JSON array of [`ModifierOption`].
    #[serde(default = "empty_json_array")]
    pub modifiers_json: String,
    /// Other names the seller types for this same item. The id is what
    /// stays unique; these strings are never translated when the UI language
    /// changes.
    #[serde(default = "empty_json_array")]
    pub aliases_json: String,
    /// Owning branch; empty rows are treated as the active branch (legacy).
    #[serde(default)]
    pub branch_id: String,
    /// Quantity unit label (`kg`, empty = piece). Rate × weight uses this.
    #[serde(default)]
    pub unit_label: String,
    /// Destination zone id from the zone master (empty = unzoned).
    #[serde(default)]
    pub zone_id: String,
    /// First-kg slab weight; 0 = flat ₹/kg via `price`.
    #[serde(default)]
    pub first_kg: rust_decimal::Decimal,
    /// Paise for the first slab.
    #[serde(default)]
    pub first_paise: i64,
    /// Paise per kg beyond `first_kg`.
    #[serde(default)]
    pub extra_per_kg_paise: i64,
}

fn stock_untracked() -> i64 {
    STOCK_UNTRACKED
}

fn empty_json_array() -> String {
    "[]".into()
}

/// Untracked stock and an empty modifier list — never `0` stock by accident.
impl Default for Product {
    fn default() -> Self {
        Self::new_basic(String::new(), String::new(), Money(0))
    }
}

impl Product {
    /// Minimal product with the new columns at their defaults — use from UI so
    /// new fields never have to be spelled out at every call site.
    pub fn new_basic(id: impl Into<String>, name: impl Into<String>, price: Money) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            price,
            available: true,
            archived: false,
            image_key: None,
            category_id: String::new(),
            station_id: String::new(),
            tax_bps: 0,
            tax_rate_id: String::new(),
            stock_qty: STOCK_UNTRACKED,
            modifiers_json: "[]".into(),
            aliases_json: "[]".into(),
            branch_id: String::new(),
            unit_label: String::new(),
            zone_id: String::new(),
            first_kg: rust_decimal::Decimal::ZERO,
            first_paise: 0,
            extra_per_kg_paise: 0,
        }
    }

    pub fn rate_slab(&self) -> crate::rates::RateSlab {
        crate::rates::RateSlab {
            first_kg: self.first_kg,
            first_paise: self.first_paise,
            extra_per_kg_paise: self.extra_per_kg_paise,
            flat_per_kg_paise: self.price.paise(),
        }
    }

    pub fn is_weight_unit(&self) -> bool {
        let u = self.unit_label.trim().to_lowercase();
        u == "kg" || u == "g" || u == "lb" || u == "oz"
    }

    /// Extra names, in the order the seller typed them.
    pub fn aliases(&self) -> Vec<String> {
        serde_json::from_str(&self.aliases_json).unwrap_or_default()
    }

    pub fn set_aliases(&mut self, aliases: &[String]) {
        self.aliases_json = serde_json::to_string(aliases).unwrap_or_else(|_| "[]".into());
    }

    /// Display name plus aliases. Comparison is case-insensitive and does
    /// not depend on the UI language.
    pub fn matches_label(&self, needle: &str) -> bool {
        let needle = needle.trim();
        if needle.is_empty() {
            return false;
        }
        let needle = needle.to_lowercase();
        self.name.to_lowercase() == needle
            || self
                .aliases()
                .iter()
                .any(|alias| alias.to_lowercase() == needle)
    }

    pub fn tracks_stock(&self) -> bool {
        self.stock_qty >= 0
    }

    /// Untracked items are always sellable; tracked items need stock on hand.
    pub fn in_stock(&self) -> bool {
        !self.tracks_stock() || self.stock_qty > 0
    }

    pub fn modifiers(&self) -> Vec<ModifierOption> {
        ModifierOption::parse_list(&self.modifiers_json)
    }

    pub fn set_modifiers(&mut self, list: &[ModifierOption]) {
        self.modifiers_json = ModifierOption::serialize_list(list);
    }

    /// Tax this item contributes for `amount_paise` under `pricing` — inside
    /// the amount when Inclusive, added on top when Exclusive.
    pub fn tax_for(&self, amount_paise: i64, pricing: TaxPricing) -> i64 {
        fiscal::tax_on_amount(amount_paise, self.tax_bps, pricing)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CartLine {
    pub product_id: String,
    pub name: String,
    pub unit_price: Money,
    /// Whole-unit mirror of [`Self::qty_milli`] / 1000 (ceil) for stock checks.
    pub qty: i64,
    /// Milli-units (1000 = 1 piece or 1 kg). Source of truth for line math.
    pub qty_milli: i64,
    /// Copied from the product at add time (`kg`, empty = piece).
    pub unit_label: String,
    /// Optional seller note for this line (e.g. "less sugar"). Empty = none.
    /// Modifier names are kept separately and joined onto the kitchen note.
    pub note: String,
    /// Catalogue price before modifiers. `unit_price` is this plus the
    /// applied modifier prices.
    pub base_unit_paise: i64,
    pub applied_modifiers: Vec<ModifierOption>,
    /// Which page of modifier chips this line is showing. Three fit on a row.
    pub modifier_page: usize,
    /// Applied line total when it differs from Actual (`unit_price × qty`).
    /// Unit price is never rewritten when this is set.
    pub line_total_override: Option<Money>,
}

/// How many modifier chips fit on one cart row.
pub const MODIFIER_PAGE: usize = 3;

impl CartLine {
    fn sync_qty_from_milli(&mut self) {
        self.qty = milli_to_whole_qty(self.qty_milli);
    }

    /// Step when +/- is tapped: 1.0 unit, or 0.1 for weight units.
    pub fn bump_step_milli(&self) -> i64 {
        let u = self.unit_label.trim().to_lowercase();
        if u == "kg" || u == "lb" {
            100
        } else if u == "g" || u == "oz" {
            10
        } else {
            QTY_MILLI_UNIT
        }
    }

    /// Actual line total: `unit_price × qty_milli / 1000`.
    pub fn actual_total(&self) -> Money {
        let milli = if self.qty_milli > 0 {
            self.qty_milli
        } else {
            qty_to_milli(self.qty)
        };
        self.unit_price.saturating_mul_milli(milli)
    }

    /// Applied line total: override if set, otherwise Actual.
    pub fn applied_total(&self) -> Money {
        self.line_total_override
            .unwrap_or_else(|| self.actual_total())
    }

    /// Alias for [`CartLine::applied_total`] (charged amount for this line).
    pub fn line_total(&self) -> Money {
        self.applied_total()
    }

    /// Line-level rounding: `Applied − Actual` (negative = round down).
    pub fn rounding(&self) -> Money {
        Money(
            self.applied_total()
                .paise()
                .saturating_sub(self.actual_total().paise()),
        )
    }

    /// What the kitchen and the receipt should read: chosen modifiers, then
    /// any free-text note the seller typed.
    pub fn kitchen_note(&self) -> String {
        let mods = self
            .applied_modifiers
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        match (mods.is_empty(), self.note.is_empty()) {
            (true, _) => self.note.clone(),
            (false, true) => mods,
            (false, false) => format!("{mods}, {}", self.note),
        }
    }

    fn reprice_from_modifiers(&mut self) {
        let extra: i64 = self.applied_modifiers.iter().map(|m| m.price_paise).sum();
        self.unit_price = Money(self.base_unit_paise.saturating_add(extra));
    }
}

#[derive(Clone, Debug, Default)]
pub struct Cart {
    pub lines: Vec<CartLine>,
    /// Optional charged total for this open cart (e.g. round ₹52 → ₹50).
    /// Cleared whenever lines/qty/unit prices change so rounding never sticks
    /// to a stale basket. Applied only at finalize via [`Cart::to_sale_lines`].
    pub charge_total: Option<Money>,
    /// How this bill is served. Counter by default so the stall flow is unchanged.
    pub order_type: OrderType,
    /// Bill-type tag (Account booking / Cash booking). Set from the shell.
    pub money_lane: MoneyLane,
}

impl Cart {
    pub fn clear(&mut self) {
        self.lines.clear();
        self.charge_total = None;
        self.order_type = OrderType::Counter;
        // Keep money_lane — owner choice persists across carts until they flip the chip.
    }

    pub fn set_order_type(&mut self, order_type: OrderType) {
        self.order_type = order_type;
    }

    pub fn add_or_bump(&mut self, product: &Product) {
        if !product.available || product.archived || product.price.paise() <= 0 {
            return;
        }
        self.charge_total = None;
        if let Some(line) = self
            .lines
            .iter_mut()
            .find(|l| l.product_id == product.id)
        {
            line.line_total_override = None;
            let step = line.bump_step_milli();
            line.qty_milli = line.qty_milli.saturating_add(step);
            line.sync_qty_from_milli();
        } else {
            let step = if product.is_weight_unit() { 100 } else { QTY_MILLI_UNIT };
            let mut line = CartLine {
                product_id: product.id.clone(),
                name: product.name.clone(),
                unit_price: product.price,
                qty: 0,
                qty_milli: step,
                unit_label: product.unit_label.clone(),
                note: String::new(),
                base_unit_paise: product.price.paise(),
                applied_modifiers: Vec::new(),
                modifier_page: 0,
                line_total_override: None,
            };
            line.sync_qty_from_milli();
            self.lines.push(line);
        }
    }

    /// How many of `product_id` this cart already holds.
    pub fn qty_of(&self, product_id: &str) -> i64 {
        self.lines
            .iter()
            .filter(|l| l.product_id == product_id)
            .map(|l| l.qty)
            .sum()
    }

    /// Add one, refusing to book more of a tracked item than the shop has. The
    /// silent [`Cart::add_or_bump`] let a seller build a cart that could only
    /// fail at the till.
    pub fn add_or_bump_in_stock(&mut self, product: &Product) -> Result<(), String> {
        if !product.available || product.archived {
            return Err(format!("{} is marked sold out", product.name));
        }
        if product.price.paise() <= 0 {
            return Err(format!("{} has no price yet", product.name));
        }
        if !product.in_stock() {
            return Err(format!("{} is out of stock", product.name));
        }
        if product.tracks_stock() && self.qty_of(&product.id) >= product.stock_qty {
            return Err(format!(
                "Only {} of {} in stock",
                product.stock_qty, product.name
            ));
        }
        self.add_or_bump(product);
        Ok(())
    }

    pub fn bump_qty(&mut self, index: usize, delta: i64) {
        if let Some(line) = self.lines.get_mut(index) {
            self.charge_total = None;
            line.line_total_override = None;
            let step = line.bump_step_milli().saturating_mul(delta);
            line.qty_milli = (line.qty_milli + step).max(0);
            line.sync_qty_from_milli();
            if line.qty_milli == 0 {
                self.lines.remove(index);
            }
        }
    }

    /// Set absolute milli-qty for a line (e.g. typed weight).
    pub fn set_qty_milli(&mut self, index: usize, qty_milli: i64) {
        if let Some(line) = self.lines.get_mut(index) {
            self.charge_total = None;
            line.line_total_override = None;
            line.qty_milli = qty_milli.max(0);
            line.sync_qty_from_milli();
            if line.qty_milli == 0 {
                self.lines.remove(index);
            }
        }
    }

    /// Edit unit price for this open cart only — does not change the catalogue.
    pub fn set_unit_price(&mut self, index: usize, unit_price: Money) -> Result<(), String> {
        if unit_price.paise() <= 0 {
            return Err("Unit price must be > 0".into());
        }
        let line = self
            .lines
            .get_mut(index)
            .ok_or_else(|| "Missing cart line".to_string())?;
        // Blurring the field without a change must not drop modifiers. A new
        // number is the price the seller means, so the chips clear.
        if unit_price.paise() == line.unit_price.paise() {
            return Ok(());
        }
        line.base_unit_paise = unit_price.paise();
        line.applied_modifiers.clear();
        line.unit_price = unit_price;
        line.line_total_override = None;
        self.charge_total = None;
        Ok(())
    }

    /// Turn a catalogue modifier on or off for this line. The unit price
    /// follows. Returns whether it is on after the tap.
    pub fn toggle_modifier(
        &mut self,
        index: usize,
        modifier: &ModifierOption,
    ) -> Result<bool, String> {
        if modifier.name.trim().is_empty() {
            return Err("Modifier name required".into());
        }
        let line = self
            .lines
            .get_mut(index)
            .ok_or_else(|| "Missing cart line".to_string())?;
        let turning_on = !line
            .applied_modifiers
            .iter()
            .any(|m| m.name == modifier.name);
        if turning_on {
            line.applied_modifiers.push(modifier.clone());
        } else {
            line.applied_modifiers.retain(|m| m.name != modifier.name);
        }
        line.reprice_from_modifiers();
        if line.unit_price.paise() <= 0 {
            return Err("Unit price must be > 0".into());
        }
        line.line_total_override = None;
        self.charge_total = None;
        Ok(turning_on)
    }

    pub fn shift_modifier_page(&mut self, index: usize, delta: isize, count: usize) {
        let pages = count.div_ceil(MODIFIER_PAGE).max(1);
        if let Some(line) = self.lines.get_mut(index) {
            let cur = line.modifier_page % pages;
            line.modifier_page = (cur as isize + delta).rem_euclid(pages as isize) as usize;
        }
    }

    /// Set Applied line total. Does **not** change `@` unit price.
    /// Clears the override when Applied equals Actual (`unit × qty`).
    pub fn set_line_total(&mut self, index: usize, total: Money) -> Result<(), String> {
        if total.paise() <= 0 {
            return Err("Line Applied Total must be > 0".into());
        }
        let line = self
            .lines
            .get_mut(index)
            .ok_or_else(|| "Missing cart line".to_string())?;
        if line.qty <= 0 {
            return Err("Quantity must be > 0".into());
        }
        let actual = line.actual_total().paise();
        if total.paise() == actual {
            line.line_total_override = None;
        } else {
            line.line_total_override = Some(total);
        }
        self.charge_total = None;
        Ok(())
    }

    /// Optional free-text note for a cart line (kitchen / memory aid).
    pub fn set_note(&mut self, index: usize, note: impl Into<String>) -> Result<(), String> {
        let line = self
            .lines
            .get_mut(index)
            .ok_or_else(|| "Missing cart line".to_string())?;
        line.note = note.into().trim().to_string();
        Ok(())
    }

    /// Set Rounded Total (bill amount after seller round-off).
    /// Clears when equal to Applied Total (sum of line Applied).
    pub fn set_charge_total(&mut self, total: Money) -> Result<(), String> {
        if self.lines.is_empty() {
            return Err("Cart is empty".into());
        }
        if total.paise() <= 0 {
            return Err("Total must be > 0".into());
        }
        if total.paise() == self.applied_total().paise() {
            self.charge_total = None;
        } else {
            self.charge_total = Some(total);
        }
        Ok(())
    }

    /// Actual Total: sum of `@ × qty` on every line.
    pub fn actual_total(&self) -> Money {
        self.lines
            .iter()
            .fold(Money(0), |acc, line| acc.saturating_add(line.actual_total()))
    }

    /// Applied Total: sum of each line's Applied amount.
    pub fn applied_total(&self) -> Money {
        self.lines
            .iter()
            .fold(Money(0), |acc, line| acc.saturating_add(line.applied_total()))
    }

    /// Alias used by older call sites.
    pub fn line_sum(&self) -> Money {
        self.applied_total()
    }

    /// Rounded Total: amount collected / written to the sale.
    /// Defaults to Applied Total until the seller edits it.
    pub fn rounded_total(&self) -> Money {
        self.charge_total
            .unwrap_or_else(|| self.applied_total())
    }

    /// Alias for [`Cart::rounded_total`].
    pub fn total(&self) -> Money {
        self.rounded_total()
    }

    /// Bill-level round-off: `Rounded Total − Applied Total` (negative = round down).
    pub fn rounding(&self) -> Money {
        Money(
            self.rounded_total()
                .paise()
                .saturating_sub(self.applied_total().paise()),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Load a recorded sale into an editable cart for owner/manager amend.
    pub fn from_sale(sale: &Sale) -> Self {
        let mut cart = Self {
            order_type: OrderType::parse(&sale.order_type),
            lines: sale
                .lines
                .iter()
                .map(|l| {
                    let milli = if l.qty_milli > 0 {
                        l.qty_milli
                    } else {
                        qty_to_milli(l.qty)
                    };
                    let computed = Money(l.unit_price_paise).saturating_mul_milli(milli);
                    let override_total = if l.line_total_paise != computed.paise() {
                        Some(Money(l.line_total_paise))
                    } else {
                        None
                    };
                    let mut line = CartLine {
                        product_id: l.product_id.clone(),
                        name: l.name.clone(),
                        unit_price: Money(l.unit_price_paise),
                        qty: l.qty,
                        qty_milli: milli,
                        unit_label: l.unit_label.clone(),
                        note: l.note.clone(),
                        base_unit_paise: l.unit_price_paise,
                        applied_modifiers: Vec::new(),
                        modifier_page: 0,
                        line_total_override: override_total,
                    };
                    line.sync_qty_from_milli();
                    line
                })
                .collect(),
            charge_total: None,
            money_lane: MoneyLane::parse(&sale.money_lane),
        };
        if sale.rounding_paise != 0 {
            cart.charge_total = Some(Money(sale.total_paise));
        }
        cart
    }

    /// Build a sale draft.
    ///
    /// Line unit prices and totals stay as the seller set them on the cart.
    /// Any bill-level round/override becomes an explicit `rounding_paise`
    /// so accounts can reconcile: `lines_total + rounding = total`.
    pub fn to_sale_draft(&self) -> Result<SaleDraft, String> {
        if self.lines.is_empty() {
            return Err("Cart is empty".into());
        }
        let lines_total = self.line_sum().paise();
        if lines_total <= 0 {
            return Err("Sale total must be positive".into());
        }
        let total = self.total().paise();
        if total <= 0 {
            return Err("Sale total must be positive".into());
        }
        let rounding = total.saturating_sub(lines_total);
        let lines: Vec<SaleLine> = self
            .lines
            .iter()
            .map(|l| {
                let milli = if l.qty_milli > 0 {
                    l.qty_milli
                } else {
                    qty_to_milli(l.qty)
                };
                SaleLine {
                    product_id: l.product_id.clone(),
                    name: l.name.clone(),
                    unit_price_paise: l.unit_price.paise(),
                    qty: l.qty,
                    line_total_paise: l.applied_total().paise(),
                    note: l.kitchen_note(),
                    qty_milli: milli,
                    unit_label: l.unit_label.clone(),
                }
            })
            .collect();
        // Applied line totals must sum to lines_total; unit prices stay true.
        debug_assert_eq!(
            lines.iter().map(|l| l.line_total_paise).sum::<i64>(),
            lines_total
        );
        Ok(SaleDraft {
            lines,
            lines_total_paise: lines_total,
            rounding_paise: rounding,
            total_paise: total,
            // The cart is deliberately tax-agnostic under both pricing modes.
            // The store resolves the shop's country pack at finalize: it
            // extracts inclusive tax, or adds exclusive tax on top of this
            // total. Doing it here would let a stale pack price a bill.
            tax_paise: 0,
            order_type: self.order_type,
            money_lane: self.money_lane,
            user_at: None,
            awb_no: String::new(),
            destination: String::new(),
            consignee_name: String::new(),
            actual_weight_kg: rust_decimal::Decimal::ZERO,
            volumetric_weight_kg: rust_decimal::Decimal::ZERO,
            chargeable_weight_kg: rust_decimal::Decimal::ZERO,
            zone_id: String::new(),
            cod_paise: 0,
            carrier_cost_paise: 0,
            extras_json: "[]".into(),
            notify_phone: String::new(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaleLine {
    pub product_id: String,
    pub name: String,
    pub unit_price_paise: i64,
    pub qty: i64,
    pub line_total_paise: i64,
    /// Optional note captured at sale time (empty if none).
    #[serde(default)]
    pub note: String,
    /// Milli-units (1000 = 1). 0 on legacy rows means `qty × 1000`.
    #[serde(default)]
    pub qty_milli: i64,
    /// Unit label frozen at sale time (`kg`, empty = piece).
    #[serde(default)]
    pub unit_label: String,
}

/// Snapshot ready to persist. Prior snapshots stay in revision history;
/// the live sale row may be amended by permitted staff.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaleDraft {
    pub lines: Vec<SaleLine>,
    /// Sum of line totals before bill-level rounding.
    pub lines_total_paise: i64,
    /// `total_paise - lines_total_paise` (e.g. −200 for ₹52 → ₹50).
    pub rounding_paise: i64,
    /// Amount the seller quoted and the cart agreed on.
    ///
    /// Under Inclusive pricing this is already the charged total. Under
    /// Exclusive pricing the store adds tax to it at finalize, so the *sale*
    /// total can be higher than the draft total.
    pub total_paise: i64,
    /// Tax **inside** `lines_total_paise`. Carts always leave this at 0; the
    /// store fills it from the shop's country pack at finalize, which is why
    /// [`SaleDraft::assert_balanced`] only ever checks the untaxed equation.
    #[serde(default)]
    pub tax_paise: i64,
    #[serde(default)]
    pub order_type: OrderType,
    /// Which money lane this bill posts to.
    #[serde(default)]
    pub money_lane: MoneyLane,
    /// Optional books time override (`YYYY-MM-DD` or RFC3339). Empty → device now.
    #[serde(default)]
    pub user_at: Option<String>,
    /// AWB / CN — filled by the store at finalize when empty.
    #[serde(default)]
    pub awb_no: String,
    /// Destination city / zone text from the booking desk.
    #[serde(default)]
    pub destination: String,
    /// Consignee (receiver) name — usually not the paying party.
    #[serde(default)]
    pub consignee_name: String,
    #[serde(default)]
    pub actual_weight_kg: rust_decimal::Decimal,
    #[serde(default)]
    pub volumetric_weight_kg: rust_decimal::Decimal,
    #[serde(default)]
    pub chargeable_weight_kg: rust_decimal::Decimal,
    #[serde(default)]
    pub zone_id: String,
    /// Cash-on-delivery amount to collect from consignee (paise).
    #[serde(default)]
    pub cod_paise: i64,
    /// Expected carrier / rider payout cost (paise) for margin.
    #[serde(default)]
    pub carrier_cost_paise: i64,
    /// JSON array of `{kind,name,amount_paise}` extras already in lines or noted.
    #[serde(default = "empty_json_array")]
    pub extras_json: String,
    /// Optional phone for booking alert outbox.
    #[serde(default)]
    pub notify_phone: String,
}

impl SaleDraft {
    /// A draft balances under **both** pricing modes, because it is quoted
    /// before tax is resolved: `lines_total + rounding = total`, every line
    /// adds up, and the bill is non-empty and positive. Exclusive tax is added
    /// by the store on top of `total_paise` and never checked here.
    pub fn assert_balanced(&self) -> Result<(), String> {
        let sum: i64 = self.lines.iter().map(|l| l.line_total_paise).sum();
        if sum != self.lines_total_paise {
            return Err("Line totals do not match lines_total".into());
        }
        if self.lines_total_paise.saturating_add(self.rounding_paise) != self.total_paise {
            return Err("lines_total + rounding must equal charged total".into());
        }
        if self.tax_paise < 0 || self.tax_paise > self.lines_total_paise {
            return Err("Draft tax must be between 0 and lines_total".into());
        }
        if self.total_paise <= 0 {
            return Err("Sale total must be positive".into());
        }
        if self.lines.is_empty() {
            return Err("Cart is empty".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Sale {
    pub id: String,
    /// Books / user time — prefer [`Self::user_at`]; kept equal on write.
    pub created_at: String,
    #[serde(default)]
    pub user_at: String,
    #[serde(default)]
    pub device_at: String,
    #[serde(default)]
    pub server_at: String,
    pub lines_total_paise: i64,
    pub rounding_paise: i64,
    pub total_paise: i64,
    pub pay_method: String,
    pub status: String,
    pub table_id: Option<String>,
    #[serde(default)]
    pub customer_id: Option<String>,
    /// Staff who finalized the sale — never rewritten on amend.
    #[serde(default)]
    pub created_by_id: String,
    #[serde(default)]
    pub created_by_name: String,
    /// Branch that recorded this sale (empty = legacy row on the active branch).
    #[serde(default)]
    pub branch_id: String,
    /// [`OrderType::as_str`] value; empty/unknown reads as Counter.
    #[serde(default)]
    pub order_type: String,
    /// Tax on this bill. Inside `total_paise` when the shop prices inclusive,
    /// added on top of the line total when it prices exclusive.
    #[serde(default)]
    pub tax_paise: i64,
    /// [`TaxPricing::as_str`] at the moment of sale. Frozen on the row so a
    /// later country switch can never re-interpret an old bill.
    #[serde(default)]
    pub tax_pricing: String,
    /// Country wording for the tax at the moment of sale (`GST`, `VAT`, `IVA`…).
    #[serde(default)]
    pub tax_label: String,
    /// Taxable base: `total − tax` when inclusive, `lines + rounding` when
    /// exclusive. Always `total_paise − tax_paise` for a completed sale.
    #[serde(default)]
    pub net_paise: i64,
    /// House service charge added on top of the lines, already inside
    /// `total_paise`. 0 for every shop that does not charge one.
    #[serde(default)]
    pub service_paise: i64,
    /// Human bill number (`INV-MAIN-000042`) a customer can quote back. Empty
    /// on rows written before schema v13.
    #[serde(default)]
    pub invoice_no: String,
    /// Bill-type tag (`declared` = account/tax path; `owner` = cash booking/memo).
    #[serde(default)]
    pub money_lane: String,
    /// Air waybill / docket / CN number.
    #[serde(default)]
    pub awb_no: String,
    #[serde(default)]
    pub destination: String,
    #[serde(default)]
    pub consignee_name: String,
    #[serde(default)]
    pub actual_weight_kg: rust_decimal::Decimal,
    #[serde(default)]
    pub volumetric_weight_kg: rust_decimal::Decimal,
    #[serde(default)]
    pub chargeable_weight_kg: rust_decimal::Decimal,
    /// Ops status on this AWB (`booked`, `out_for_delivery`, …).
    #[serde(default)]
    pub tracking_status: String,
    #[serde(default)]
    pub zone_id: String,
    #[serde(default)]
    pub cod_paise: i64,
    #[serde(default)]
    pub carrier_cost_paise: i64,
    /// Period bill id when this account docket was invoiced; empty = unbilled.
    #[serde(default)]
    pub period_bill_id: String,
    /// `carrier` | `own_fleet` when assigned; empty = unassigned.
    #[serde(default)]
    pub handover_kind: String,
    #[serde(default)]
    pub carrier_party_id: String,
    #[serde(default)]
    pub rider_id: String,
    #[serde(default)]
    pub pod_note: String,
    #[serde(default)]
    pub extras_json: String,
    pub lines: Vec<SaleLine>,
}

impl Sale {
    pub fn margin_paise(&self) -> i64 {
        crate::cod::margin_paise(self.total_paise, self.carrier_cost_paise)
    }

    pub fn is_unbilled_account(&self) -> bool {
        self.pay_method == "credit" && self.period_bill_id.trim().is_empty()
    }

    pub fn event_times(&self) -> EventTimes {
        EventTimes {
            user_at: if self.user_at.trim().is_empty() {
                self.created_at.clone()
            } else {
                self.user_at.clone()
            },
            device_at: if self.device_at.trim().is_empty() {
                self.created_at.clone()
            } else {
                self.device_at.clone()
            },
            server_at: self.server_at.clone(),
        }
    }

    pub fn order_type(&self) -> OrderType {
        OrderType::parse(&self.order_type)
    }

    pub fn tax(&self) -> Money {
        Money(self.tax_paise)
    }

    pub fn tax_pricing(&self) -> TaxPricing {
        TaxPricing::parse(&self.tax_pricing)
    }

    /// Never empty — legacy rows written before country packs read as "Tax".
    pub fn tax_label(&self) -> &str {
        let t = self.tax_label.trim();
        if t.is_empty() { "Tax" } else { t }
    }

    /// What to print as the bill reference: the invoice number when the sale
    /// has one, else the short id older rows were identified by.
    pub fn bill_no(&self) -> String {
        let inv = self.invoice_no.trim();
        if inv.is_empty() {
            self.id[..8.min(self.id.len())].to_string()
        } else {
            inv.to_string()
        }
    }

    /// Taxable base, recomputed for rows written before schema v12.
    pub fn net(&self) -> Money {
        if self.net_paise > 0 {
            Money(self.net_paise)
        } else {
            Money(fiscal::net_of_tax(
                self.total_paise,
                self.tax_paise,
                TaxPricing::Inclusive,
            ))
        }
    }
}

/// Lightweight credit party / customer (same primitive; templates rename).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub note: String,
    pub active: bool,
    /// Outstanding balance in paise (positive = customer owes).
    pub balance_paise: i64,
    /// Freeform kind label (`Carrier`, `Retail`, …) — not a product fork.
    #[serde(default)]
    pub kind_label: String,
    /// Max outstanding for account booking (paise). 0 = no limit.
    #[serde(default)]
    pub credit_limit_paise: i64,
}

impl Customer {
    pub fn balance(&self) -> Money {
        Money(self.balance_paise)
    }

    pub fn credit_limit(&self) -> Money {
        Money(self.credit_limit_paise)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreditLedgerEntry {
    pub id: String,
    pub customer_id: String,
    pub created_at: String,
    #[serde(default)]
    pub user_at: String,
    #[serde(default)]
    pub device_at: String,
    #[serde(default)]
    pub server_at: String,
    pub kind: String,
    pub amount_paise: i64,
    pub sale_id: Option<String>,
    pub pay_method: Option<String>,
    pub actor_name: String,
    pub detail: String,
}

impl CreditLedgerEntry {
    pub fn event_times(&self) -> EventTimes {
        EventTimes {
            user_at: if self.user_at.trim().is_empty() {
                self.created_at.clone()
            } else {
                self.user_at.clone()
            },
            device_at: if self.device_at.trim().is_empty() {
                self.created_at.clone()
            } else {
                self.device_at.clone()
            },
            server_at: self.server_at.clone(),
        }
    }
}

/// Append-only history entry: previous live state before an amendment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaleRevision {
    pub id: String,
    pub sale_id: String,
    pub revised_at: String,
    pub actor_id: String,
    pub actor_name: String,
    pub reason: String,
    /// Full previous sale (including lines) as JSON-friendly struct.
    pub previous: Sale,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DayTotals {
    pub sale_count: i64,
    /// Money collected (sum of charged totals).
    pub revenue_paise: i64,
    /// Sum of line totals before rounding.
    pub lines_total_paise: i64,
    /// Net bill-level rounding (usually ≤ 0 for round-downs).
    pub rounding_paise: i64,
    pub cash_paise: i64,
    pub upi_paise: i64,
    pub card_paise: i64,
    /// Today's sales put on credit (not cash-in).
    pub credit_paise: i64,
    /// Tax already contained in `lines_total_paise` (tax-inclusive pricing).
    #[serde(default)]
    pub tax_paise: i64,
    /// Revenue tagged Account booking / Tax invoice.
    #[serde(default)]
    pub declared_paise: i64,
    /// Revenue tagged Cash booking / Cash memo.
    #[serde(default)]
    pub owner_lane_paise: i64,
}

impl DayTotals {
    /// Fold another branch/day's totals into this one (org roll-up).
    pub fn merge(&mut self, other: &Self) {
        self.sale_count += other.sale_count;
        self.revenue_paise += other.revenue_paise;
        self.lines_total_paise += other.lines_total_paise;
        self.rounding_paise += other.rounding_paise;
        self.cash_paise += other.cash_paise;
        self.upi_paise += other.upi_paise;
        self.card_paise += other.card_paise;
        self.credit_paise += other.credit_paise;
        self.tax_paise += other.tax_paise;
        self.declared_paise += other.declared_paise;
        self.owner_lane_paise += other.owner_lane_paise;
    }

    pub fn tax(&self) -> Money {
        Money(self.tax_paise)
    }
    pub fn revenue(&self) -> Money {
        Money(self.revenue_paise)
    }

    pub fn lines_total(&self) -> Money {
        Money(self.lines_total_paise)
    }

    pub fn rounding(&self) -> Money {
        Money(self.rounding_paise)
    }

    pub fn cash(&self) -> Money {
        Money(self.cash_paise)
    }

    pub fn upi(&self) -> Money {
        Money(self.upi_paise)
    }

    pub fn card(&self) -> Money {
        Money(self.card_paise)
    }

    pub fn credit(&self) -> Money {
        Money(self.credit_paise)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableStatus {
    Available,
    Occupied,
}

impl TableStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Occupied => "occupied",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "occupied" => Self::Occupied,
            _ => Self::Available,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiningTable {
    pub id: String,
    pub name: String,
    pub status: TableStatus,
    pub active_sale_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KotStatus {
    New,
    Preparing,
    Ready,
    Completed,
    Cancelled,
}

impl KotStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Preparing => "preparing",
            Self::Ready => "ready",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "preparing" => Self::Preparing,
            "ready" => Self::Ready,
            "completed" => Self::Completed,
            "cancelled" => Self::Cancelled,
            _ => Self::New,
        }
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Self::New => Some(Self::Preparing),
            Self::Preparing => Some(Self::Ready),
            Self::Ready => Some(Self::Completed),
            Self::Completed | Self::Cancelled => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KotTicket {
    pub id: String,
    pub sale_id: String,
    pub created_at: String,
    pub status: KotStatus,
    pub summary: String,
    /// Kitchen station this ticket belongs to; empty = whole order / unassigned.
    #[serde(default)]
    pub station_id: String,
    /// Station display name resolved for the KDS (empty when unassigned).
    #[serde(default)]
    pub station_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub created_at: String,
    pub actor_id: String,
    pub actor_name: String,
    pub action: String,
    pub detail: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OpsReport {
    pub day: DayTotals,
    pub outstanding_credit_paise: i64,
    pub expenses_paise: i64,
    pub expense_count: i64,
    pub top_items: Vec<(String, i64, i64)>, // name, qty, revenue_paise
    /// Till→bank and other bank posts for the same day (when available).
    #[serde(default)]
    pub bank_net_paise: i64,
}

/// One append-only change to on-hand stock (sale, purchase, wastage, count).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StockMovement {
    pub id: String,
    pub branch_id: String,
    pub product_id: String,
    /// Resolved item name for display (filled by the store when listing).
    #[serde(default)]
    pub product_name: String,
    pub created_at: String,
    #[serde(default)]
    pub user_at: String,
    #[serde(default)]
    pub device_at: String,
    #[serde(default)]
    pub server_at: String,
    /// Negative for sales/wastage, positive for purchases/corrections.
    pub delta_qty: i64,
    /// `sale` | `purchase` | `adjust` | `wastage` | free text.
    pub reason: String,
    pub note: String,
    pub actor_id: String,
    pub actor_name: String,
    /// Sale / purchase id this movement came from (empty for manual adjustments).
    pub ref_id: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PurchaseLine {
    pub id: String,
    pub purchase_id: String,
    pub product_id: String,
    pub name: String,
    pub qty: i64,
    pub unit_cost_paise: i64,
    /// Milli-units (1000 = 1). 0 on legacy rows means `qty × 1000`.
    #[serde(default)]
    pub qty_milli: i64,
    #[serde(default)]
    pub unit_label: String,
}

impl PurchaseLine {
    pub fn effective_qty_milli(&self) -> i64 {
        if self.qty_milli > 0 {
            self.qty_milli
        } else {
            self.qty.saturating_mul(QTY_MILLI_UNIT)
        }
    }

    /// Unit cost is per whole unit (per kg / per piece); scale by milli qty.
    pub fn line_cost_paise(&self) -> i64 {
        let milli = self.effective_qty_milli().max(0);
        (self.unit_cost_paise.saturating_mul(milli)) / QTY_MILLI_UNIT
    }
}

/// Goods received from a supplier — raises stock and records what it cost.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Purchase {
    pub id: String,
    pub branch_id: String,
    pub created_at: String,
    #[serde(default)]
    pub user_at: String,
    #[serde(default)]
    pub device_at: String,
    #[serde(default)]
    pub server_at: String,
    pub supplier: String,
    /// Party id when supplier is a party (AP path); empty = text-only legacy.
    #[serde(default)]
    pub party_id: String,
    /// When true, total posts to payable ledger (we owe the party).
    #[serde(default)]
    pub on_account: bool,
    pub total_paise: i64,
    pub note: String,
    pub actor_id: String,
    pub actor_name: String,
    #[serde(default)]
    pub lines: Vec<PurchaseLine>,
}

impl Purchase {
    pub fn total(&self) -> Money {
        Money(self.total_paise)
    }
}

/// One branch's day exported as a file/QR payload so branches can roll up
/// without any cloud hosting. Imported on the owner's device.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BranchSnapshot {
    pub branch_id: String,
    pub branch_name: String,
    pub branch_code: String,
    pub org_id: String,
    pub org_name: String,
    pub exported_at: String,
    /// Calendar day the totals cover (`%Y-%m-%d`).
    pub day: String,
    pub totals: DayTotals,
    pub outstanding_credit_paise: i64,
    pub expenses_paise: i64,
    pub expense_count: i64,
    /// name, qty, revenue_paise
    pub top_items: Vec<(String, i64, i64)>,
}

impl BranchSnapshot {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".into())
    }

    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("Not a branch snapshot: {e}"))
    }
}

/// Owner roll-up across branches: this device live + imported snapshots.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrgConsolidatedReport {
    pub org_name: String,
    /// branch name → that branch's day totals.
    pub branches: Vec<(String, DayTotals)>,
    pub totals: DayTotals,
    pub outstanding_credit_paise: i64,
    pub expenses_paise: i64,
    /// Branches whose snapshot is not from today — shown as "last known".
    pub stale_branches: Vec<String>,
    /// Branches with no snapshot imported yet.
    pub missing_branches: Vec<String>,
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fiscal::FiscalSettings;

    fn tea() -> Product {
        Product::new_basic("tea", "Tea", Money::from_rupees(13))
    }

    #[test]
    fn rounding_keeps_true_line_prices() {
        let mut cart = Cart::default();
        let p = tea();
        for _ in 0..4 {
            cart.add_or_bump(&p);
        }
        assert_eq!(cart.line_sum().paise(), 5200);
        cart.set_charge_total(Money::from_rupees(50)).unwrap();
        assert_eq!(cart.rounding().paise(), -200);
        let draft = cart.to_sale_draft().unwrap();
        draft.assert_balanced().unwrap();
        assert_eq!(draft.lines[0].unit_price_paise, 1300);
        assert_eq!(draft.lines[0].line_total_paise, 5200);
        assert_eq!(draft.lines_total_paise, 5200);
        assert_eq!(draft.rounding_paise, -200);
        assert_eq!(draft.total_paise, 5000);
    }

    #[test]
    fn unit_price_edit_clears_stale_rounding() {
        let mut cart = Cart::default();
        let p = tea();
        cart.add_or_bump(&p);
        cart.set_charge_total(Money::from_rupees(10)).unwrap();
        assert!(cart.charge_total.is_some());
        cart.set_unit_price(0, Money::from_rupees(12)).unwrap();
        assert!(cart.charge_total.is_none());
        assert_eq!(cart.total().paise(), 1200);
    }

    #[test]
    fn cart_line_note_roundtrips_into_draft() {
        let mut cart = Cart::default();
        let p = tea();
        cart.add_or_bump(&p);
        cart.set_note(0, "  less sugar  ").unwrap();
        let draft = cart.to_sale_draft().unwrap();
        assert_eq!(draft.lines[0].note, "less sugar");
    }

    #[test]
    fn line_total_edit_keeps_unit_price() {
        let mut cart = Cart::default();
        let p = tea();
        for _ in 0..4 {
            cart.add_or_bump(&p);
        }
        assert_eq!(cart.lines[0].actual_total().paise(), 5200);
        assert_eq!(cart.lines[0].applied_total().paise(), 5200);
        // Round this line's Applied Total to ₹50 — @ stays ₹13.
        cart.set_line_total(0, Money::from_rupees(50)).unwrap();
        assert_eq!(cart.lines[0].unit_price.paise(), 1300);
        assert_eq!(cart.lines[0].actual_total().paise(), 5200);
        assert_eq!(cart.lines[0].applied_total().paise(), 5000);
        assert_eq!(cart.lines[0].rounding().paise(), -200);
        assert_eq!(cart.actual_total().paise(), 5200);
        assert_eq!(cart.line_sum().paise(), 5000);
        assert_eq!(cart.total().paise(), 5000);
        let draft = cart.to_sale_draft().unwrap();
        assert_eq!(draft.lines[0].unit_price_paise, 1300);
        assert_eq!(draft.lines[0].qty, 4);
        assert_eq!(draft.lines[0].line_total_paise, 5000);
        assert_eq!(draft.lines_total_paise, 5000);
        assert_eq!(draft.rounding_paise, 0);
        assert_eq!(draft.total_paise, 5000);
        // Setting Applied back to Actual clears the override.
        cart.set_line_total(0, Money::from_rupees(52)).unwrap();
        assert!(cart.lines[0].line_total_override.is_none());
        assert_eq!(cart.lines[0].unit_price.paise(), 1300);
    }

    #[test]
    fn charge_defaults_to_natural_and_clears_when_equal() {
        let mut cart = Cart::default();
        let p = tea();
        for _ in 0..4 {
            cart.add_or_bump(&p);
        }
        assert_eq!(cart.actual_total().paise(), 5200);
        assert_eq!(cart.total().paise(), 5200);
        assert!(cart.charge_total.is_none());
        cart.set_charge_total(Money::from_rupees(50)).unwrap();
        assert_eq!(cart.total().paise(), 5000);
        assert!(cart.charge_total.is_some());
        cart.set_charge_total(Money::from_rupees(52)).unwrap();
        assert!(cart.charge_total.is_none());
        assert_eq!(cart.total().paise(), 5200);
    }

    #[test]
    fn rounded_follows_applied_across_multiple_products() {
        let mut cart = Cart::default();
        let tea = tea();
        let samosa = Product::new_basic("samosa", "Samosa", Money::from_rupees(20));
        cart.add_or_bump(&tea);
        assert_eq!(cart.applied_total().paise(), 1300);
        assert_eq!(cart.rounded_total().paise(), 1300);
        cart.set_charge_total(Money::from_rupees(10)).unwrap();
        assert_eq!(cart.rounded_total().paise(), 1000);
        // Adding another line must clear the stale bill round.
        cart.add_or_bump(&samosa);
        assert!(cart.charge_total.is_none());
        assert_eq!(cart.applied_total().paise(), 3300);
        assert_eq!(cart.rounded_total().paise(), 3300);
        cart.add_or_bump(&tea);
        assert_eq!(cart.applied_total().paise(), 4600);
        assert_eq!(cart.rounded_total().paise(), 4600);
    }

    #[test]
    fn order_type_and_modifiers_roundtrip() {
        assert_eq!(OrderType::parse("dine_in"), OrderType::DineIn);
        assert_eq!(OrderType::parse("garbage"), OrderType::Counter);
        assert_eq!(OrderType::DineIn.as_str(), "dine_in");
        let mods = vec![
            ModifierOption {
                name: "Extra cheese".into(),
                price_paise: 2000,
            },
            ModifierOption {
                name: "No onion".into(),
                price_paise: 0,
            },
        ];
        let mut p = tea();
        p.set_modifiers(&mods);
        assert_eq!(p.modifiers(), mods);
        // Bad JSON must not break the menu.
        p.modifiers_json = "not json".into();
        assert!(p.modifiers().is_empty());
    }

    #[test]
    fn modifier_toggle_changes_unit_price_and_kitchen_note() {
        let mut cart = Cart::default();
        let mut tea = tea();
        let cheese = ModifierOption {
            name: "Extra cheese".into(),
            price_paise: 2_000,
        };
        tea.set_modifiers(std::slice::from_ref(&cheese));
        cart.add_or_bump(&tea);
        assert!(cart.toggle_modifier(0, &cheese).unwrap());
        assert_eq!(cart.lines[0].unit_price.paise(), 1_300 + 2_000);
        let draft = cart.to_sale_draft().unwrap();
        assert_eq!(draft.lines[0].note, "Extra cheese");
        assert_eq!(draft.lines[0].unit_price_paise, 3_300);
        cart.set_note(0, "no onion").unwrap();
        assert_eq!(cart.to_sale_draft().unwrap().lines[0].note, "Extra cheese, no onion");
        assert!(!cart.toggle_modifier(0, &cheese).unwrap());
        assert_eq!(cart.lines[0].unit_price.paise(), 1_300);
        assert_eq!(cart.to_sale_draft().unwrap().lines[0].note, "no onion");
    }

    #[test]
    fn inclusive_tax_never_inflates_the_bill() {
        // ₹105 at 5% inclusive → ₹5 tax inside, bill unchanged.
        assert_eq!(fiscal::tax_on_amount(10_500, 500, TaxPricing::Inclusive), 500);
        assert_eq!(fiscal::tax_on_amount(10_500, 0, TaxPricing::Inclusive), 0);
        let mut cart = Cart::default();
        let mut p = tea();
        p.tax_bps = 500;
        cart.add_or_bump(&p);
        cart.set_order_type(OrderType::Takeaway);
        let draft = cart.to_sale_draft().unwrap();
        draft.assert_balanced().unwrap();
        assert_eq!(draft.tax_paise, 0, "store fills tax at finalize");
        assert_eq!(draft.order_type, OrderType::Takeaway);
        assert_eq!(draft.total_paise, 1300);
        assert_eq!(p.tax_for(1300, TaxPricing::Inclusive), 61);
    }

    #[test]
    fn exclusive_tax_adds_on_top_of_a_balanced_draft() {
        // $10 net at 5% exclusive → $0.50 added at the till. The cart stays
        // tax-agnostic, so the draft still balances at the quoted $10.
        let mut cart = Cart::default();
        let mut p = Product::new_basic("burger", "Burger", Money(1_000));
        p.tax_bps = 500;
        cart.add_or_bump(&p);
        let draft = cart.to_sale_draft().unwrap();
        draft.assert_balanced().unwrap();
        assert_eq!(draft.total_paise, 1_000);
        assert_eq!(draft.tax_paise, 0);

        let tax = p.tax_for(draft.lines_total_paise, TaxPricing::Exclusive);
        assert_eq!(tax, 50);
        assert_eq!(
            fiscal::gross_with_tax(draft.total_paise, tax, TaxPricing::Exclusive),
            1_050
        );
        assert_eq!(
            fiscal::net_of_tax(draft.total_paise, tax, TaxPricing::Exclusive),
            1_000
        );
    }

    #[test]
    fn pay_labels_follow_the_country_pack() {
        let mut f = FiscalSettings::default();
        assert_eq!(PayMethod::Upi.label_for(&f), "UPI");
        assert_eq!(PayMethod::Cash.label_for(&f), "Cash");
        f = FiscalSettings::from_profile(
            fiscal::country_by_code("BR").unwrap(),
            String::new(),
        );
        assert_eq!(PayMethod::Upi.label_for(&f), "Pix");
        assert_eq!(PayMethod::Split.label_for(&f), "Split Cash+Pix");
        assert_eq!(PayMethod::Credit.label_for(&f), "Fiado");
        // Storage is untouched, so the rail still round-trips as `upi`.
        assert_eq!(PayMethod::parse(PayMethod::Upi.as_str()), Some(PayMethod::Upi));
    }

    #[test]
    fn tax_rate_labels_and_matching() {
        let rate = TaxRate {
            id: "r1".into(),
            name: "VAT 13.5%".into(),
            code: "13.5".into(),
            bps: 1_350,
            active: true,
        };
        assert_eq!(rate.percent_label(), "13.5%");
        assert!(rate.matches("vat 13.5%"));
        assert!(rate.matches("13.5"));
        assert!(!rate.matches("20"));
        let flat = TaxRate {
            bps: 1_800,
            code: "18".into(),
            ..TaxRate::default()
        };
        assert_eq!(flat.percent_label(), "18%");
    }

    #[test]
    fn event_times_three_clocks() {
        let t = EventTimes::capture(None);
        assert!(!t.user_at.is_empty());
        assert_eq!(t.user_at, t.device_at);
        assert!(t.server_at.is_empty());
        let backdated = EventTimes::capture(Some("2024-01-15"));
        assert!(backdated.user_at.starts_with("2024-01-15T"));
        assert!(!backdated.device_at.starts_with("2024-01-15T") || backdated.device_at.contains('T'));
        assert_ne!(backdated.user_at, backdated.device_at);
        let stamped = backdated.with_server_now();
        assert!(!stamped.server_at.is_empty());
        let legacy = EventTimes::from_legacy_created("2023-06-01T10:00:00+05:30");
        assert_eq!(legacy.user_at, legacy.device_at);
        assert!(legacy.server_at.is_empty());
    }
}


