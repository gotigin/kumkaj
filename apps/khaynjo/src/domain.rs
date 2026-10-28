//! Core Kumkaj domain: integer money, catalog, cart, sales, staff, tables/KDS.

use crate::fiscal::{self, FiscalSettings, TaxPricing};
use chrono::{Local, Timelike};
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
    /// Bank transfer / cheque — money that moves through the bank
    /// account rather than the till or a card terminal.
    Bank,
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
            Self::Bank => "bank",
            Self::Credit => "credit",
            Self::Split => "split",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cash" => Some(Self::Cash),
            "upi" => Some(Self::Upi),
            "card" => Some(Self::Card),
            "bank" | "cheque" | "check" | "transfer" => Some(Self::Bank),
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
            Self::Bank => "Bank",
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
    Delivery,
}

impl OrderType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::DineIn => "dine_in",
            Self::Takeaway => "takeaway",
            Self::Delivery => "delivery",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "dine_in" | "dinein" | "dine-in" => Self::DineIn,
            "takeaway" | "take_away" | "parcel" => Self::Takeaway,
            "delivery" | "home_delivery" => Self::Delivery,
            _ => Self::Counter,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Counter => "Counter",
            Self::DineIn => "Dine-in",
            Self::Takeaway => "Takeaway",
            Self::Delivery => "Delivery",
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
        // Cashier and Waiter also bump tickets: one-person shops and
        // captains often clear "In kitchen" without a Kitchen login.
        matches!(
            self,
            Self::Owner | Self::Manager | Self::Kitchen | Self::Cashier | Self::Waiter
        )
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
    /// Owner-edited rights; `None` = the role's preset ([`StaffRights::for_role`]).
    #[serde(default)]
    pub rights: Option<StaffRights>,
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
            rights: None,
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
                rights: None,
            },
            StaffRole::Cashier => Self {
                sell: true,
                items: false,
                sales: false,
                credit: true,
                floor: true,
                // Same device often is the kitchen screen — they must be
                // able to clear "In kitchen" without a Kitchen login.
                kitchen: true,
                team: false,
                backup: false,
                inventory: false,
                branches: false,
                rights: None,
            },
            // A waiter is a cashier who also works the floor.
            StaffRole::Waiter => Self {
                sell: true,
                items: false,
                sales: false,
                credit: true,
                floor: true,
                // Captains bump Ready → Done when they serve; cooking Start
                // still works if the shop has no separate kitchen tablet.
                kitchen: true,
                team: false,
                backup: false,
                inventory: false,
                branches: false,
                rights: None,
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
                rights: None,
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

/// One open bill for bill-wise (open-item) allocation: a credit sale
/// (AR) or an on-account purchase (AP) and how much of it is still
/// open after settlements linked to it plus the FIFO share of
/// unlinked settlements.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenBill {
    /// Sale id (AR) or purchase id (AP).
    pub bill_id: String,
    pub created_at: String,
    /// Net amount billed on credit for this bill.
    pub billed_paise: i64,
    pub paid_paise: i64,
    pub open_paise: i64,
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
    /// Bank reconciliation: the owner has seen this movement on the
    /// bank statement and ticked it off (BRS). Uncleared movements are
    /// the difference between the book balance and the bank's balance.
    #[serde(default)]
    pub cleared: bool,
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

/// Today's books figures, pre-formatted for the Books home stat cards.
#[derive(Clone, Debug, Default)]
pub struct BooksOverview {
    pub money_in: String,
    pub purchases: String,
    pub expenses: String,
    pub gross: String,
    pub net: String,
    pub lane_declared_label: String,
    pub lane_declared: String,
    pub lane_owner_label: String,
    pub lane_owner: String,
    pub cash: String,
    pub digital_label: String,
    pub digital: String,
    pub card: String,
    pub they_owe: String,
    pub we_owe: String,
    pub bank_balance: String,
    pub bank_net_today: String,
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: String,
    pub code: String,
    pub name: String,
    pub builtin: bool,
    /// Shown in Buy / Menu / conversion pickers when true. Disabled units
    /// stay for conversions but leave day-to-day dropdowns.
    #[serde(default = "unit_picker_default_on")]
    pub picker_enabled: bool,
}

fn unit_picker_default_on() -> bool {
    true
}

impl Default for UnitDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            code: String::new(),
            name: String::new(),
            builtin: false,
            picker_enabled: true,
        }
    }
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

/// Exact standard conversion factors into the canonical bases (cm, kg).
/// These win for the standard codes; a shop defines entirely custom
/// units (e.g. `crate`) with its own conversion to cm or kg. Returns
/// `None` for unknown codes.
pub fn standard_factor_to_cm(code: &str) -> Option<rust_decimal::Decimal> {
    use rust_decimal::Decimal;
    Some(match code.trim().to_ascii_lowercase().as_str() {
        "cm" => Decimal::ONE,
        "mm" => Decimal::new(1, 1),
        "m" => Decimal::from(100),
        "in" => Decimal::new(254, 2),
        "ft" => Decimal::new(3_048, 2),
        _ => return None,
    })
}

/// Unit families (dimensions) and their one canonical anchor unit.
/// Every unit — built-in or custom — only means something because it
/// resolves to its family anchor through a chain of conversions; that
/// is what keeps quantities comparable in books, reports and P&L.
/// Returns `(family, factor converting 1 code into the anchor)`.
/// Physical factors are fixed constants of nature — unlike currency
/// rates, which are prices at a moment in time and are never units.
pub fn standard_unit_family(
    code: &str,
) -> Option<(&'static str, rust_decimal::Decimal)> {
    use rust_decimal::Decimal;
    let code = code.trim().to_ascii_lowercase();
    if let Some(f) = standard_factor_to_kg(&code) {
        return Some(("weight", f));
    }
    if let Some(f) = standard_factor_to_cm(&code) {
        return Some(("length", f));
    }
    Some(match code.as_str() {
        "ml" => ("volume", Decimal::ONE),
        "l" => ("volume", Decimal::from(1000)),
        "pc" => ("count", Decimal::ONE),
        "dozen" => ("count", Decimal::from(12)),
        _ => return None,
    })
}

/// The canonical anchor unit a family's quantities are stored and
/// compared in (weight → kg, length → cm, volume → ml, count → pc).
pub fn unit_family_anchor(family: &str) -> Option<&'static str> {
    Some(match family {
        "weight" => "kg",
        "length" => "cm",
        "volume" => "ml",
        "count" => "pc",
        _ => return None,
    })
}

/// See [`standard_factor_to_cm`].
pub fn standard_factor_to_kg(code: &str) -> Option<rust_decimal::Decimal> {
    use rust_decimal::Decimal;
    Some(match code.trim().to_ascii_lowercase().as_str() {
        "kg" => Decimal::ONE,
        "g" => Decimal::new(1, 3),
        "t" => Decimal::from(1000),
        "lb" => Decimal::new(45_359_237, 8),
        "oz" => Decimal::new(28_349_523_125, 12),
        _ => return None,
    })
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

    /// What this person may do with money and sent work. The Owner can do
    /// everything; others get what the owner set, else their role preset.
    pub fn rights(&self) -> StaffRights {
        if self.is_owner() {
            return StaffRights::all_allowed();
        }
        self.features
            .rights
            .clone()
            .unwrap_or_else(|| StaffRights::for_role(self.role))
    }
}

/// How far one person may go with one sensitive action (vision §17).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RightLevel {
    /// May do it.
    Allowed,
    /// May start it; an Owner/Manager PIN is entered at that moment and the
    /// approver is recorded with the action.
    WithApproval,
    /// Refused with a plain reason.
    #[default]
    Denied,
}

impl RightLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::WithApproval => "with_approval",
            Self::Denied => "denied",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "allowed" => Self::Allowed,
            "with_approval" => Self::WithApproval,
            _ => Self::Denied,
        }
    }

    /// Allowed → With approval → Not allowed → Allowed (a tap cycles it).
    pub fn cycle(self) -> Self {
        match self {
            Self::Allowed => Self::WithApproval,
            Self::WithApproval => Self::Denied,
            Self::Denied => Self::Allowed,
        }
    }
}

/// The sensitive restaurant actions the owner grants per person — the
/// Petpooja-style rights matrix, enforced in [`crate::store::Store`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Right {
    /// Take payment and close a bill (Settle & Save).
    Settle,
    /// Cancel a line (or part of it) after its KOT went to the kitchen.
    CancelSentItem,
    /// Make a line or a whole bill complimentary (NC, ₹0).
    Complimentary,
    /// Move items or a whole table, or merge tables.
    MoveTable,
    /// Print the bill again, or a KOT again.
    Reprint,
    /// Take money off a bill (percent or amount).
    Discount,
}

impl Right {
    pub const ALL: [Right; 6] = [
        Right::Settle,
        Right::CancelSentItem,
        Right::Complimentary,
        Right::MoveTable,
        Right::Reprint,
        Right::Discount,
    ];

    /// Locale key of the right's name.
    pub fn label_key(self) -> &'static str {
        match self {
            Right::Settle => "right_settle",
            Right::CancelSentItem => "right_cancel_sent",
            Right::Complimentary => "right_complimentary",
            Right::MoveTable => "right_move_table",
            Right::Reprint => "right_reprint",
            Right::Discount => "right_discount",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaffRights {
    #[serde(default)]
    pub settle: RightLevel,
    #[serde(default)]
    pub cancel_sent_item: RightLevel,
    #[serde(default)]
    pub complimentary: RightLevel,
    #[serde(default)]
    pub move_table: RightLevel,
    #[serde(default)]
    pub reprint: RightLevel,
    #[serde(default)]
    pub discount: RightLevel,
}

impl StaffRights {
    pub fn all_allowed() -> Self {
        Self {
            settle: RightLevel::Allowed,
            cancel_sent_item: RightLevel::Allowed,
            complimentary: RightLevel::Allowed,
            move_table: RightLevel::Allowed,
            reprint: RightLevel::Allowed,
            discount: RightLevel::Allowed,
        }
    }

    /// Restaurant defaults (Petpooja-like): the counter settles; a waiter
    /// takes orders and moves guests but never settles money; giving food
    /// away or cancelling what the kitchen already has needs a manager.
    pub fn for_role(role: StaffRole) -> Self {
        use RightLevel::*;
        match role {
            StaffRole::Owner | StaffRole::Manager => Self::all_allowed(),
            StaffRole::Cashier => Self {
                settle: Allowed,
                cancel_sent_item: WithApproval,
                complimentary: WithApproval,
                move_table: Allowed,
                reprint: Allowed,
                discount: WithApproval,
            },
            StaffRole::Waiter => Self {
                settle: Denied,
                cancel_sent_item: WithApproval,
                complimentary: Denied,
                move_table: Allowed,
                reprint: Denied,
                discount: Denied,
            },
            StaffRole::Kitchen => Self {
                settle: Denied,
                cancel_sent_item: Denied,
                complimentary: Denied,
                move_table: Denied,
                reprint: Denied,
                discount: Denied,
            },
        }
    }

    pub fn level(&self, right: Right) -> RightLevel {
        match right {
            Right::Settle => self.settle,
            Right::CancelSentItem => self.cancel_sent_item,
            Right::Complimentary => self.complimentary,
            Right::MoveTable => self.move_table,
            Right::Reprint => self.reprint,
            Right::Discount => self.discount,
        }
    }

    pub fn set_level(&mut self, right: Right, level: RightLevel) {
        match right {
            Right::Settle => self.settle = level,
            Right::CancelSentItem => self.cancel_sent_item = level,
            Right::Complimentary => self.complimentary = level,
            Right::MoveTable => self.move_table = level,
            Right::Reprint => self.reprint = level,
            Right::Discount => self.discount = level,
        }
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
    /// Owner-picked photo key (`custom:<file>`); empty = initials placeholder.
    #[serde(default)]
    pub image_key: String,
}

/// Kitchen station a line is routed to (Hot / Cold / Tandoor …).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KitchenStation {
    pub id: String,
    pub branch_id: String,
    pub name: String,
    pub sort_order: i64,
    /// Target prep minutes for KDS ageing (amber at ~⅔, red at ~⁴⁄₃). Default 15.
    #[serde(default = "default_target_prep_mins")]
    pub target_prep_mins: i64,
    /// Shown in station pickers when true (like units).
    #[serde(default = "unit_picker_default_on")]
    pub picker_enabled: bool,
}

fn default_target_prep_mins() -> i64 {
    15
}

/// KDS ticket age band for colouring (normal / amber / red).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KotAgeBand {
    Normal,
    Amber,
    Red,
}

/// Age colour from minutes since `created_at` and the station's target.
/// With the default target of 15: normal &lt;10, amber 10–20, red &gt;20.
pub fn kot_age_band(minutes: i64, target_prep_mins: i64) -> KotAgeBand {
    let target = target_prep_mins.max(1);
    let amber_at = (target * 2 / 3).max(1);
    let red_at = (target * 4 / 3).max(amber_at + 1);
    if minutes < amber_at {
        KotAgeBand::Normal
    } else if minutes < red_at {
        KotAgeBand::Amber
    } else {
        KotAgeBand::Red
    }
}

/// One paid choice for an item (`Extra cheese +₹20`, or `Large` under Size).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifierOption {
    pub name: String,
    #[serde(default)]
    pub price_paise: i64,
    /// Catalogue group this option belongs to (`Size`, `Toppings`). Empty on
    /// legacy flat lists / when the group is the default "Options".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group: String,
}

/// How a [`ModifierGroup`] behaves at the till (Petpooja-style).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModifierGroupKind {
    /// Pick one (size, crust). Price adds to the base item price.
    Variation,
    /// Pick any that apply (toppings). Each adds its own price.
    #[default]
    Addon,
}

impl ModifierGroupKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Variation => "variation",
            Self::Addon => "addon",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "variation" | "var" | "size" => Self::Variation,
            _ => Self::Addon,
        }
    }
}

/// Named set of choices on one dish — e.g. Size (variation) or Toppings (add-ons).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifierGroup {
    pub name: String,
    #[serde(default)]
    pub kind: ModifierGroupKind,
    /// Minimum selections (variation defaults to 1; add-on to 0).
    #[serde(default)]
    pub min: u32,
    /// Maximum selections (`0` = no cap). Variation is always capped at 1.
    #[serde(default)]
    pub max: u32,
    pub options: Vec<ModifierOption>,
}

impl Default for ModifierGroup {
    fn default() -> Self {
        Self {
            name: "Options".into(),
            kind: ModifierGroupKind::Addon,
            min: 0,
            max: 0,
            options: Vec::new(),
        }
    }
}

impl ModifierGroup {
    pub fn is_variation(&self) -> bool {
        self.kind == ModifierGroupKind::Variation
    }

    pub fn max_allowed(&self) -> usize {
        if self.is_variation() {
            1
        } else if self.max == 0 {
            usize::MAX
        } else {
            self.max as usize
        }
    }

    pub fn min_required(&self) -> usize {
        if self.is_variation() {
            self.min.max(1) as usize
        } else {
            self.min as usize
        }
    }
}

impl ModifierOption {
    /// Parse a `modifiers_json` column as a flat option list (legacy). Prefer
    /// [`ModifierGroup::parse_list`] / [`Product::modifier_groups`].
    pub fn parse_list(json: &str) -> Vec<Self> {
        ModifierGroup::parse_list(json)
            .into_iter()
            .flat_map(|g| {
                let group = g.name.clone();
                g.options.into_iter().map(move |mut o| {
                    if o.group.is_empty() {
                        o.group = group.clone();
                    }
                    o
                })
            })
            .collect()
    }

    pub fn serialize_list(list: &[Self]) -> String {
        // Keep writing the compact legacy shape when nothing is grouped.
        if list.iter().all(|o| o.group.is_empty() || o.group == "Options") {
            let flat: Vec<Self> = list
                .iter()
                .map(|o| Self {
                    name: o.name.clone(),
                    price_paise: o.price_paise,
                    group: String::new(),
                })
                .collect();
            return serde_json::to_string(&flat).unwrap_or_else(|_| "[]".into());
        }
        let groups = ModifierGroup::from_flat_options(list);
        ModifierGroup::serialize_list(&groups)
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
                group: String::new(),
            });
        }
        Ok(out)
    }
}

impl ModifierGroup {
    /// Parse `modifiers_json`: either a group array or a legacy flat option list.
    pub fn parse_list(json: &str) -> Vec<Self> {
        let s = json.trim();
        if s.is_empty() {
            return Vec::new();
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(s) else {
            return Vec::new();
        };
        let serde_json::Value::Array(arr) = value else {
            return Vec::new();
        };
        if arr.is_empty() {
            return Vec::new();
        }
        if arr
            .first()
            .and_then(|v| v.get("options"))
            .map(|o| o.is_array())
            .unwrap_or(false)
        {
            return serde_json::from_value(serde_json::Value::Array(arr)).unwrap_or_default();
        }
        let opts: Vec<ModifierOption> =
            serde_json::from_value(serde_json::Value::Array(arr)).unwrap_or_default();
        if opts.is_empty() {
            Vec::new()
        } else {
            vec![Self {
                name: "Options".into(),
                kind: ModifierGroupKind::Addon,
                min: 0,
                max: 0,
                options: opts,
            }]
        }
    }

    pub fn serialize_list(list: &[Self]) -> String {
        if list.is_empty() {
            return "[]".into();
        }
        // One unnamed Options add-on group → legacy flat array (smaller, compatible).
        if list.len() == 1
            && list[0].kind == ModifierGroupKind::Addon
            && (list[0].name.is_empty() || list[0].name == "Options")
            && list[0].min == 0
            && list[0].max == 0
        {
            let flat: Vec<ModifierOption> = list[0]
                .options
                .iter()
                .map(|o| ModifierOption {
                    name: o.name.clone(),
                    price_paise: o.price_paise,
                    group: String::new(),
                })
                .collect();
            return serde_json::to_string(&flat).unwrap_or_else(|_| "[]".into());
        }
        serde_json::to_string(list).unwrap_or_else(|_| "[]".into())
    }

    pub(crate) fn from_flat_options(list: &[ModifierOption]) -> Vec<Self> {
        let mut order: Vec<String> = Vec::new();
        let mut by: std::collections::HashMap<String, Vec<ModifierOption>> =
            std::collections::HashMap::new();
        for o in list {
            let g = if o.group.trim().is_empty() {
                "Options".to_string()
            } else {
                o.group.trim().to_string()
            };
            if !order.iter().any(|n| n == &g) {
                order.push(g.clone());
            }
            by.entry(g).or_default().push(ModifierOption {
                name: o.name.clone(),
                price_paise: o.price_paise,
                group: String::new(),
            });
        }
        order
            .into_iter()
            .filter_map(|name| {
                let options = by.remove(&name)?;
                let variation = name != "Options";
                Some(Self {
                    kind: if variation {
                        ModifierGroupKind::Variation
                    } else {
                        ModifierGroupKind::Addon
                    },
                    min: if variation { 1 } else { 0 },
                    max: if variation { 1 } else { 0 },
                    name,
                    options,
                })
            })
            .collect()
    }

    /// Human form: `Size*: Small=199, Medium=299 | Toppings+: Mushrooms=40 | Extra cheese=20`.
    ///
    /// - `Name*:` — required variation (pick one)
    /// - `Name+:` — add-ons (pick any)
    /// - `Name:` — variation (pick one), same as `*`
    /// - bare `a=1, b=2` — add-ons under Options
    pub fn parse_human_list(text: &str, currency_code: &str) -> Result<Vec<Self>, String> {
        let text = text.trim();
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let mut groups: Vec<Self> = Vec::new();
        for seg in text.split('|') {
            let seg = seg.trim();
            if seg.is_empty() {
                continue;
            }
            let (header, body) = match seg.split_once(':') {
                Some((h, rest)) if !h.contains('=') => (Some(h.trim()), rest.trim()),
                _ => (None, seg),
            };
            let options = ModifierOption::parse_human_list(body, currency_code)?;
            if options.is_empty() {
                continue;
            }
            match header {
                None => {
                    if let Some(existing) = groups
                        .iter_mut()
                        .find(|g| g.name == "Options" && g.kind == ModifierGroupKind::Addon)
                    {
                        existing.options.extend(options);
                    } else {
                        groups.push(Self {
                            name: "Options".into(),
                            kind: ModifierGroupKind::Addon,
                            min: 0,
                            max: 0,
                            options,
                        });
                    }
                }
                Some(raw) => {
                    let (name, kind, min, max) = parse_group_header(raw)?;
                    if name.is_empty() {
                        return Err(
                            "Option group needs a name (e.g. Size*: Small=199 or Toppings+3: …)"
                                .into(),
                        );
                    }
                    groups.push(Self {
                        name: name.to_string(),
                        kind,
                        min,
                        max,
                        options,
                    });
                }
            }
        }
        Ok(groups)
    }

    /// Inverse of [`Self::parse_human_list`] for the menu form.
    pub fn format_human_list(groups: &[Self], currency_code: &str) -> String {
        groups
            .iter()
            .map(|g| {
                let opts = g
                    .options
                    .iter()
                    .map(|o| {
                        let price = crate::locale::format_major_input(o.price_paise, currency_code);
                        if o.price_paise == 0 {
                            o.name.clone()
                        } else {
                            format!("{}={price}", o.name)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if g.name == "Options" && g.kind == ModifierGroupKind::Addon && g.max == 0 {
                    opts
                } else {
                    let mark = match g.kind {
                        ModifierGroupKind::Variation => "*".to_string(),
                        ModifierGroupKind::Addon if g.max > 0 => format!("+{}", g.max),
                        ModifierGroupKind::Addon => "+".to_string(),
                    };
                    format!("{}{mark}: {opts}", g.name)
                }
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

/// `Size*` → variation; `Toppings+` / `Toppings+3` → add-on (optional max).
fn parse_group_header(raw: &str) -> Result<(String, ModifierGroupKind, u32, u32), String> {
    let raw = raw.trim();
    if let Some(n) = raw.strip_suffix('*') {
        let name = n.trim().trim_end_matches('+').trim();
        return Ok((name.to_string(), ModifierGroupKind::Variation, 1, 1));
    }
    if let Some(idx) = raw.rfind('+') {
        let name = raw[..idx].trim();
        let after = raw[idx + 1..].trim();
        let max = if after.is_empty() {
            0u32
        } else {
            after
                .parse::<u32>()
                .map_err(|_| format!("Bad max after + in '{raw}' — use Toppings+3:"))?
        };
        if max > 20 {
            return Err("Add-on max is at most 20".into());
        }
        return Ok((name.to_string(), ModifierGroupKind::Addon, 0, max));
    }
    Ok((raw.to_string(), ModifierGroupKind::Variation, 1, 1))
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
    /// Shown in tax pickers when true (like units).
    #[serde(default = "unit_picker_default_on")]
    pub picker_enabled: bool,
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
    /// Optional prep-station id — where this dish is made (Tandoor, Fryer…).
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
    /// Quantity unit label (`kg`, `plate`, empty = piece).
    #[serde(default)]
    pub unit_label: String,
    /// Optional dish family ("Paneer Tikka" for its Half and Full). Each
    /// variant is its own item with its own UUID, price, stock and station;
    /// the group is only a label to find and total them together — never
    /// identity (IDENTITY_AND_NAMES.md).
    #[serde(default)]
    pub group_name: String,
    /// Price by floor area (AC ₹220 when the menu says ₹200): JSON array of
    /// [`ModifierOption`] with the area as the name. Areas not listed use the
    /// item price.
    #[serde(default = "empty_json_array")]
    pub area_prices_json: String,
    /// What this item is to the restaurant: a dish it cooks and sells, a
    /// good it buys and resells as is, or a raw material it buys and uses
    /// up ([`ITEM_KIND_DISH`] / [`ITEM_KIND_RESALE`] / [`ITEM_KIND_RAW`]).
    /// A kitchen never buys its own dishes, and raw material is never on
    /// the menu.
    #[serde(default = "dish_kind")]
    pub kind: String,
    /// Menu timings / happy-hour: JSON array of [`ScheduleWindow`]. Empty =
    /// always available (still respects [`Self::available`]). Days: Mon=1 …
    /// Sun=7.
    #[serde(default = "empty_json_array")]
    pub schedule_json: String,
    /// Short allergen tags (`dairy, nuts`) printed on KOT.
    #[serde(default)]
    pub allergens: String,
}

/// One visibility and/or alternate-price window on a menu item.
///
/// Days use **Mon=1 … Sun=7** (ISO-ish; not chrono's Sun=0). Empty `days`
/// means every day. `price_paise: None` is a visibility window (hide outside);
/// `Some` is a happy-hour / timed price while still on the menu.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleWindow {
    #[serde(default)]
    pub days: Vec<u8>,
    pub start: String,
    pub end: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_paise: Option<i64>,
}

/// Cooked here and sold (on the menu; never bought).
pub const ITEM_KIND_DISH: &str = "dish";
/// Bought and sold as is — bottled drinks, packaged water (on the menu and
/// bought; stock counted).
pub const ITEM_KIND_RESALE: &str = "resale";
/// Bought and used up in the kitchen — rice, paneer, oil, gas (never on the
/// menu; stock counted).
pub const ITEM_KIND_RAW: &str = "raw";

fn dish_kind() -> String {
    ITEM_KIND_DISH.into()
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
            kind: ITEM_KIND_DISH.into(),
            group_name: String::new(),
            area_prices_json: "[]".into(),
            schedule_json: "[]".into(),
            allergens: String::new(),
        }
    }

    /// A raw material the kitchen buys and uses up: never on the menu,
    /// stock counted from zero.
    pub fn new_raw_material(name: &str, unit_label: &str) -> Self {
        let mut p = Self::new_basic(new_id(), name.trim(), Money(0));
        p.kind = ITEM_KIND_RAW.into();
        p.unit_label = unit_label.trim().to_string();
        p.stock_qty = 0;
        p
    }

    pub fn is_raw(&self) -> bool {
        self.kind == ITEM_KIND_RAW
    }

    /// Something the restaurant buys: raw material, a resold good, or an
    /// item whose stock it already counts.
    pub fn is_bought(&self) -> bool {
        self.kind == ITEM_KIND_RAW || self.kind == ITEM_KIND_RESALE || self.tracks_stock()
    }

    /// Variant items for `base` ("Paneer Tikka" + Half=120, Full=200 →
    /// "Paneer Tikka (Half)" ₹120 and "Paneer Tikka (Full)" ₹200): each a new
    /// item with a fresh UUID, the base's shared settings (category, station,
    /// tax, unit, options), and `base`'s name as their group. Prices that
    /// belong to one size — area prices — and short codes / aliases are not
    /// copied: each variant gets its own. Quick entry only — afterwards each
    /// is edited on its own.
    pub fn variant_items(base: &Product, variants: &[ModifierOption]) -> Vec<Product> {
        let group = if base.group_name.trim().is_empty() {
            base.name.trim().to_string()
        } else {
            base.group_name.trim().to_string()
        };
        variants
            .iter()
            .map(|v| Product {
                id: new_id(),
                name: format!("{} ({})", base.name.trim(), v.name.trim()),
                price: Money(v.price_paise),
                group_name: group.clone(),
                image_key: None,
                area_prices_json: "[]".into(),
                aliases_json: "[]".into(),
                ..base.clone()
            })
            .collect()
    }

    pub fn area_prices(&self) -> Vec<ModifierOption> {
        serde_json::from_str(&self.area_prices_json).unwrap_or_default()
    }

    pub fn set_area_prices(&mut self, prices: &[ModifierOption]) {
        self.area_prices_json = serde_json::to_string(prices).unwrap_or_else(|_| "[]".into());
    }

    /// The price at a table in `area` (case-insensitive); the item price
    /// where the area has no price of its own.
    pub fn price_in_area(&self, area: &str) -> Money {
        let area = area.trim();
        if area.is_empty() {
            return self.price;
        }
        self.area_prices()
            .into_iter()
            .find(|p| p.name.trim().eq_ignore_ascii_case(area))
            .map(|p| Money(p.price_paise))
            .unwrap_or(self.price)
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

    /// Flat option list for chips (each option carries its [`ModifierOption::group`]).
    pub fn modifiers(&self) -> Vec<ModifierOption> {
        ModifierOption::parse_list(&self.modifiers_json)
    }

    pub fn set_modifiers(&mut self, list: &[ModifierOption]) {
        self.modifiers_json = ModifierOption::serialize_list(list);
    }

    /// Petpooja-style groups: Size / Crust variations and Toppings add-ons.
    pub fn modifier_groups(&self) -> Vec<ModifierGroup> {
        ModifierGroup::parse_list(&self.modifiers_json)
    }

    pub fn set_modifier_groups(&mut self, groups: &[ModifierGroup]) {
        self.modifiers_json = ModifierGroup::serialize_list(groups);
    }

    pub fn schedule(&self) -> Vec<ScheduleWindow> {
        ScheduleWindow::parse_list(&self.schedule_json)
    }

    pub fn set_schedule(&mut self, windows: &[ScheduleWindow]) {
        self.schedule_json = ScheduleWindow::serialize_list(windows);
    }

    /// Area price, then a timed/happy-hour override when `now` falls in a
    /// priced schedule window.
    pub fn price_now(&self, area: &str, now: chrono::DateTime<chrono::Local>) -> Money {
        let base = self.price_in_area(area);
        let Some(override_paise) = self.schedule().into_iter().find_map(|w| {
            if w.price_paise.is_some() && w.contains(now) {
                w.price_paise
            } else {
                None
            }
        }) else {
            return base;
        };
        Money(override_paise)
    }

    /// Visibility: empty schedule → always (respect sold-out). If any
    /// visibility-only windows exist, the item is shown only inside one of
    /// them (priced windows do not themselves hide the item).
    pub fn available_now(&self, now: chrono::DateTime<chrono::Local>) -> bool {
        if !self.available || self.archived {
            return false;
        }
        let windows = self.schedule();
        if windows.is_empty() {
            return true;
        }
        let visibility: Vec<_> = windows.iter().filter(|w| w.price_paise.is_none()).collect();
        if visibility.is_empty() {
            return true;
        }
        visibility.iter().any(|w| w.contains(now))
    }

    /// Tax this item contributes for `amount_paise` under `pricing` — inside
    /// the amount when Inclusive, added on top when Exclusive.
    pub fn tax_for(&self, amount_paise: i64, pricing: TaxPricing) -> i64 {
        fiscal::tax_on_amount(amount_paise, self.tax_bps, pricing)
    }
}

impl ScheduleWindow {
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

    /// Human form like area prices: `Mon-Fri 07:00-11:00` (visibility) and/or
    /// `HappyHour 17:00-19:00=150` (alt price). Comma-separated.
    pub fn parse_human_list(text: &str, currency_code: &str) -> Result<Vec<Self>, String> {
        let mut out = Vec::new();
        for part in text.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            out.push(Self::parse_one(part, currency_code)?);
        }
        Ok(out)
    }

    fn parse_one(part: &str, currency_code: &str) -> Result<Self, String> {
        let (left, price_paise) = match part.rsplit_once('=') {
            Some((l, p)) => {
                let paise = crate::locale::parse_major_input(p.trim(), currency_code)
                    .map_err(|_| format!("Bad schedule price in '{part}'"))?;
                if paise < 0 {
                    return Err("Schedule price cannot be negative".into());
                }
                (l.trim(), Some(paise))
            }
            None => (part, None),
        };
        // Optional leading label ("HappyHour ") before days/time.
        let tokens: Vec<&str> = left.split_whitespace().collect();
        if tokens.is_empty() {
            return Err("Schedule window needs a time range".into());
        }
        let (days, time_tok) = if tokens.len() == 1 {
            (Vec::new(), tokens[0])
        } else if looks_like_time_range(tokens.last().copied().unwrap_or("")) {
            let time = tokens[tokens.len() - 1];
            let day_part = tokens[..tokens.len() - 1]
                .iter()
                .copied()
                .filter(|t| parse_day_token(t).is_some() || t.contains('-'))
                .collect::<Vec<_>>()
                .join(" ");
            let days = if day_part.is_empty() {
                Vec::new()
            } else {
                parse_days(&day_part)?
            };
            (days, time)
        } else {
            return Err(format!("Schedule needs HH:MM-HH:MM in '{part}'"));
        };
        let (start, end) = parse_time_range(time_tok)?;
        Ok(Self {
            days,
            start,
            end,
            price_paise,
        })
    }

    /// True when `now`'s weekday (Mon=1…Sun=7) and clock fall in this window.
    pub fn contains(&self, now: chrono::DateTime<chrono::Local>) -> bool {
        use chrono::Datelike;
        let dow = match now.weekday() {
            chrono::Weekday::Mon => 1,
            chrono::Weekday::Tue => 2,
            chrono::Weekday::Wed => 3,
            chrono::Weekday::Thu => 4,
            chrono::Weekday::Fri => 5,
            chrono::Weekday::Sat => 6,
            chrono::Weekday::Sun => 7,
        };
        if !self.days.is_empty() && !self.days.contains(&dow) {
            return false;
        }
        let mins = now.hour() as i32 * 60 + now.minute() as i32;
        let start = parse_hhmm(&self.start).unwrap_or(0);
        let end = parse_hhmm(&self.end).unwrap_or(24 * 60);
        if start <= end {
            mins >= start && mins < end
        } else {
            // Overnight window (22:00–02:00).
            mins >= start || mins < end
        }
    }
}

fn looks_like_time_range(s: &str) -> bool {
    s.contains(':') && s.contains('-')
}

fn parse_time_range(s: &str) -> Result<(String, String), String> {
    let (a, b) = s
        .split_once('-')
        .ok_or_else(|| format!("Time range needs HH:MM-HH:MM, got '{s}'"))?;
    let start = normalize_hhmm(a.trim())?;
    let end = normalize_hhmm(b.trim())?;
    Ok((start, end))
}

fn normalize_hhmm(s: &str) -> Result<String, String> {
    let _ = parse_hhmm(s).ok_or_else(|| format!("Bad time '{s}' (want HH:MM)"))?;
    let parts: Vec<_> = s.split(':').collect();
    Ok(format!(
        "{:02}:{:02}",
        parts[0].parse::<u32>().unwrap_or(0),
        parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0)
    ))
}

fn parse_hhmm(s: &str) -> Option<i32> {
    let (h, m) = s.split_once(':')?;
    let h: i32 = h.parse().ok()?;
    let m: i32 = m.parse().ok()?;
    if (0..24).contains(&h) && (0..60).contains(&m) {
        Some(h * 60 + m)
    } else {
        None
    }
}

fn parse_days(text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for bit in text.split(|c| c == ',' || c == '/' || c == '&') {
        let bit = bit.trim();
        if bit.is_empty() {
            continue;
        }
        if let Some((a, b)) = bit.split_once('-') {
            let from = parse_day_token(a.trim())
                .ok_or_else(|| format!("Bad day '{a}'"))?;
            let to = parse_day_token(b.trim()).ok_or_else(|| format!("Bad day '{b}'"))?;
            if from <= to {
                out.extend(from..=to);
            } else {
                out.extend(from..=7);
                out.extend(1..=to);
            }
        } else {
            let d = parse_day_token(bit).ok_or_else(|| format!("Bad day '{bit}'"))?;
            out.push(d);
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

fn parse_day_token(s: &str) -> Option<u8> {
    match s.trim().to_ascii_lowercase().as_str() {
        "mon" | "monday" => Some(1),
        "tue" | "tues" | "tuesday" => Some(2),
        "wed" | "wednesday" => Some(3),
        "thu" | "thur" | "thurs" | "thursday" => Some(4),
        "fri" | "friday" => Some(5),
        "sat" | "saturday" => Some(6),
        "sun" | "sunday" => Some(7),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
    /// Set when this line already went to the kitchen on the table's running
    /// order ([`RunningLine::id`]). Empty = new, not sent yet. A sent line is
    /// never bumped or dropped like a new one — it is cancelled with a
    /// reason ([`crate::store::Store::cancel_running_line`]).
    pub running_line_id: String,
    /// Complimentary (NC): kept at its real item and qty, billed ₹0.
    pub complimentary: bool,
    /// The KOT this sent line went out on ([`RunningLine::kot_id`]); empty
    /// for a line not sent yet. The ticket groups lines by it.
    pub kot_id: String,
    /// Held back from the next Send KOT (Order desk). Still on the ticket;
    /// tap Hold again to include it. Not the same as a sent-line kitchen hold.
    #[serde(default)]
    pub hold_send: bool,
}

/// How many modifier chips fit on one cart row.
pub const MODIFIER_PAGE: usize = 3;

/// Why a menu item's price cannot be saved, or `None` when it is fine.
///
/// ₹0 is a real price: complimentary water, a free chutney, a dish priced at
/// the counter by its `@`. The menu and the till agree on it — the form saves
/// it and [`Cart::add_or_bump_in_stock`] bills it. Only a negative price is
/// refused (a discount is a line/bill edit, not a menu price).
pub fn menu_price_error(price: Money) -> Option<String> {
    if price.paise() < 0 {
        Some("Price cannot be negative".into())
    } else {
        None
    }
}

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

    /// Applied line total: override if set, otherwise Actual. A
    /// complimentary (NC) line is billed ₹0; its Actual keeps the real value.
    pub fn applied_total(&self) -> Money {
        if self.complimentary {
            return Money(0);
        }
        self.line_total_override
            .unwrap_or_else(|| self.actual_total())
    }

    /// Already carried on a KOT to the kitchen (`kot_id` set). Pending lines
    /// on the table's running order have a `running_line_id` but empty `kot_id`.
    pub fn is_sent(&self) -> bool {
        !self.kot_id.is_empty()
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
            .map(|m| {
                if m.group.is_empty() || m.group == "Options" {
                    m.name.clone()
                } else {
                    format!("{} {}", m.group, m.name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        match (mods.is_empty(), self.note.is_empty()) {
            (true, _) => self.note.clone(),
            (false, true) => mods,
            (false, false) => format!("{mods}, {}", self.note),
        }
    }

    pub(crate) fn reprice_from_modifiers(&mut self) {
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
    /// Bill discount (10 % or ₹50) with its reason; spread over the lines
    /// at finalize so tax is on what is actually charged.
    pub discount: Option<BillDiscount>,
    /// Delivery orders: who takes it out (staff or rider name).
    pub delivery_by: String,
    /// Names of schemes applied at settle (shown on the ticket as "Scheme: …").
    pub scheme_notes: Vec<String>,
}

/// How much off the bill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscountKind {
    /// Percent in basis points (1000 = 10 %).
    Percent(i64),
    /// A fixed amount (paise).
    Amount(i64),
}

/// A discount on the whole bill — a right (vision §17): reason always,
/// a manager's PIN when the person's right says "with approval".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillDiscount {
    pub kind: DiscountKind,
    pub reason: String,
    /// Manager PIN typed on the device; the Server checks it at finalize.
    pub approval_pin: String,
}

impl DiscountKind {
    /// `10%` / `10 %` → 10 percent; `50` / `50.5` → that amount.
    pub fn parse(text: &str, currency_code: &str) -> Result<Self, String> {
        let t = text.trim();
        if let Some(p) = t.strip_suffix('%') {
            let bps = crate::locale::parse_percent_bps(p.trim())?;
            if bps <= 0 || bps > 10_000 {
                return Err("Discount must be between 0 and 100 %".into());
            }
            return Ok(Self::Percent(bps));
        }
        let paise = crate::locale::parse_major_input(t, currency_code)
            .map_err(|_| "Type a discount like 10% or 50".to_string())?;
        if paise <= 0 {
            return Err("Discount must be more than 0".into());
        }
        Ok(Self::Amount(paise))
    }
}

impl Cart {
    pub fn clear(&mut self) {
        self.lines.clear();
        self.charge_total = None;
        self.discount = None;
        self.delivery_by.clear();
        self.scheme_notes.clear();
        self.order_type = OrderType::Counter;
        // Keep money_lane — owner choice persists across carts until they flip the chip.
    }

    pub fn set_order_type(&mut self, order_type: OrderType) {
        self.order_type = order_type;
    }

    pub fn add_or_bump(&mut self, product: &Product) {
        self.add_priced(product, product.price);
    }

    /// Add one of `product` at `unit_price` (its price in the table's area).
    fn add_priced(&mut self, product: &Product, unit_price: Money) {
        if !product.available || product.archived || unit_price.paise() < 0 {
            return;
        }
        self.charge_total = None;
        // A line already sent to the kitchen is history: another tap starts
        // (or bumps) a new line that goes on the next KOT.
        if let Some(line) = self.lines.iter_mut().find(|l| {
            l.product_id == product.id && !l.is_sent() && !l.complimentary
        }) {
            line.line_total_override = None;
            let step = line.bump_step_milli();
            line.qty_milli = line.qty_milli.saturating_add(step);
            line.sync_qty_from_milli();
        } else {
            let step = if product.is_weight_unit() { 100 } else { QTY_MILLI_UNIT };
            let mut line = CartLine {
                product_id: product.id.clone(),
                name: product.name.clone(),
                unit_price,
                qty: 0,
                qty_milli: step,
                unit_label: product.unit_label.clone(),
                note: String::new(),
                base_unit_paise: unit_price.paise(),
                applied_modifiers: Vec::new(),
                modifier_page: 0,
                line_total_override: None,
                running_line_id: String::new(),
                complimentary: false,
                kot_id: String::new(),
                hold_send: false,
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
        self.add_in_area_in_stock(product, "")
    }

    /// Add one at the table's area price, refusing what the shop cannot
    /// serve.
    pub fn add_in_area_in_stock(&mut self, product: &Product, area: &str) -> Result<(), String> {
        let now = chrono::Local::now();
        if !product.available_now(now) {
            return Err(format!("{} is not on the menu right now", product.name));
        }
        let unit_price = product.price_now(area, now);
        self.check_can_add(product, unit_price)?;
        self.add_priced(product, unit_price);
        Ok(())
    }

    pub fn check_can_add(&self, product: &Product, unit_price: Money) -> Result<(), String> {
        if !product.available || product.archived {
            return Err(format!("{} is marked sold out", product.name));
        }
        if unit_price.paise() < 0 {
            return Err(format!(
                "{} has a negative price — fix it in Menu & items",
                product.name
            ));
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
        Ok(())
    }

    /// One fewer of `product_id` (the menu window's −): the line still to
    /// go to the kitchen gives one back, and leaves at nought. What the
    /// kitchen already has changes only through a reasoned cancel on the
    /// ticket, so that is an error here.
    pub fn remove_one_of(&mut self, product_id: &str) -> Result<(), String> {
        let open = |l: &CartLine| l.product_id == product_id && !l.is_sent();
        let index = self
            .lines
            .iter()
            .rposition(|l| open(l) && !l.complimentary)
            .or_else(|| self.lines.iter().rposition(open));
        match index {
            Some(index) => {
                self.bump_qty(index, -1);
                Ok(())
            }
            None if self.lines.iter().any(|l| l.product_id == product_id) => {
                Err("Already sent to the kitchen — cancel it from the ticket".into())
            }
            None => Ok(()),
        }
    }

    pub fn bump_qty(&mut self, index: usize, delta: i64) {
        // A sent line changes only through a reasoned cancel (store).
        if let Some(line) = self.lines.get_mut(index).filter(|l| !l.is_sent()) {
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
        if let Some(line) = self.lines.get_mut(index).filter(|l| !l.is_sent()) {
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
        if unit_price.paise() < 0 {
            return Err("Unit price cannot be negative".into());
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

    /// Turn a catalogue modifier on or off for this line. Variation groups
    /// keep only one choice; add-on groups honour their max. The unit price
    /// follows. Returns whether it is on after the tap.
    pub fn toggle_modifier(
        &mut self,
        index: usize,
        modifier: &ModifierOption,
    ) -> Result<bool, String> {
        self.toggle_modifier_in_groups(index, modifier, &[])
    }

    /// Like [`Self::toggle_modifier`], with catalogue groups for exclusivity /
    /// max rules (pass [`Product::modifier_groups`]).
    pub fn toggle_modifier_in_groups(
        &mut self,
        index: usize,
        modifier: &ModifierOption,
        groups: &[ModifierGroup],
    ) -> Result<bool, String> {
        if modifier.name.trim().is_empty() {
            return Err("Modifier name required".into());
        }
        let group_name = modifier.group.trim();
        let group = groups.iter().find(|g| {
            if !group_name.is_empty() {
                g.name == group_name
            } else {
                g.options.iter().any(|o| o.name == modifier.name)
            }
        });
        let line = self
            .lines
            .get_mut(index)
            .ok_or_else(|| "Missing cart line".to_string())?;
        let same = |m: &ModifierOption| {
            m.name == modifier.name
                && (group_name.is_empty()
                    || m.group.is_empty()
                    || m.group == group_name)
        };
        let turning_on = !line.applied_modifiers.iter().any(same);
        if turning_on {
            if let Some(g) = group {
                if g.is_variation() || g.max_allowed() == 1 {
                    let gname = g.name.as_str();
                    line.applied_modifiers.retain(|m| {
                        let mg = m.group.trim();
                        mg.is_empty() || mg != gname
                    });
                } else {
                    let gname = g.name.as_str();
                    let count = line
                        .applied_modifiers
                        .iter()
                        .filter(|m| m.group == gname || (m.group.is_empty() && group_name.is_empty()))
                        .count();
                    if count >= g.max_allowed() {
                        return Err(format!(
                            "{}: pick at most {}",
                            g.name,
                            g.max_allowed()
                        ));
                    }
                }
            }
            let mut applied = modifier.clone();
            if applied.group.is_empty() {
                if let Some(g) = group {
                    applied.group = g.name.clone();
                }
            }
            line.applied_modifiers.push(applied);
        } else {
            // Required variation: tapping the only pick again does not clear it.
            if let Some(g) = group {
                if g.is_variation() && g.min_required() > 0 {
                    let gname = g.name.as_str();
                    let others = line
                        .applied_modifiers
                        .iter()
                        .filter(|m| !same(m) && (m.group == gname))
                        .count();
                    if others == 0 {
                        return Ok(true);
                    }
                }
            }
            line.applied_modifiers.retain(|m| !same(m));
        }
        line.reprice_from_modifiers();
        if line.unit_price.paise() < 0 {
            return Err("Unit price cannot be negative".into());
        }
        line.line_total_override = None;
        self.charge_total = None;
        Ok(turning_on)
    }

    /// Lines that still need a required variation (e.g. Size) before pay / KOT.
    pub fn missing_required_modifiers<'a>(
        &self,
        product_groups: impl Fn(&str) -> Vec<ModifierGroup>,
    ) -> Vec<String> {
        let mut out = Vec::new();
        for line in &self.lines {
            let groups = product_groups(&line.product_id);
            for g in groups {
                let need = g.min_required();
                if need == 0 {
                    continue;
                }
                let have = line
                    .applied_modifiers
                    .iter()
                    .filter(|m| m.group == g.name || (m.group.is_empty() && g.name == "Options"))
                    .count();
                if have < need {
                    out.push(format!("{}: pick {}", line.name, g.name));
                }
            }
        }
        out
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
        if total.paise() < 0 {
            return Err("Line Applied Total cannot be negative".into());
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
        if total.paise() < 0 {
            return Err("Total cannot be negative".into());
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

    /// Applied Total: sum of each line's Applied amount, less the bill
    /// discount.
    pub fn applied_total(&self) -> Money {
        Money(self.lines_applied_paise() - self.discount_paise())
    }

    /// Sum of the lines' Applied amounts before any bill discount.
    pub fn lines_applied_paise(&self) -> i64 {
        self.lines
            .iter()
            .fold(Money(0), |acc, line| acc.saturating_add(line.applied_total()))
            .paise()
    }

    /// The bill discount in money (never more than the bill).
    pub fn discount_paise(&self) -> i64 {
        let base = self.lines_applied_paise().max(0);
        match self.discount.as_ref().map(|d| d.kind) {
            None => 0,
            // Half-up rounding to the paisa.
            Some(DiscountKind::Percent(bps)) => ((base as i128 * bps as i128 + 5_000) / 10_000) as i64,
            Some(DiscountKind::Amount(paise)) => paise.min(base),
        }
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
                    let mods = l.modifiers();
                    let base = l.unit_price_paise
                        - mods.iter().map(|m| m.price_paise).sum::<i64>();
                    let mut line = CartLine {
                        product_id: l.product_id.clone(),
                        name: l.name.clone(),
                        unit_price: Money(l.unit_price_paise),
                        qty: l.qty,
                        qty_milli: milli,
                        unit_label: l.unit_label.clone(),
                        // Free-text only — modifiers live on applied_modifiers.
                        note: {
                            let kitchen = l.note.clone();
                            let mod_note = mods
                                .iter()
                                .map(|m| {
                                    if m.group.is_empty() || m.group == "Options" {
                                        m.name.clone()
                                    } else {
                                        format!("{} {}", m.group, m.name)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            if mod_note.is_empty() {
                                kitchen
                            } else if kitchen == mod_note {
                                String::new()
                            } else if let Some(rest) = kitchen.strip_prefix(&format!("{mod_note}, "))
                            {
                                rest.to_string()
                            } else if kitchen.starts_with(&mod_note) {
                                String::new()
                            } else {
                                kitchen
                            }
                        },
                        base_unit_paise: base.max(0),
                        applied_modifiers: mods,
                        modifier_page: 0,
                        line_total_override: override_total,
                        running_line_id: String::new(),
                        complimentary: false,
                        kot_id: String::new(),
                        hold_send: false,
                    };
                    line.sync_qty_from_milli();
                    line
                })
                .collect(),
            charge_total: None,
            money_lane: MoneyLane::parse(&sale.money_lane),
            discount: None,
            // Kept on the sale's own delivery record; amending leaves it be.
            delivery_by: String::new(),
            scheme_notes: Vec::new(),
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
    /// The lines not yet sent to the kitchen, as a draft for the next KOT.
    /// Unlike a bill, a KOT may be all ₹0 (complimentary water is still
    /// cooked/served), so only "nothing new" is refused.
    pub fn to_kot_draft(&self) -> Result<SaleDraft, String> {
        let lines: Vec<SaleLine> = self
            .lines
            .iter()
            .filter(|l| !l.is_sent() && !l.hold_send && l.qty_milli > 0)
            .map(|l| sale_line_from_cart(l, false))
            .collect();
        if lines.is_empty() {
            let held = self.lines.iter().any(|l| !l.is_sent() && l.hold_send);
            return Err(if held {
                "Every new dish is on Hold — tap Hold on a line to include it, then Send KOT".into()
            } else {
                "Nothing new to send — tap items first".into()
            });
        }
        let total: i64 = lines.iter().map(|l| l.line_total_paise).sum();
        Ok(SaleDraft {
            lines,
            lines_total_paise: total,
            rounding_paise: 0,
            total_paise: total,
            tax_paise: 0,
            order_type: self.order_type,
            money_lane: self.money_lane,
            user_at: None,
            extras_json: "[]".into(),
            delivery_by: self.delivery_by.trim().to_string(),
            notify_phone: String::new(),
            discount_paise: 0,
            discount_reason: String::new(),
            discount_approval_pin: String::new(),
        })
    }

    /// Mark the cart's new lines as sent with no running order behind them
    /// (a counter KOT before payment): the kitchen has them, so finalize must
    /// not fire them again.
    pub fn mark_new_lines_sent(&mut self, marker: &str) {
        for line in self.lines.iter_mut().filter(|l| !l.is_sent()) {
            line.running_line_id = marker.to_string();
        }
    }

    pub fn to_sale_draft(&self) -> Result<SaleDraft, String> {
        if self.lines.is_empty() {
            return Err("Cart is empty".into());
        }
        let lines_total = self.line_sum().paise();
        if lines_total < 0 {
            return Err("Sale total cannot be negative".into());
        }
        let total = self.total().paise();
        if total < 0 {
            return Err("Sale total cannot be negative".into());
        }
        let rounding = total.saturating_sub(lines_total);
        let lines: Vec<SaleLine> = self.lines.iter().map(|l| sale_line_from_cart(l, l.is_sent())).collect();
        // The bill discount comes off each line in proportion, so tax (by
        // line) is on what is really charged; the last charged line takes
        // the rounding remainder so the lines still add up exactly.
        let mut lines = lines;
        let discount = self.discount_paise();
        if discount > 0 {
            let base = self.lines_applied_paise();
            let mut left = discount;
            let last = lines.iter().rposition(|l| l.line_total_paise > 0);
            for (i, line) in lines.iter_mut().enumerate() {
                if line.line_total_paise <= 0 {
                    continue;
                }
                let share = if Some(i) == last {
                    left
                } else {
                    (discount as i128 * line.line_total_paise as i128 / base as i128) as i64
                };
                let share = share.min(line.line_total_paise).min(left);
                line.line_total_paise -= share;
                left -= share;
            }
        }
        // Applied line totals must sum to lines_total; unit prices stay true.
        debug_assert_eq!(
            lines.iter().map(|l| l.line_total_paise).sum::<i64>(),
            lines_total
        );
        let extras_json = if self.scheme_notes.is_empty() {
            "[]".into()
        } else {
            let extras: Vec<serde_json::Value> = self
                .scheme_notes
                .iter()
                .map(|n| serde_json::json!({ "kind": "scheme", "name": n, "amount_paise": 0 }))
                .collect();
            serde_json::to_string(&extras).unwrap_or_else(|_| "[]".into())
        };
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
            extras_json,
            delivery_by: self.delivery_by.trim().to_string(),
            notify_phone: String::new(),
            discount_paise: discount,
            discount_reason: self.discount.as_ref().map(|d| d.reason.clone()).unwrap_or_default(),
            discount_approval_pin: self
                .discount
                .as_ref()
                .map(|d| d.approval_pin.clone())
                .unwrap_or_default(),
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
    /// Already sent to the kitchen (on the table's running order) — finalize
    /// must not fire a second KOT for it.
    #[serde(default)]
    pub kot_sent: bool,
    /// Chosen Size / toppings at sale time (JSON array of [`ModifierOption`]).
    #[serde(default = "empty_json_array")]
    pub modifiers_json: String,
}

impl SaleLine {
    pub fn modifiers(&self) -> Vec<ModifierOption> {
        let s = self.modifiers_json.trim();
        if s.is_empty() {
            return Vec::new();
        }
        serde_json::from_str(s).unwrap_or_default()
    }

    pub fn set_modifiers_from(&mut self, mods: &[ModifierOption]) {
        self.modifiers_json = if mods.is_empty() {
            "[]".into()
        } else {
            serde_json::to_string(mods).unwrap_or_else(|_| "[]".into())
        };
    }
}

fn sale_line_from_cart(l: &CartLine, kot_sent: bool) -> SaleLine {
    let milli = if l.qty_milli > 0 {
        l.qty_milli
    } else {
        qty_to_milli(l.qty)
    };
    let mut line = SaleLine {
        product_id: l.product_id.clone(),
        name: l.name.clone(),
        unit_price_paise: l.unit_price.paise(),
        qty: l.qty,
        line_total_paise: l.applied_total().paise(),
        note: l.kitchen_note(),
        qty_milli: milli,
        unit_label: l.unit_label.clone(),
        kot_sent,
        modifiers_json: "[]".into(),
    };
    line.set_modifiers_from(&l.applied_modifiers);
    line
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
    /// JSON array of `{kind,name,amount_paise}` extra charges already in lines or noted.
    #[serde(default = "empty_json_array")]
    pub extras_json: String,
    /// Optional phone for order alert outbox.
    #[serde(default)]
    pub notify_phone: String,
    /// Bill discount already taken off the lines (paise), and why.
    #[serde(default)]
    pub discount_paise: i64,
    #[serde(default)]
    pub discount_reason: String,
    /// Manager PIN for a discount needing approval — checked by the Server
    /// at finalize, never stored.
    #[serde(default)]
    pub discount_approval_pin: String,
    /// Delivery orders: who takes it out.
    #[serde(default)]
    pub delivery_by: String,
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
        // ₹0 bills are allowed here (complimentary / owner wish). Settings
        // `allow_zero_sales` can still refuse them at finalize time.
        if self.total_paise < 0 {
            return Err("Sale total cannot be negative".into());
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
    /// Bill-type tag (`declared` = tax invoice; `owner` = cash memo).
    #[serde(default)]
    pub money_lane: String,
    #[serde(default)]
    pub extras_json: String,
    pub lines: Vec<SaleLine>,
}

impl Sale {
    pub fn is_unbilled_account(&self) -> bool {
        self.pay_method == "credit"
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
    /// Outstanding **AR** in paise (positive = they owe us). AP is on payable_ledger.
    pub balance_paise: i64,
    /// Role tag(s): `Customer`, `Supplier`, or combined
    /// (`Customer, Supplier`). One party identity — not two masters.
    #[serde(default)]
    pub kind_label: String,
    /// Max outstanding for account booking (paise). 0 = no limit.
    #[serde(default)]
    pub credit_limit_paise: i64,
    /// Local loyalty points (earn on pay / redeem at till).
    #[serde(default)]
    pub loyalty_points: i64,
    /// Prepaid wallet balance (paise).
    #[serde(default)]
    pub wallet_paise: i64,
    /// Birthday as `MM-DD` or `YYYY-MM-DD` (CRM birthday messages). Empty = unknown.
    #[serde(default)]
    pub birthday: String,
}

impl Customer {
    pub fn balance(&self) -> Money {
        Money(self.balance_paise)
    }

    pub fn credit_limit(&self) -> Money {
        Money(self.credit_limit_paise)
    }

    pub fn is_customer(&self) -> bool {
        party_has_role(&self.kind_label, PARTY_ROLE_CUSTOMER)
    }

    pub fn is_supplier(&self) -> bool {
        party_has_role(&self.kind_label, PARTY_ROLE_SUPPLIER)
    }

    /// Month-day of [`Self::birthday`] when the field is set.
    pub fn birthday_md(&self) -> Option<(u32, u32)> {
        birthday_month_day(&self.birthday)
    }
}

/// Normalize birthday input to `MM-DD` (or empty). Accepts `MM-DD` or `YYYY-MM-DD`.
pub fn normalize_birthday(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Ok(String::new());
    }
    let (m, d) = birthday_month_day(t).ok_or_else(|| {
        "Birthday must be MM-DD or YYYY-MM-DD".to_string()
    })?;
    Ok(format!("{m:02}-{d:02}"))
}

/// Parse `MM-DD` or `YYYY-MM-DD` to (month, day).
pub fn birthday_month_day(raw: &str) -> Option<(u32, u32)> {
    let t = raw.trim();
    let parts: Vec<&str> = t.split('-').collect();
    let (m, d) = match parts.as_slice() {
        [mm, dd] => (mm.parse().ok()?, dd.parse().ok()?),
        [_, mm, dd] => (mm.parse().ok()?, dd.parse().ok()?),
        _ => return None,
    };
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some((m, d))
}

/// True when the guest's birthday month-day falls on any day in `[from, to]` (inclusive).
pub fn birthday_in_range(birthday: &str, from: chrono::NaiveDate, to: chrono::NaiveDate) -> bool {
    use chrono::Datelike;
    let Some((m, d)) = birthday_month_day(birthday) else {
        return false;
    };
    let mut day = from;
    loop {
        if day.month() == m && day.day() == d {
            return true;
        }
        if day >= to {
            return false;
        }
        day = match day.succ_opt() {
            Some(n) => n,
            None => return false,
        };
    }
}

/// Canonical role tokens stored in [`Customer::kind_label`] (comma-separated).
/// A Customer runs a khata tab with us (AR); a Supplier sells stock to us (AP).
pub const PARTY_ROLE_CUSTOMER: &str = "Customer";
pub const PARTY_ROLE_SUPPLIER: &str = "Supplier";

/// True if `kind_label` lists this role (case-insensitive; comma / `+` / `/` separators).
pub fn party_has_role(kind_label: &str, role: &str) -> bool {
    let want = role.trim().to_ascii_lowercase();
    if want.is_empty() {
        return false;
    }
    party_role_tokens(kind_label)
        .iter()
        .any(|t| t.eq_ignore_ascii_case(&want))
}

fn party_role_tokens(kind_label: &str) -> Vec<String> {
    kind_label
        .split(|c: char| c == ',' || c == '+' || c == '/' || c == ';' || c == '|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Normalize role text: unique tokens, title-ish known roles, sorted for stable storage.
///
/// `"supplier + customer"` → `"Customer, Supplier"`.
pub fn normalize_party_roles(kind_label: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for raw in party_role_tokens(kind_label) {
        let canon = match raw.to_ascii_lowercase().as_str() {
            "customer" | "debtor" | "guest" => PARTY_ROLE_CUSTOMER.to_string(),
            "supplier" | "vendor" | "creditor" => PARTY_ROLE_SUPPLIER.to_string(),
            "both" | "dual" => {
                // Expand to both party roles.
                for r in [PARTY_ROLE_CUSTOMER, PARTY_ROLE_SUPPLIER] {
                    if !out.iter().any(|x| x == r) {
                        out.push(r.to_string());
                    }
                }
                continue;
            }
            _ => {
                // Preserve custom labels with first letter uppercased when ASCII.
                let mut c = raw.chars();
                match c.next() {
                    Some(f) => format!("{}{}", f.to_uppercase(), c.as_str()),
                    None => continue,
                }
            }
        };
        if !out.iter().any(|x| x.eq_ignore_ascii_case(&canon)) {
            out.push(canon);
        }
    }
    out.sort();
    out.join(", ")
}

/// Merge two kind labels (e.g. existing Customer + new Supplier → both).
pub fn merge_party_roles(existing: &str, added: &str) -> String {
    normalize_party_roles(&format!("{existing}, {added}"))
}

/// Parent of an [`EntryAttachment`] (books row).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentEntity {
    Sale,
    Party,
    Expense,
    Purchase,
}

impl AttachmentEntity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sale => "sale",
            Self::Party => "party",
            Self::Expense => "expense",
            Self::Purchase => "purchase",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "sale" => Some(Self::Sale),
            "party" | "customer" => Some(Self::Party),
            "expense" => Some(Self::Expense),
            "purchase" => Some(Self::Purchase),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentKind {
    Note,
    Image,
    File,
}

impl AttachmentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Image => "image",
            Self::File => "file",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "note" => Some(Self::Note),
            "image" | "photo" => Some(Self::Image),
            "file" | "doc" | "pdf" => Some(Self::File),
            _ => None,
        }
    }
}

/// Note, WebP photo, or PDF/doc on a sale / party / expense / …
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntryAttachment {
    pub id: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub created_at: String,
    #[serde(default)]
    pub user_at: String,
    #[serde(default)]
    pub device_at: String,
    /// `note` | `image` | `file`
    pub kind: String,
    pub label: String,
    /// Note body, or empty for files.
    pub body: String,
    /// Filename under attachments/ (empty for notes).
    pub stored_name: String,
    pub mime: String,
    pub bytes: i64,
    pub actor_id: String,
    pub actor_name: String,
}

/// Combined AR/AP position for one party (overall who-owes-whom).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartyPositionRow {
    pub customer_id: String,
    pub name: String,
    pub kind_label: String,
    /// They owe us (AR). Positive = receivable.
    pub ar_paise: i64,
    /// We owe them (AP). Positive = payable.
    pub ap_paise: i64,
    /// `ar_paise - ap_paise`. Positive → they owe us net; negative → we owe them net.
    pub net_paise: i64,
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
    /// Guests are done; the counter has been asked for the bill.
    /// The table is released only when payment is recorded.
    PaymentPending,
    /// Paid; guests leaving, table to be cleared. Never blocks: "Table
    /// ready" frees it, seating new guests starts fresh, and it clears by
    /// itself after [`PAID_TABLE_CLEARS_AFTER_MIN`] minutes.
    Paid,
}

/// Minutes a paid table shows "Paid — clear the table" before it reads as
/// free again on its own.
pub const PAID_TABLE_CLEARS_AFTER_MIN: i64 = 15;

impl TableStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Occupied => "occupied",
            Self::PaymentPending => "payment_pending",
            Self::Paid => "paid",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "occupied" => Self::Occupied,
            "payment_pending" => Self::PaymentPending,
            "paid" => Self::Paid,
            _ => Self::Available,
        }
    }

    /// Short floor-card wording.
    pub fn label(self) -> &'static str {
        match self {
            Self::Available => "Free",
            Self::Occupied => "Running",
            Self::PaymentPending => "Payment pending",
            Self::Paid => "Paid",
        }
    }

    /// Seated with an open bill (running or waiting to pay). A paid table
    /// is not open: it never blocks seating new guests.
    pub fn is_open(self) -> bool {
        matches!(self, Self::Occupied | Self::PaymentPending)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiningTable {
    pub id: String,
    pub name: String,
    pub status: TableStatus,
    pub active_sale_id: Option<String>,
    /// RFC3339 instant the current seating started (empty when free).
    #[serde(default)]
    pub occupied_at: String,
    /// Open-bill amount sent for this seating (paise), for the floor card.
    #[serde(default)]
    pub running_total_paise: i64,
    /// When the last bill on this table was paid (RFC3339; empty if not).
    #[serde(default)]
    pub paid_at: String,
    /// Where the visit stands, worked out on the Server from the table, its
    /// order and its kitchen tickets — every device shows the same.
    #[serde(default)]
    pub floor_state: FloorState,
    /// Floor area this table sits in ("Dine In", "Parcel", "AC" …). Empty =
    /// the floor's unnamed area.
    #[serde(default)]
    pub area: String,
    /// Waiter section: staff id assigned to this table (empty = unassigned).
    #[serde(default)]
    pub waiter_staff_id: String,
    /// Optional drawn layout (0 = use grid).
    #[serde(default)]
    pub layout_x: f64,
    #[serde(default)]
    pub layout_y: f64,
    #[serde(default)]
    pub layout_w: f64,
    #[serde(default)]
    pub layout_h: f64,
}

/// A table's place in the guests' visit, as the floor shows it (learned from
/// how the best restaurant systems colour their floor): free → seated → food
/// in the kitchen → food ready → eating → bill printed → paid → free.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloorState {
    #[default]
    Free,
    /// Guests seated; no dishes punched yet (no live order / cooking).
    Seated,
    /// KOTs being prepared.
    InKitchen,
    /// Every open KOT is Ready — serve it.
    FoodReady,
    /// Food served (no open KOTs); the guests are eating.
    Eating,
    /// Bill printed or requested; waiting for payment.
    BillPrinted,
    /// Paid; clear the table.
    Paid,
}

impl FloorState {
    /// Work out the visit's state from the table, whether it has a live
    /// order, and its open KOTs (food tickets only — cancel slips do not
    /// count).
    pub fn derive(
        status: TableStatus,
        open_kot_statuses: &[KotStatus],
        has_live_order: bool,
    ) -> Self {
        match status {
            TableStatus::Available => Self::Free,
            TableStatus::Paid => Self::Paid,
            TableStatus::PaymentPending => Self::BillPrinted,
            TableStatus::Occupied => {
                let cooking: Vec<_> = open_kot_statuses
                    .iter()
                    .filter(|s| !matches!(s, KotStatus::Completed | KotStatus::Cancelled))
                    .collect();
                if !cooking.is_empty() {
                    if cooking.iter().all(|s| **s == KotStatus::Ready) {
                        Self::FoodReady
                    } else {
                        Self::InKitchen
                    }
                } else if has_live_order {
                    Self::Eating
                } else {
                    Self::Seated
                }
            }
        }
    }


    /// Short words for a floor tile and the colour key (the tile's colour
    /// carries the rest; colours live in the floor's own design).
    pub fn tile_key(self) -> &'static str {
        match self {
            Self::Free => "tile_free",
            Self::Seated => "tile_occupied",
            Self::InKitchen => "floor_in_kitchen",
            Self::FoodReady => "tile_food_ready",
            Self::Eating => "floor_eating",
            Self::BillPrinted => "tile_bill_printed",
            Self::Paid => "tile_paid",
        }
    }

    pub const ALL: [FloorState; 7] = [
        Self::Free,
        Self::Seated,
        Self::InKitchen,
        Self::FoodReady,
        Self::Eating,
        Self::BillPrinted,
        Self::Paid,
    ];
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

    /// Undo one bump: Completed→Ready→Preparing→New. Cancelled stays.
    pub fn prev(self) -> Option<Self> {
        match self {
            Self::Completed => Some(Self::Ready),
            Self::Ready => Some(Self::Preparing),
            Self::Preparing => Some(Self::New),
            Self::New | Self::Cancelled => None,
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
    /// Table this ticket belongs to (empty for counter orders).
    #[serde(default)]
    pub table_id: String,
    /// Order amount at send time (paise) — kitchen never sees this;
    /// it powers the floor card's running total.
    #[serde(default)]
    pub total_paise: i64,
    /// Kitchen station this ticket belongs to; empty = whole order / unassigned.
    #[serde(default)]
    pub station_id: String,
    /// Station display name resolved for the KDS (empty when unassigned).
    #[serde(default)]
    pub station_name: String,
    /// Today's ticket number (the kitchen calls tickets by number).
    #[serde(default)]
    pub kot_no: i64,
    /// `kot` (cook this) or `cancel` (stop this — a cancellation slip).
    #[serde(default)]
    pub kind: String,
    /// Who sent it (waiter / cashier name).
    #[serde(default)]
    pub created_by: String,
}

impl KotTicket {
    pub fn is_cancel(&self) -> bool {
        self.kind == "cancel"
    }
}

/// Where a table's open order stands (vision `BUSINESS_BOOKS.md` § Open
/// orders are not books yet).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunningOrderStatus {
    /// Items going to the kitchen; more can be added.
    #[default]
    Running,
    /// The bill was printed for the guests; still open until settled.
    Printed,
    /// Paid — closed by a sale.
    Settled,
    /// Folded into another table's order (merge).
    Merged,
}

impl RunningOrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Printed => "printed",
            Self::Settled => "settled",
            Self::Merged => "merged",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "printed" => Self::Printed,
            "settled" => Self::Settled,
            "merged" => Self::Merged,
            _ => Self::Running,
        }
    }

    pub fn is_open(self) -> bool {
        matches!(self, Self::Running | Self::Printed)
    }
}

/// One line of a table's open order: what the kitchen was told, kept even
/// when cancelled or given free (vision §18).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RunningLine {
    pub id: String,
    pub order_id: String,
    /// KOT that carried this line to the kitchen.
    pub kot_id: String,
    pub product_id: String,
    pub name: String,
    pub unit_price_paise: i64,
    pub qty_milli: i64,
    #[serde(default)]
    pub unit_label: String,
    #[serde(default)]
    pub note: String,
    /// Cancelled after the KOT: not billed, never deleted.
    #[serde(default)]
    pub cancelled: bool,
    /// Complimentary (NC): billed ₹0, real value kept for reports.
    #[serde(default)]
    pub complimentary: bool,
    /// Why it was cancelled / given free.
    #[serde(default)]
    pub reason: String,
    /// Cancelled after the kitchen had cooked it (wastage) vs before.
    #[serde(default)]
    pub prepared: bool,
    /// Who changed it (cancel / NC) and who approved, by name.
    #[serde(default)]
    pub changed_by: String,
    #[serde(default)]
    pub approved_by: String,
    pub created_at: String,
    #[serde(default)]
    pub created_by: String,
    /// Seat number for split-by-seat (0 = unassigned).
    #[serde(default)]
    pub seat_no: i64,
    /// Coursing hold: Send KOT skips until Fire course / unhold.
    #[serde(default)]
    pub held: bool,
    /// Course number for fire-by-course (0 = none).
    #[serde(default)]
    pub course_no: i64,
}

impl RunningLine {
    /// Value at the menu price — what the line would cost if charged.
    pub fn value_paise(&self) -> i64 {
        Money(self.unit_price_paise).saturating_mul_milli(self.qty_milli).paise()
    }

    /// What this line adds to the bill: ₹0 when cancelled or complimentary.
    pub fn billed_paise(&self) -> i64 {
        if self.cancelled || self.complimentary {
            0
        } else {
            self.value_paise()
        }
    }

    pub fn is_live(&self) -> bool {
        !self.cancelled
    }
}

/// A table's open order between KOTs and payment. One per seating.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RunningOrder {
    pub id: String,
    pub table_id: String,
    pub status: RunningOrderStatus,
    pub opened_at: String,
    #[serde(default)]
    pub opened_by: String,
    #[serde(default)]
    pub printed_at: String,
    #[serde(default)]
    pub lines: Vec<RunningLine>,
}

impl RunningOrder {
    pub fn live_lines(&self) -> impl Iterator<Item = &RunningLine> {
        self.lines.iter().filter(|l| l.is_live())
    }

    /// Running amount for the floor card and the bill.
    pub fn total_paise(&self) -> i64 {
        self.lines.iter().map(RunningLine::billed_paise).sum()
    }

    /// The order as cart lines to bill: every live line, marked sent, NC at
    /// ₹0. Cancelled lines stay on the order, off the bill.
    pub fn to_cart_lines(&self) -> Vec<CartLine> {
        self.live_lines()
            .map(|l| {
                let mut line = CartLine {
                    product_id: l.product_id.clone(),
                    name: l.name.clone(),
                    unit_price: Money(l.unit_price_paise),
                    qty: 0,
                    qty_milli: l.qty_milli,
                    unit_label: l.unit_label.clone(),
                    note: l.note.clone(),
                    base_unit_paise: l.unit_price_paise,
                    applied_modifiers: Vec::new(),
                    modifier_page: 0,
                    line_total_override: None,
                    running_line_id: l.id.clone(),
                    complimentary: l.complimentary,
                    kot_id: l.kot_id.clone(),
                    hold_send: false,
                };
                line.sync_qty_from_milli();
                line
            })
            .collect()
    }
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

    /// The menu window's − takes back what is still to go to the kitchen;
    /// what the kitchen already has needs a reasoned cancel.
    #[test]
    fn minus_takes_back_only_what_the_kitchen_has_not_got() {
        let mut cart = Cart::default();
        cart.add_or_bump_in_stock(&tea()).unwrap();
        cart.add_or_bump_in_stock(&tea()).unwrap();
        cart.remove_one_of("tea").unwrap();
        assert_eq!(cart.qty_of("tea"), 1);
        cart.remove_one_of("tea").unwrap();
        assert!(cart.lines.is_empty(), "the line leaves at nought");
        assert!(cart.remove_one_of("tea").is_ok(), "nothing to take back is not an error");
        // Two already in the kitchen, one more punched since.
        cart.add_or_bump_in_stock(&tea()).unwrap();
        cart.add_or_bump_in_stock(&tea()).unwrap();
        cart.lines[0].kot_id = "sent".into();
        cart.add_or_bump_in_stock(&tea()).unwrap();
        assert_eq!(cart.qty_of("tea"), 3);
        cart.remove_one_of("tea").unwrap();
        assert_eq!(cart.qty_of("tea"), 2);
        let refused = cart.remove_one_of("tea").unwrap_err();
        assert!(refused.contains("kitchen"), "{refused}");
        assert_eq!(cart.qty_of("tea"), 2);
    }

    /// ₹0 is a real menu price (complimentary items, price-at-counter dishes);
    /// only a negative price is refused. The till agrees with the form.
    #[test]
    fn menu_items_may_be_free_but_never_negative() {
        assert_eq!(menu_price_error(Money::from_rupees(13)), None);
        assert_eq!(menu_price_error(Money(0)), None);
        let negative = menu_price_error(Money(-500)).expect("negative is refused");
        assert!(negative.contains("negative"), "{negative}");
        // A ₹0 item goes on the bill, and its @ can still be set at the counter.
        let mut cart = Cart::default();
        let free = Product::new_basic("free", "Water", Money(0));
        cart.add_or_bump_in_stock(&free).expect("₹0 item is billable");
        assert_eq!(cart.lines.len(), 1);
        assert_eq!(cart.applied_total(), Money(0));
        cart.set_unit_price(0, Money::from_rupees(20)).unwrap();
        assert_eq!(cart.applied_total(), Money::from_rupees(20));
        assert!(cart.set_unit_price(0, Money(-1)).is_err());
        // Free line next to a paid one: the bill is the paid part.
        let mut cart = Cart::default();
        cart.add_or_bump_in_stock(&free).unwrap();
        cart.add_or_bump_in_stock(&tea()).unwrap();
        let draft = cart.to_sale_draft().expect("bill with a free line");
        assert_eq!(draft.total_paise, Money::from_rupees(13).paise());
        assert_eq!(draft.lines[0].line_total_paise, 0);
        // Entire bill at ₹0 (complimentary cover) is a valid draft.
        let mut free_only = Cart::default();
        free_only.add_or_bump_in_stock(&free).unwrap();
        let zero = free_only.to_sale_draft().expect("₹0 bill draft");
        assert_eq!(zero.total_paise, 0);
        zero.assert_balanced().unwrap();
        free_only.set_charge_total(Money(0)).unwrap();
        assert!(free_only.charge_total.is_none() || free_only.total().paise() == 0);
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
                group: String::new(),
            },
            ModifierOption {
                name: "No onion".into(),
                price_paise: 0,
                group: String::new(),
            },
        ];
        let mut p = tea();
        p.set_modifiers(&mods);
        let got = p.modifiers();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].name, "Extra cheese");
        assert_eq!(got[0].price_paise, 2000);
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
            group: String::new(),
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
    fn variation_group_pick_one_and_addons_stack() {
        let groups = ModifierGroup::parse_human_list(
            "Size*: Small=100, Large=200 | Toppings+2: Cheese=40, Olive=30, Jalapeno=20",
            "INR",
        )
        .unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups[0].is_variation());
        assert_eq!(groups[0].options.len(), 2);
        assert!(!groups[1].is_variation());
        assert_eq!(groups[1].max, 2);
        assert_eq!(groups[1].max_allowed(), 2);

        let mut pizza = Product::new_basic("pizza", "Pizza", Money(0));
        pizza.set_modifier_groups(&groups);
        let mut cart = Cart::default();
        cart.add_or_bump(&pizza);
        let small = ModifierOption {
            name: "Small".into(),
            price_paise: 10_000,
            group: "Size".into(),
        };
        let large = ModifierOption {
            name: "Large".into(),
            price_paise: 20_000,
            group: "Size".into(),
        };
        let cheese = ModifierOption {
            name: "Cheese".into(),
            price_paise: 4_000,
            group: "Toppings".into(),
        };
        assert!(cart
            .toggle_modifier_in_groups(0, &small, &groups)
            .unwrap());
        assert_eq!(cart.lines[0].unit_price.paise(), 10_000);
        assert!(cart
            .toggle_modifier_in_groups(0, &large, &groups)
            .unwrap());
        assert_eq!(cart.lines[0].applied_modifiers.len(), 1);
        assert_eq!(cart.lines[0].unit_price.paise(), 20_000);
        assert!(cart
            .toggle_modifier_in_groups(0, &cheese, &groups)
            .unwrap());
        assert_eq!(cart.lines[0].unit_price.paise(), 24_000);
        let olive = ModifierOption {
            name: "Olive".into(),
            price_paise: 3_000,
            group: "Toppings".into(),
        };
        let jal = ModifierOption {
            name: "Jalapeno".into(),
            price_paise: 2_000,
            group: "Toppings".into(),
        };
        assert!(cart
            .toggle_modifier_in_groups(0, &olive, &groups)
            .unwrap());
        assert!(cart
            .toggle_modifier_in_groups(0, &jal, &groups)
            .unwrap_err()
            .contains("at most 2"));
        assert_eq!(
            cart.to_sale_draft().unwrap().lines[0].note,
            "Size Large, Toppings Cheese, Toppings Olive"
        );
        let draft = cart.to_sale_draft().unwrap();
        assert_eq!(draft.lines[0].modifiers().len(), 3);
        assert!(cart
            .missing_required_modifiers(|id| {
                if id == "pizza" {
                    groups.clone()
                } else {
                    Vec::new()
                }
            })
            .is_empty());
        cart.lines[0].applied_modifiers.clear();
        cart.lines[0].reprice_from_modifiers();
        let missing = cart.missing_required_modifiers(|_| groups.clone());
        assert!(missing.iter().any(|m| m.contains("Size")), "{missing:?}");
        let human = ModifierGroup::format_human_list(&groups, "INR");
        assert!(human.contains("Size*"));
        assert!(human.contains("Toppings+2"), "{human}");
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
            picker_enabled: true,
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

    #[test]
    fn party_roles_normalize_merge_and_both() {
        assert_eq!(normalize_party_roles("both"), "Customer, Supplier");
        assert_eq!(
            merge_party_roles("Customer", "Supplier"),
            "Customer, Supplier"
        );
        assert!(party_has_role("Customer, Supplier", PARTY_ROLE_CUSTOMER));
        assert!(party_has_role("Customer, Supplier", PARTY_ROLE_SUPPLIER));
        let c = Customer {
            id: "1".into(),
            name: "Peer".into(),
            phone: String::new(),
            note: String::new(),
            active: true,
            balance_paise: 0,
            kind_label: "Customer, Supplier".into(),
            credit_limit_paise: 0,
            loyalty_points: 0,
            wallet_paise: 0,
            birthday: String::new(),
        };
        assert!(c.is_customer() && c.is_supplier());
    }

    #[test]
    fn kot_age_band_default_target_fifteen() {
        // target 15 → amber at 10, red at 20
        assert_eq!(kot_age_band(0, 15), KotAgeBand::Normal);
        assert_eq!(kot_age_band(9, 15), KotAgeBand::Normal);
        assert_eq!(kot_age_band(10, 15), KotAgeBand::Amber);
        assert_eq!(kot_age_band(19, 15), KotAgeBand::Amber);
        assert_eq!(kot_age_band(20, 15), KotAgeBand::Red);
    }

    #[test]
    fn happy_hour_price_and_breakfast_visibility() {
        use chrono::TimeZone;
        let mut p = Product::new_basic("idli", "Idli", Money::from_rupees(40));
        p.set_schedule(&[
            ScheduleWindow {
                days: vec![1, 2, 3, 4, 5],
                start: "07:00".into(),
                end: "11:00".into(),
                price_paise: None,
            },
            ScheduleWindow {
                days: Vec::new(),
                start: "17:00".into(),
                end: "19:00".into(),
                price_paise: Some(15_000),
            },
        ]);
        // Breakfast visibility only Mon–Fri 07–11; outside → hidden.
        let tue_breakfast = chrono::Local
            .with_ymd_and_hms(2026, 10, 27, 8, 0, 0)
            .unwrap(); // Tuesday
        assert!(p.available_now(tue_breakfast));
        let tue_afternoon = chrono::Local
            .with_ymd_and_hms(2026, 10, 27, 14, 0, 0)
            .unwrap();
        assert!(!p.available_now(tue_afternoon));
        // Happy hour price (no visibility window for evenings — but visibility
        // windows alone gate availability, so evenings are hidden). Split:
        // only priced window → always available with override in window.
        let mut happy = Product::new_basic("beer", "Beer", Money::from_rupees(200));
        happy.set_schedule(&[ScheduleWindow {
            days: Vec::new(),
            start: "17:00".into(),
            end: "19:00".into(),
            price_paise: Some(15_000),
        }]);
        let evening = chrono::Local
            .with_ymd_and_hms(2026, 10, 27, 18, 0, 0)
            .unwrap();
        assert!(happy.available_now(evening));
        assert_eq!(happy.price_now("", evening).paise(), 15_000);
        let noon = chrono::Local
            .with_ymd_and_hms(2026, 10, 27, 12, 0, 0)
            .unwrap();
        assert_eq!(happy.price_now("", noon).paise(), 20_000);
        // Human parse
        let windows =
            ScheduleWindow::parse_human_list("Mon-Fri 07:00-11:00, HappyHour 17:00-19:00=150", "INR")
                .unwrap();
        assert_eq!(windows.len(), 2);
        assert!(windows[0].price_paise.is_none());
        assert_eq!(windows[1].price_paise, Some(15_000));
    }

    #[test]
    fn kot_status_prev_recall_chain() {
        assert_eq!(KotStatus::Completed.prev(), Some(KotStatus::Ready));
        assert_eq!(KotStatus::Ready.prev(), Some(KotStatus::Preparing));
        assert_eq!(KotStatus::Preparing.prev(), Some(KotStatus::New));
        assert_eq!(KotStatus::New.prev(), None);
        assert_eq!(KotStatus::Cancelled.prev(), None);
    }
}


