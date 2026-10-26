//! Country fiscal packs: tax system, pay-rail wording, and business-id labels.
//!
//! A pack is **seller-facing wording plus arithmetic defaults**, not a
//! statutory filing engine. Kumkaj does not produce government e-invoices,
//! fiscal-printer payloads or tax returns anywhere in the world; sellers stay
//! responsible for the rates they enter and for their own filings.
//!
//! Two pricing models cover every country we ship:
//!
//! * [`TaxPricing::Inclusive`] — the menu price already contains the tax
//!   (India, UK hospitality, most of the EU, Australia, Singapore, Japan …).
//!   The bill total never moves; the tax portion is *extracted* for the
//!   receipt and reports.
//! * [`TaxPricing::Exclusive`] — the menu price is net and tax is *added* at
//!   the till (US sales tax, Gulf VAT in B2B, Mexico, Brazil …). The charged
//!   total is `lines + rounding + tax`.

use serde::{Deserialize, Serialize};

/// Whether quoted item prices already contain tax.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxPricing {
    #[default]
    Inclusive,
    Exclusive,
}

impl TaxPricing {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inclusive => "inclusive",
            Self::Exclusive => "exclusive",
        }
    }

    /// Anything unrecognised reads as Inclusive — the historical Kumkaj
    /// behaviour, so legacy rows and bad input never inflate a bill.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "exclusive" | "excl" | "added" | "on_top" => Self::Exclusive,
            _ => Self::Inclusive,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Inclusive => "Tax inside price",
            Self::Exclusive => "Tax added at till",
        }
    }

    pub fn is_inclusive(self) -> bool {
        matches!(self, Self::Inclusive)
    }
}

/// Family of consumption tax, for wording and future per-family behaviour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxSystemKind {
    #[default]
    None,
    Vat,
    Gst,
    SalesTax,
    Other,
}

impl TaxSystemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Vat => "vat",
            Self::Gst => "gst",
            Self::SalesTax => "sales_tax",
            Self::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "vat" => Self::Vat,
            "gst" => Self::Gst,
            "sales_tax" | "sales tax" => Self::SalesTax,
            "other" => Self::Other,
            _ => Self::None,
        }
    }
}

/// One rate a country ships out of the box (`Standard 20%` → bps 2000).
#[derive(Clone, Copy, Debug)]
pub struct TaxRateSeed {
    pub name: &'static str,
    pub code: &'static str,
    pub bps: i64,
}

const fn seed(name: &'static str, code: &'static str, bps: i64) -> TaxRateSeed {
    TaxRateSeed { name, code, bps }
}

/// Everything a country changes in Kumkaj: tax wording and maths, the local
/// instant-payment rail, the business-id label on receipts, and date order.
#[derive(Clone, Copy, Debug)]
pub struct CountryProfile {
    /// ISO 3166-1 alpha-2, or `XX` for the neutral custom pack.
    pub code: &'static str,
    pub name_en: &'static str,
    pub default_currency: &'static str,
    pub default_language: &'static str,
    /// Decimal mark used when writing money: `.` in India and the US, `,`
    /// across most of Europe and Latin America.
    pub decimal_sep: &'static str,
    /// Group mark between thousands: `,`, `.`, a space, an apostrophe in
    /// Switzerland, or empty for countries that do not group at all.
    pub thousands_sep: &'static str,
    pub tax_system: TaxSystemKind,
    /// Seller-facing name of the tax: `GST`, `VAT`, `IVA`, `BTW`, `SST`…
    pub tax_name: &'static str,
    pub tax_pricing: TaxPricing,
    pub default_rates: &'static [TaxRateSeed],
    /// Optional equal-split *display* of the tax on receipts (India shows
    /// CGST/SGST halves). Fractions in bps of the **tax amount**, summing to
    /// 10000. Empty = show one tax line.
    pub tax_split: &'static [(&'static str, i64)],
    /// Storage value stays `upi` so [`crate::domain::PayMethod`] and every
    /// historical sale row keep working; only the label is localized.
    pub digital_pay_code: &'static str,
    pub digital_pay_label: &'static str,
    pub business_id_label: &'static str,
    pub credit_label: &'static str,
    /// `dmy` | `mdy` | `ymd` — receipt date order.
    pub date_order: &'static str,
}

impl CountryProfile {
    pub fn has_tax(&self) -> bool {
        !matches!(self.tax_system, TaxSystemKind::None)
    }

    /// What applying this pack will change, as (what, value) pairs the
    /// country picker shows one per row.
    pub fn apply_facts(&self) -> Vec<(&'static str, String)> {
        let tax = if self.has_tax() {
            format!("{} · {}", self.tax_name, self.tax_pricing.label())
        } else {
            "No consumption tax by default".to_string()
        };
        let mut facts = vec![("Country", format!("{} ({})", self.name_en, self.code)), ("Tax", tax)];
        if self.code != GENERIC_CODE {
            facts.push(("Business id", self.business_id_label.to_string()));
        }
        facts.push(("Pay rail", self.digital_pay_label.to_string()));
        if self.code != GENERIC_CODE {
            let language = crate::locale::language_by_code(self.default_language)
                .map(|l| l.name_en)
                .unwrap_or(self.default_language);
            facts.push(("Currency", self.default_currency.to_string()));
            facts.push(("Language", language.to_string()));
        }
        facts
    }
}

/// Neutral pack for sellers whose country is not listed (or who want to keep
/// their own currency/language while naming the tax themselves).
pub const GENERIC_CODE: &str = "XX";

const EU_STANDARD_SPLIT: &[(&str, i64)] = &[];
const NO_SPLIT: &[(&str, i64)] = &[];
const NO_RATES: &[TaxRateSeed] = &[];

/// India shows the same GST as two equal halves on the bill.
const IN_SPLIT: &[(&str, i64)] = &[("CGST", 5_000), ("SGST", 5_000)];

const IN_RATES: &[TaxRateSeed] = &[
    seed("Exempt", "0", 0),
    seed("GST 5%", "5", 500),
    seed("GST 12%", "12", 1_200),
    seed("GST 18%", "18", 1_800),
    seed("GST 28%", "28", 2_800),
];

const GB_RATES: &[TaxRateSeed] = &[
    seed("Zero rated", "0", 0),
    seed("Reduced 5%", "5", 500),
    seed("Standard 20%", "20", 2_000),
];

/// Broad world coverage. Rates are **starting points a seller can edit** —
/// never a claim that they are current or complete for any jurisdiction.
pub const COUNTRY_PROFILES: &[CountryProfile] = &[
    // ————————————————————————————— South Asia —————————————————————————————
    CountryProfile {
        code: "IN",
        name_en: "India",
        default_currency: "INR",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Gst,
        tax_name: "GST",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: IN_RATES,
        tax_split: IN_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "UPI",
        business_id_label: "GSTIN",
        // Stays "Credit" so the default pack changes nothing for shops that
        // were already selling; Hindi and the other Indian language tables
        // already translate it to उधार.
        credit_label: "Credit",
        date_order: "dmy",
    },
    CountryProfile {
        code: "PK",
        name_en: "Pakistan",
        default_currency: "PKR",
        default_language: "ur",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::SalesTax,
        tax_name: "Sales tax",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Exempt", "0", 0), seed("Sales tax 18%", "18", 1_800)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Raast",
        business_id_label: "NTN",
        credit_label: "Udhaar",
        date_order: "dmy",
    },
    CountryProfile {
        code: "BD",
        name_en: "Bangladesh",
        default_currency: "BDT",
        default_language: "bn",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Exempt", "0", 0), seed("VAT 15%", "15", 1_500)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "bKash",
        business_id_label: "BIN",
        credit_label: "Baki",
        date_order: "dmy",
    },
    CountryProfile {
        code: "LK",
        name_en: "Sri Lanka",
        default_currency: "LKR",
        default_language: "si",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Exempt", "0", 0), seed("VAT 18%", "18", 1_800)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "LankaQR",
        business_id_label: "VAT number",
        credit_label: "Credit",
        date_order: "ymd",
    },
    CountryProfile {
        code: "NP",
        name_en: "Nepal",
        default_currency: "NPR",
        default_language: "ne",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Exempt", "0", 0), seed("VAT 13%", "13", 1_300)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Fonepay",
        business_id_label: "PAN",
        credit_label: "Credit",
        date_order: "ymd",
    },
    // ———————————————————————— Middle East / Gulf ————————————————————————
    CountryProfile {
        code: "AE",
        name_en: "United Arab Emirates",
        default_currency: "AED",
        default_language: "ar",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 5%", "5", 500)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "TRN",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "SA",
        name_en: "Saudi Arabia",
        default_currency: "SAR",
        default_language: "ar",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 15%", "15", 1_500)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "VAT number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "QA",
        name_en: "Qatar",
        default_currency: "QAR",
        default_language: "ar",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::None,
        tax_name: "None",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: NO_RATES,
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "Tax card number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "KW",
        name_en: "Kuwait",
        default_currency: "KWD",
        default_language: "ar",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::None,
        tax_name: "None",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: NO_RATES,
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "None",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "OM",
        name_en: "Oman",
        default_currency: "OMR",
        default_language: "ar",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 5%", "5", 500)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "VATIN",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "BH",
        name_en: "Bahrain",
        default_currency: "BHD",
        default_language: "ar",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 10%", "10", 1_000)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "BenefitPay",
        business_id_label: "VAT number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "IL",
        name_en: "Israel",
        default_currency: "ILS",
        default_language: "he",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 18%", "18", 1_800)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bit",
        business_id_label: "Tax ID",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "TR",
        name_en: "Türkiye",
        default_currency: "TRY",
        default_language: "tr",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "KDV",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("KDV 1%", "1", 100),
            seed("KDV 10%", "10", 1_000),
            seed("KDV 20%", "20", 2_000),
        ],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "FAST",
        business_id_label: "Vergi No",
        credit_label: "Veresiye",
        date_order: "dmy",
    },
    // —————————————————————————————— Europe ——————————————————————————————
    CountryProfile {
        code: "GB",
        name_en: "United Kingdom",
        default_currency: "GBP",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: GB_RATES,
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bank transfer",
        business_id_label: "VAT number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "IE",
        name_en: "Ireland",
        default_currency: "EUR",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("Zero rated", "0", 0),
            seed("VAT 9%", "9", 900),
            seed("VAT 13.5%", "13.5", 1_350),
            seed("VAT 23%", "23", 2_300),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bank transfer",
        business_id_label: "VAT number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "DE",
        name_en: "Germany",
        default_currency: "EUR",
        default_language: "de",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "MwSt",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("MwSt 7%", "7", 700), seed("MwSt 19%", "19", 1_900)],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bank transfer",
        business_id_label: "USt-IdNr",
        credit_label: "Auf Rechnung",
        date_order: "dmy",
    },
    CountryProfile {
        code: "FR",
        name_en: "France",
        default_currency: "EUR",
        default_language: "fr",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "TVA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("TVA 5.5%", "5.5", 550),
            seed("TVA 10%", "10", 1_000),
            seed("TVA 20%", "20", 2_000),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Virement",
        business_id_label: "No TVA",
        credit_label: "Sur compte",
        date_order: "dmy",
    },
    CountryProfile {
        code: "ES",
        name_en: "Spain",
        default_currency: "EUR",
        default_language: "es",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "IVA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("IVA 4%", "4", 400),
            seed("IVA 10%", "10", 1_000),
            seed("IVA 21%", "21", 2_100),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bizum",
        business_id_label: "NIF",
        credit_label: "A cuenta",
        date_order: "dmy",
    },
    CountryProfile {
        code: "IT",
        name_en: "Italy",
        default_currency: "EUR",
        default_language: "it",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "IVA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("IVA 4%", "4", 400),
            seed("IVA 10%", "10", 1_000),
            seed("IVA 22%", "22", 2_200),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bonifico",
        business_id_label: "Partita IVA",
        credit_label: "A credito",
        date_order: "dmy",
    },
    CountryProfile {
        code: "NL",
        name_en: "Netherlands",
        default_currency: "EUR",
        default_language: "nl",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "BTW",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("BTW 9%", "9", 900), seed("BTW 21%", "21", 2_100)],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "iDEAL",
        business_id_label: "BTW-nummer",
        credit_label: "Op rekening",
        date_order: "dmy",
    },
    CountryProfile {
        code: "BE",
        name_en: "Belgium",
        default_currency: "EUR",
        default_language: "nl",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "BTW",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("BTW 6%", "6", 600),
            seed("BTW 12%", "12", 1_200),
            seed("BTW 21%", "21", 2_100),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Payconiq",
        business_id_label: "BTW-nummer",
        credit_label: "Op rekening",
        date_order: "dmy",
    },
    CountryProfile {
        code: "PT",
        name_en: "Portugal",
        default_currency: "EUR",
        default_language: "pt",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "IVA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("IVA 6%", "6", 600),
            seed("IVA 13%", "13", 1_300),
            seed("IVA 23%", "23", 2_300),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "MB WAY",
        business_id_label: "NIF",
        credit_label: "Em conta",
        date_order: "dmy",
    },
    CountryProfile {
        code: "PL",
        name_en: "Poland",
        default_currency: "PLN",
        default_language: "pl",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("VAT 5%", "5", 500),
            seed("VAT 8%", "8", 800),
            seed("VAT 23%", "23", 2_300),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "BLIK",
        business_id_label: "NIP",
        credit_label: "Na rachunek",
        date_order: "dmy",
    },
    CountryProfile {
        code: "SE",
        name_en: "Sweden",
        default_currency: "SEK",
        default_language: "sv",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "Moms",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("Moms 6%", "6", 600),
            seed("Moms 12%", "12", 1_200),
            seed("Moms 25%", "25", 2_500),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Swish",
        business_id_label: "Momsnummer",
        credit_label: "Mot faktura",
        date_order: "ymd",
    },
    CountryProfile {
        code: "NO",
        name_en: "Norway",
        default_currency: "NOK",
        default_language: "no",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "MVA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("MVA 15%", "15", 1_500), seed("MVA 25%", "25", 2_500)],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Vipps",
        business_id_label: "MVA-nummer",
        credit_label: "På regning",
        date_order: "dmy",
    },
    CountryProfile {
        code: "DK",
        name_en: "Denmark",
        default_currency: "DKK",
        default_language: "da",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "Moms",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Moms 25%", "25", 2_500)],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "MobilePay",
        business_id_label: "CVR-nummer",
        credit_label: "På regning",
        date_order: "dmy",
    },
    CountryProfile {
        code: "FI",
        name_en: "Finland",
        default_currency: "EUR",
        default_language: "fi",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "ALV",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("ALV 10%", "10", 1_000),
            seed("ALV 14%", "14", 1_400),
            seed("ALV 25.5%", "25.5", 2_550),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "MobilePay",
        business_id_label: "ALV-numero",
        credit_label: "Laskulle",
        date_order: "dmy",
    },
    CountryProfile {
        code: "AT",
        name_en: "Austria",
        default_currency: "EUR",
        default_language: "de",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "MwSt",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("MwSt 10%", "10", 1_000),
            seed("MwSt 13%", "13", 1_300),
            seed("MwSt 20%", "20", 2_000),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bank transfer",
        business_id_label: "UID-Nummer",
        credit_label: "Auf Rechnung",
        date_order: "dmy",
    },
    CountryProfile {
        code: "GR",
        name_en: "Greece",
        default_currency: "EUR",
        default_language: "el",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("VAT 6%", "6", 600),
            seed("VAT 13%", "13", 1_300),
            seed("VAT 24%", "24", 2_400),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "IRIS",
        business_id_label: "AFM",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "CZ",
        name_en: "Czechia",
        default_currency: "CZK",
        default_language: "cs",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "DPH",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("DPH 12%", "12", 1_200), seed("DPH 21%", "21", 2_100)],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "QR platba",
        business_id_label: "DIC",
        credit_label: "Na fakturu",
        date_order: "dmy",
    },
    CountryProfile {
        code: "HU",
        name_en: "Hungary",
        default_currency: "HUF",
        default_language: "hu",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "AFA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("AFA 5%", "5", 500),
            seed("AFA 18%", "18", 1_800),
            seed("AFA 27%", "27", 2_700),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Instant transfer",
        business_id_label: "Adoszam",
        credit_label: "Szamlara",
        date_order: "ymd",
    },
    CountryProfile {
        code: "RO",
        name_en: "Romania",
        default_currency: "RON",
        default_language: "ro",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "TVA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("TVA 11%", "11", 1_100), seed("TVA 21%", "21", 2_100)],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bank transfer",
        business_id_label: "CUI",
        credit_label: "Pe cont",
        date_order: "dmy",
    },
    CountryProfile {
        code: "CH",
        name_en: "Switzerland",
        default_currency: "CHF",
        default_language: "de",
        decimal_sep: ".",
        thousands_sep: "'",
        tax_system: TaxSystemKind::Vat,
        tax_name: "MWST",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("MWST 2.6%", "2.6", 260),
            seed("MWST 3.8%", "3.8", 380),
            seed("MWST 8.1%", "8.1", 810),
        ],
        tax_split: EU_STANDARD_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "TWINT",
        business_id_label: "UID",
        credit_label: "Auf Rechnung",
        date_order: "dmy",
    },
    CountryProfile {
        code: "RU",
        name_en: "Russia",
        default_currency: "RUB",
        default_language: "ru",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("VAT 10%", "10", 1_000), seed("VAT 20%", "20", 2_000)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "SBP",
        business_id_label: "INN",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "UA",
        name_en: "Ukraine",
        default_currency: "UAH",
        default_language: "uk",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("VAT 7%", "7", 700), seed("VAT 20%", "20", 2_000)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bank transfer",
        business_id_label: "Tax number",
        credit_label: "On account",
        date_order: "dmy",
    },
    // ————————————————————————————— Americas —————————————————————————————
    CountryProfile {
        code: "US",
        name_en: "United States",
        default_currency: "USD",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::SalesTax,
        tax_name: "Sales tax",
        // State + county + city rates differ street by street, so Kumkaj
        // ships no rate at all: the seller enters the one they collect.
        tax_pricing: TaxPricing::Exclusive,
        default_rates: NO_RATES,
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "EIN",
        credit_label: "Tab",
        date_order: "mdy",
    },
    CountryProfile {
        code: "CA",
        name_en: "Canada",
        default_currency: "CAD",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Gst,
        tax_name: "GST/HST",
        // Provinces stack PST/QST on top of GST — seed the federal rate and
        // the common HST blends; the seller picks what their province charges.
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[
            seed("Zero rated", "0", 0),
            seed("GST 5%", "5", 500),
            seed("HST 13%", "13", 1_300),
            seed("HST 15%", "15", 1_500),
        ],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Interac",
        business_id_label: "GST/HST number",
        credit_label: "On account",
        date_order: "ymd",
    },
    CountryProfile {
        code: "MX",
        name_en: "Mexico",
        default_currency: "MXN",
        default_language: "es",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "IVA",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Tasa 0%", "0", 0), seed("IVA 16%", "16", 1_600)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "SPEI",
        business_id_label: "RFC",
        credit_label: "A cuenta",
        date_order: "dmy",
    },
    CountryProfile {
        code: "BR",
        name_en: "Brazil",
        default_currency: "BRL",
        default_language: "pt-BR",
        decimal_sep: ",",
        thousands_sep: ".",
        // Brazilian indirect tax (ICMS/ISS/PIS/COFINS) is layered per state and
        // per service. Kumkaj keeps one simplified seller-set rate.
        tax_system: TaxSystemKind::Other,
        tax_name: "Imposto",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: NO_RATES,
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Pix",
        business_id_label: "CNPJ",
        credit_label: "Fiado",
        date_order: "dmy",
    },
    CountryProfile {
        code: "AR",
        name_en: "Argentina",
        default_currency: "ARS",
        default_language: "es",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "IVA",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("IVA 10.5%", "10.5", 1_050), seed("IVA 21%", "21", 2_100)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Transferencia 3.0",
        business_id_label: "CUIT",
        credit_label: "A cuenta",
        date_order: "dmy",
    },
    CountryProfile {
        code: "CL",
        name_en: "Chile",
        default_currency: "CLP",
        default_language: "es",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "IVA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("IVA 19%", "19", 1_900)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Transferencia",
        business_id_label: "RUT",
        credit_label: "A cuenta",
        date_order: "dmy",
    },
    CountryProfile {
        code: "CO",
        name_en: "Colombia",
        default_currency: "COP",
        default_language: "es",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "IVA",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("IVA 5%", "5", 500), seed("IVA 19%", "19", 1_900)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bre-B",
        business_id_label: "NIT",
        credit_label: "A cuenta",
        date_order: "dmy",
    },
    CountryProfile {
        code: "PE",
        name_en: "Peru",
        default_currency: "PEN",
        default_language: "es",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "IGV",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("IGV 18%", "18", 1_800)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Yape",
        business_id_label: "RUC",
        credit_label: "A cuenta",
        date_order: "dmy",
    },
    // ———————————————————————— Asia Pacific / Oceania ————————————————————————
    CountryProfile {
        code: "AU",
        name_en: "Australia",
        default_currency: "AUD",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Gst,
        tax_name: "GST",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("GST free", "0", 0), seed("GST 10%", "10", 1_000)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "PayID",
        business_id_label: "ABN",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "NZ",
        name_en: "New Zealand",
        default_currency: "NZD",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Gst,
        tax_name: "GST",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("GST 15%", "15", 1_500)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bank transfer",
        business_id_label: "GST number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "SG",
        name_en: "Singapore",
        default_currency: "SGD",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Gst,
        tax_name: "GST",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("GST 9%", "9", 900)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "PayNow",
        business_id_label: "GST registration number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "MY",
        name_en: "Malaysia",
        default_currency: "MYR",
        default_language: "ms",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Other,
        tax_name: "SST",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[
            seed("Service tax 6%", "6", 600),
            seed("Sales tax 10%", "10", 1_000),
        ],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "DuitNow",
        business_id_label: "SST number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "TH",
        name_en: "Thailand",
        default_currency: "THB",
        default_language: "th",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 7%", "7", 700)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "PromptPay",
        business_id_label: "Tax ID",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "ID",
        name_en: "Indonesia",
        default_currency: "IDR",
        default_language: "id",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "PPN",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("PPN 11%", "11", 1_100), seed("PPN 12%", "12", 1_200)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "QRIS",
        business_id_label: "NPWP",
        credit_label: "Kasbon",
        date_order: "dmy",
    },
    CountryProfile {
        code: "PH",
        name_en: "Philippines",
        default_currency: "PHP",
        default_language: "fil",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("VAT exempt", "0", 0), seed("VAT 12%", "12", 1_200)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "InstaPay",
        business_id_label: "TIN",
        credit_label: "Utang",
        date_order: "mdy",
    },
    CountryProfile {
        code: "VN",
        name_en: "Vietnam",
        default_currency: "VND",
        default_language: "vi",
        decimal_sep: ",",
        thousands_sep: ".",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("VAT 8%", "8", 800), seed("VAT 10%", "10", 1_000)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "VietQR",
        business_id_label: "Tax code",
        credit_label: "Ghi no",
        date_order: "dmy",
    },
    CountryProfile {
        code: "JP",
        name_en: "Japan",
        default_currency: "JPY",
        default_language: "ja",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Other,
        tax_name: "Consumption tax",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[
            seed("Reduced 8%", "8", 800),
            seed("Standard 10%", "10", 1_000),
        ],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "Registration number",
        credit_label: "On account",
        date_order: "ymd",
    },
    CountryProfile {
        code: "KR",
        name_en: "South Korea",
        default_currency: "KRW",
        default_language: "ko",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 10%", "10", 1_000)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "Business number",
        credit_label: "On account",
        date_order: "ymd",
    },
    CountryProfile {
        code: "CN",
        name_en: "China",
        default_currency: "CNY",
        default_language: "zh",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("VAT 6%", "6", 600), seed("VAT 13%", "13", 1_300)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "USCC",
        credit_label: "On account",
        date_order: "ymd",
    },
    CountryProfile {
        code: "HK",
        name_en: "Hong Kong SAR",
        default_currency: "HKD",
        default_language: "zh-Hant",
        decimal_sep: ".",
        thousands_sep: ",",
        // No general consumption tax on food and drink.
        tax_system: TaxSystemKind::None,
        tax_name: "None",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: NO_RATES,
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "FPS",
        business_id_label: "BR number",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "TW",
        name_en: "Taiwan",
        default_currency: "TWD",
        default_language: "zh-Hant",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Other,
        tax_name: "Business tax",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Business tax 5%", "5", 500)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "E-wallet",
        business_id_label: "Unified business number",
        credit_label: "On account",
        date_order: "ymd",
    },
    // —————————————————————————————— Africa ——————————————————————————————
    CountryProfile {
        code: "ZA",
        name_en: "South Africa",
        default_currency: "ZAR",
        default_language: "en",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 15%", "15", 1_500)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "PayShap",
        business_id_label: "VAT number",
        credit_label: "On account",
        date_order: "ymd",
    },
    CountryProfile {
        code: "NG",
        name_en: "Nigeria",
        default_currency: "NGN",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Exempt", "0", 0), seed("VAT 7.5%", "7.5", 750)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Bank transfer",
        business_id_label: "TIN",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "KE",
        name_en: "Kenya",
        default_currency: "KES",
        default_language: "sw",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 16%", "16", 1_600)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "M-Pesa",
        business_id_label: "KRA PIN",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "GH",
        name_en: "Ghana",
        default_currency: "GHS",
        default_language: "en",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Exclusive,
        default_rates: &[seed("Exempt", "0", 0), seed("VAT 15%", "15", 1_500)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Mobile money",
        business_id_label: "TIN",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "EG",
        name_en: "Egypt",
        default_currency: "EGP",
        default_language: "ar",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Vat,
        tax_name: "VAT",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("Zero rated", "0", 0), seed("VAT 14%", "14", 1_400)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "InstaPay",
        business_id_label: "Tax ID",
        credit_label: "On account",
        date_order: "dmy",
    },
    CountryProfile {
        code: "MA",
        name_en: "Morocco",
        default_currency: "MAD",
        default_language: "ar",
        decimal_sep: ",",
        thousands_sep: " ",
        tax_system: TaxSystemKind::Vat,
        tax_name: "TVA",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: &[seed("TVA 10%", "10", 1_000), seed("TVA 20%", "20", 2_000)],
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Mobile money",
        business_id_label: "ICE",
        credit_label: "On account",
        date_order: "dmy",
    },
    // ———————————————————————————— Custom / other ————————————————————————————
    CountryProfile {
        code: GENERIC_CODE,
        name_en: "Other / custom",
        // Applying the custom pack leaves currency and language alone — the
        // seller has already chosen both in Settings.
        default_currency: "",
        default_language: "",
        decimal_sep: ".",
        thousands_sep: ",",
        tax_system: TaxSystemKind::Other,
        tax_name: "Tax",
        tax_pricing: TaxPricing::Inclusive,
        default_rates: NO_RATES,
        tax_split: NO_SPLIT,
        digital_pay_code: "upi",
        digital_pay_label: "Digital",
        business_id_label: "Tax ID",
        credit_label: "Credit",
        date_order: "dmy",
    },
];

/// Pack used until a seller picks their country. India keeps working exactly
/// as it did before country packs existed.
pub const DEFAULT_COUNTRY: &str = "IN";

pub fn countries() -> &'static [CountryProfile] {
    COUNTRY_PROFILES
}

/// Case-insensitive ISO alpha-2 lookup.
pub fn country_by_code(code: &str) -> Option<&'static CountryProfile> {
    let wanted = code.trim().to_ascii_uppercase();
    COUNTRY_PROFILES.iter().find(|c| c.code == wanted)
}

/// Lookup that always returns something — unknown codes fall back to the
/// neutral custom pack so the UI can never end up label-less.
pub fn country_or_generic(code: &str) -> &'static CountryProfile {
    country_by_code(code)
        .or_else(|| country_by_code(GENERIC_CODE))
        .unwrap_or(&COUNTRY_PROFILES[0])
}

// ———————————————————————————————— Tax maths ————————————————————————————————

/// Tax for `amount_paise` at `bps`.
///
/// * Inclusive — the portion already inside: `amount × bps / (10000 + bps)`
/// * Exclusive — the amount added on top: `amount × bps / 10000`
pub fn tax_on_amount(amount_paise: i64, bps: i64, pricing: TaxPricing) -> i64 {
    if bps <= 0 || amount_paise <= 0 {
        return 0;
    }
    match pricing {
        TaxPricing::Inclusive => amount_paise.saturating_mul(bps) / (10_000 + bps),
        TaxPricing::Exclusive => amount_paise.saturating_mul(bps) / 10_000,
    }
}

/// Taxable base. Pass the charged total under Inclusive pricing, or the
/// pre-tax line amount under Exclusive pricing — both yield the net.
pub fn net_of_tax(gross_or_net: i64, tax: i64, pricing: TaxPricing) -> i64 {
    match pricing {
        TaxPricing::Inclusive => gross_or_net.saturating_sub(tax),
        TaxPricing::Exclusive => gross_or_net,
    }
}

/// Amount actually collected: Inclusive already contains the tax, Exclusive
/// adds it on top.
pub fn gross_with_tax(lines_total: i64, tax: i64, pricing: TaxPricing) -> i64 {
    match pricing {
        TaxPricing::Inclusive => lines_total,
        TaxPricing::Exclusive => lines_total.saturating_add(tax),
    }
}

/// Split one tax amount into the receipt components a country displays
/// (India: CGST + SGST). Any rounding remainder lands on the last component so
/// the parts always add back up to `tax_paise`.
pub fn split_tax(tax_paise: i64, split: &[(&'static str, i64)]) -> Vec<(&'static str, i64)> {
    if tax_paise <= 0 || split.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(split.len());
    let mut assigned = 0i64;
    for (i, (name, fraction_bps)) in split.iter().enumerate() {
        let amount = if i + 1 == split.len() {
            tax_paise - assigned
        } else {
            let part = tax_paise.saturating_mul(*fraction_bps) / 10_000;
            assigned += part;
            part
        };
        out.push((*name, amount));
    }
    out
}

/// Reorder the `YYYY-MM-DD` head of a timestamp for the seller's country.
/// Anything that is not an ISO date comes back untouched.
pub fn format_date(timestamp: &str, date_order: &str) -> String {
    let head: String = timestamp.chars().take(10).collect();
    let parts: Vec<&str> = head.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 {
        return head;
    }
    let (y, m, d) = (parts[0], parts[1], parts[2]);
    match date_order {
        "mdy" => format!("{m}/{d}/{y}"),
        "ymd" => format!("{y}-{m}-{d}"),
        _ => format!("{d}/{m}/{y}"),
    }
}

// ———————————————————————————— Persisted settings ————————————————————————————

/// Country-derived wording and tax mode for this shop. Stored in `meta` next
/// to (not inside) [`crate::locale::LocaleSettings`] so language/currency stay
/// independently changeable after a pack is applied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiscalSettings {
    pub country_code: String,
    pub tax_pricing: TaxPricing,
    pub tax_label: String,
    pub business_tax_id: String,
    pub business_id_label: String,
    pub digital_pay_label: String,
    pub credit_label: String,
    pub date_order: String,
    /// Older peers sync without these two; they fall back to the plain
    /// `1,234.50` style rather than failing the whole snapshot.
    #[serde(default = "default_decimal_sep")]
    pub decimal_sep: String,
    #[serde(default = "default_thousands_sep")]
    pub thousands_sep: String,
    /// Optional service charge in basis points, added on top of the lines.
    /// No country pack sets one: a tip or service charge is a house rule, not
    /// a tax, so it stays 0 until the seller asks for it.
    #[serde(default)]
    pub service_charge_bps: i64,
}

fn default_decimal_sep() -> String {
    ".".to_string()
}

fn default_thousands_sep() -> String {
    ",".to_string()
}

impl Default for FiscalSettings {
    fn default() -> Self {
        Self::from_profile(country_or_generic(DEFAULT_COUNTRY), String::new())
    }
}

impl FiscalSettings {
    pub fn from_profile(profile: &CountryProfile, business_tax_id: String) -> Self {
        Self {
            country_code: profile.code.to_string(),
            tax_pricing: profile.tax_pricing,
            tax_label: profile.tax_name.to_string(),
            business_tax_id,
            business_id_label: profile.business_id_label.to_string(),
            digital_pay_label: profile.digital_pay_label.to_string(),
            credit_label: profile.credit_label.to_string(),
            date_order: profile.date_order.to_string(),
            decimal_sep: profile.decimal_sep.to_string(),
            thousands_sep: profile.thousands_sep.to_string(),
            service_charge_bps: 0,
        }
    }

    pub fn profile(&self) -> &'static CountryProfile {
        country_or_generic(&self.country_code)
    }

    /// Receipt/report components for one tax amount (empty when the country
    /// shows a single tax line).
    pub fn tax_components(&self, tax_paise: i64) -> Vec<(&'static str, i64)> {
        split_tax(tax_paise, self.profile().tax_split)
    }

    pub fn format_date(&self, timestamp: &str) -> String {
        format_date(timestamp, &self.date_order)
    }

    /// Never empty — falls back to a neutral "Tax".
    pub fn tax_label(&self) -> &str {
        let t = self.tax_label.trim();
        if t.is_empty() { "Tax" } else { t }
    }

    pub fn digital_pay_label(&self) -> &str {
        let t = self.digital_pay_label.trim();
        if t.is_empty() { "Digital" } else { t }
    }

    /// [`crate::domain::PayMethod::as_str`] value this country's instant rail
    /// is stored under. Still `upi` everywhere, so every historical sale row
    /// and every sync payload keeps parsing.
    pub fn digital_pay_code(&self) -> &'static str {
        self.profile().digital_pay_code
    }

    pub fn credit_label(&self) -> &str {
        let t = self.credit_label.trim();
        if t.is_empty() { "Credit" } else { t }
    }

    /// Decimal mark for this shop. Never empty — a missing one would glue the
    /// minor units onto the major ones.
    pub fn decimal_sep(&self) -> &str {
        if self.decimal_sep.is_empty() { "." } else { &self.decimal_sep }
    }

    /// Group mark. May legitimately be empty (no grouping) or a space, so it
    /// is never trimmed or defaulted.
    pub fn thousands_sep(&self) -> &str {
        &self.thousands_sep
    }

    /// Service charge this shop adds to a bill of `lines_total`. Always an
    /// add-on, never carved out of the quoted prices.
    pub fn service_on(&self, lines_total: i64) -> i64 {
        if self.service_charge_bps <= 0 || lines_total <= 0 {
            return 0;
        }
        lines_total.saturating_mul(self.service_charge_bps) / 10_000
    }

    /// Seller summary for the Settings screen, one "what: value" per line
    /// (the screen lays each out as a row).
    pub fn summary(&self) -> String {
        let profile = self.profile();
        let id = if self.business_tax_id.trim().is_empty() {
            "not set".to_string()
        } else {
            self.business_tax_id.trim().to_string()
        };
        let service = if self.service_charge_bps > 0 {
            format!(
                "\nService charge: {}%",
                crate::locale::format_percent_bps(self.service_charge_bps, self.decimal_sep())
            )
        } else {
            String::new()
        };
        let id_label = match self.business_id_label.trim() {
            "" => "Business id",
            label => label,
        };
        format!(
            "Country: {} ({})\nTax: {} · {}\nPay rail: {}\nCredit: {}\nNumbers: 1{}234{}56{service}\n{id_label}: {id}",
            profile.name_en,
            profile.code,
            self.tax_label(),
            self.tax_pricing.label(),
            self.digital_pay_label(),
            self.credit_label(),
            self.thousands_sep(),
            self.decimal_sep(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclusive_extracts_exclusive_adds() {
        // ₹105 quoted inclusive at 5% holds ₹5 of tax.
        assert_eq!(tax_on_amount(10_500, 500, TaxPricing::Inclusive), 500);
        assert_eq!(gross_with_tax(10_500, 500, TaxPricing::Inclusive), 10_500);
        assert_eq!(net_of_tax(10_500, 500, TaxPricing::Inclusive), 10_000);
        // $100 quoted net at 5% is charged as $105.
        assert_eq!(tax_on_amount(10_000, 500, TaxPricing::Exclusive), 500);
        assert_eq!(gross_with_tax(10_000, 500, TaxPricing::Exclusive), 10_500);
        assert_eq!(net_of_tax(10_000, 500, TaxPricing::Exclusive), 10_000);
        assert_eq!(tax_on_amount(10_000, 0, TaxPricing::Exclusive), 0);
    }

    #[test]
    fn india_splits_tax_in_equal_halves_without_losing_paise() {
        let profile = country_by_code("in").unwrap();
        assert_eq!(profile.code, "IN");
        let parts = split_tax(501, profile.tax_split);
        assert_eq!(parts, vec![("CGST", 250), ("SGST", 251)]);
        assert_eq!(parts.iter().map(|(_, a)| *a).sum::<i64>(), 501);
        assert!(split_tax(0, profile.tax_split).is_empty());
        assert!(split_tax(500, country_by_code("GB").unwrap().tax_split).is_empty());
    }

    #[test]
    fn every_profile_is_coherent() {
        for p in countries() {
            assert_eq!(p.code.len(), 2, "{} is not alpha-2", p.code);
            assert_eq!(TaxSystemKind::parse(p.tax_system.as_str()), p.tax_system);
            assert!(!p.tax_name.is_empty(), "{} has no tax name", p.code);
            assert_eq!(p.digital_pay_code, "upi", "{} must stay PayMethod::Upi", p.code);
            assert!(
                ["dmy", "mdy", "ymd"].contains(&p.date_order),
                "{} has a bad date order",
                p.code
            );
            let split_total: i64 = p.tax_split.iter().map(|(_, bps)| *bps).sum();
            assert!(
                p.tax_split.is_empty() || split_total == 10_000,
                "{} tax split must sum to 10000",
                p.code
            );
            for r in p.default_rates {
                assert!((0..=10_000).contains(&r.bps), "{} rate out of range", p.code);
            }
            assert!(
                [".", ","].contains(&p.decimal_sep),
                "{} has an unusable decimal mark",
                p.code
            );
            assert!(
                [",", ".", " ", "'", ""].contains(&p.thousands_sep),
                "{} has an unusable group mark",
                p.code
            );
            assert_ne!(
                p.decimal_sep, p.thousands_sep,
                "{} cannot group and point with the same mark",
                p.code
            );
        }
        assert!(country_by_code("ZZ").is_none());
        assert_eq!(country_or_generic("ZZ").code, GENERIC_CODE);
    }

    #[test]
    fn date_order_follows_the_country() {
        assert_eq!(format_date("2026-09-25T10:00:00+05:30", "dmy"), "25/09/2026");
        assert_eq!(format_date("2026-09-25T10:00:00+05:30", "mdy"), "09/25/2026");
        assert_eq!(format_date("2026-09-25T10:00:00+05:30", "ymd"), "2026-09-25");
        assert_eq!(format_date("not a date", "dmy"), "not a date");
    }
}
