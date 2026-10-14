//! Locale: languages, currencies and number styles for the whole world.
//!
//! The catalogue is intentionally broad. UI copy is translated where we have
//! a starter set; every other language still selects correctly and falls back
//! to English strings until its table is filled in.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocaleSettings {
    pub language: String,
    pub currency: String,
}

impl Default for LocaleSettings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            currency: "INR".into(),
        }
    }
}

/// Where the UI language comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LanguageSource {
    /// Always match the OS / device language.
    Platform,
    /// Fixed catalogue language chosen by the user.
    User,
}

impl LanguageSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Platform => "platform",
            Self::User => "user",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim() {
            "platform" => Self::Platform,
            _ => Self::User,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LanguageInfo {
    pub code: &'static str,
    pub name_en: &'static str,
    pub native_name: &'static str,
    pub region: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct CurrencyInfo {
    pub code: &'static str,
    pub symbol: &'static str,
    pub name_en: &'static str,
    pub decimals: u8,
}

/// Worldwide language catalogue (select in Settings; UI strings expand over time).
pub const LANGUAGES: &[LanguageInfo] = &[
    // ——— English ———
    LanguageInfo { code: "en", name_en: "English", native_name: "English", region: "World" },
    // ——— India: scheduled + widely used ———
    LanguageInfo { code: "hi", name_en: "Hindi", native_name: "हिन्दी", region: "India" },
    LanguageInfo { code: "bn", name_en: "Bengali", native_name: "বাংলা", region: "India" },
    LanguageInfo { code: "te", name_en: "Telugu", native_name: "తెలుగు", region: "India" },
    LanguageInfo { code: "mr", name_en: "Marathi", native_name: "मराठी", region: "India" },
    LanguageInfo { code: "ta", name_en: "Tamil", native_name: "தமிழ்", region: "India" },
    LanguageInfo { code: "ur", name_en: "Urdu", native_name: "اردو", region: "India" },
    LanguageInfo { code: "gu", name_en: "Gujarati", native_name: "ગુજરાતી", region: "India" },
    LanguageInfo { code: "kn", name_en: "Kannada", native_name: "ಕನ್ನಡ", region: "India" },
    LanguageInfo { code: "ml", name_en: "Malayalam", native_name: "മലയാളം", region: "India" },
    LanguageInfo { code: "or", name_en: "Odia", native_name: "ଓଡ଼ିଆ", region: "India" },
    LanguageInfo { code: "pa", name_en: "Punjabi", native_name: "ਪੰਜਾਬੀ", region: "India" },
    LanguageInfo { code: "as", name_en: "Assamese", native_name: "অসমীয়া", region: "India" },
    LanguageInfo { code: "mai", name_en: "Maithili", native_name: "मैथिली", region: "India" },
    LanguageInfo { code: "sa", name_en: "Sanskrit", native_name: "संस्कृतम्", region: "India" },
    LanguageInfo { code: "ks", name_en: "Kashmiri", native_name: "كٲشُر", region: "India" },
    LanguageInfo { code: "ne", name_en: "Nepali", native_name: "नेपाली", region: "India" },
    LanguageInfo { code: "sd", name_en: "Sindhi", native_name: "سنڌي", region: "India" },
    LanguageInfo { code: "kok", name_en: "Konkani", native_name: "कोंकणी", region: "India" },
    LanguageInfo { code: "doi", name_en: "Dogri", native_name: "डोगरी", region: "India" },
    LanguageInfo { code: "mni", name_en: "Manipuri", native_name: "ꯃꯤꯇꯩꯂꯣꯟ", region: "India" },
    LanguageInfo { code: "sat", name_en: "Santali", native_name: "ᱥᱟᱱᱛᱟᱲᱤ", region: "India" },
    LanguageInfo { code: "brx", name_en: "Bodo", native_name: "बर'", region: "India" },
    // ——— India: additional widely spoken ———
    LanguageInfo { code: "bho", name_en: "Bhojpuri", native_name: "भोजपुरी", region: "India" },
    LanguageInfo { code: "awa", name_en: "Awadhi", native_name: "अवधी", region: "India" },
    LanguageInfo { code: "mag", name_en: "Magahi", native_name: "मगही", region: "India" },
    LanguageInfo { code: "hne", name_en: "Chhattisgarhi", native_name: "छत्तीसगढ़ी", region: "India" },
    LanguageInfo { code: "raj", name_en: "Rajasthani", native_name: "राजस्थानी", region: "India" },
    LanguageInfo { code: "tcy", name_en: "Tulu", native_name: "ತುಳು", region: "India" },
    LanguageInfo { code: "trp", name_en: "Kokborok", native_name: "ককবরক", region: "India" },
    LanguageInfo { code: "lus", name_en: "Mizo", native_name: "Mizo ṭawng", region: "India" },
    LanguageInfo { code: "kha", name_en: "Khasi", native_name: "Khasi", region: "India" },
    LanguageInfo { code: "gom", name_en: "Goan Konkani", native_name: "गोंयची कोंकणी", region: "India" },
    LanguageInfo { code: "bo", name_en: "Tibetan", native_name: "བོད་སྐད་", region: "India" },
    // ——— South Asia neighbours ———
    LanguageInfo { code: "si", name_en: "Sinhala", native_name: "සිංහල", region: "South Asia" },
    LanguageInfo { code: "dv", name_en: "Dhivehi", native_name: "ދިވެހި", region: "South Asia" },
    LanguageInfo { code: "ps", name_en: "Pashto", native_name: "پښتو", region: "South Asia" },
    LanguageInfo { code: "fa", name_en: "Persian", native_name: "فارسی", region: "Middle East" },
    // ——— East / Southeast Asia ———
    LanguageInfo { code: "zh", name_en: "Chinese (Simplified)", native_name: "中文", region: "East Asia" },
    LanguageInfo { code: "zh-Hant", name_en: "Chinese (Traditional)", native_name: "繁體中文", region: "East Asia" },
    LanguageInfo { code: "yue", name_en: "Cantonese", native_name: "粵語", region: "East Asia" },
    LanguageInfo { code: "ja", name_en: "Japanese", native_name: "日本語", region: "East Asia" },
    LanguageInfo { code: "ko", name_en: "Korean", native_name: "한국어", region: "East Asia" },
    LanguageInfo { code: "vi", name_en: "Vietnamese", native_name: "Tiếng Việt", region: "Southeast Asia" },
    LanguageInfo { code: "th", name_en: "Thai", native_name: "ไทย", region: "Southeast Asia" },
    LanguageInfo { code: "id", name_en: "Indonesian", native_name: "Bahasa Indonesia", region: "Southeast Asia" },
    LanguageInfo { code: "ms", name_en: "Malay", native_name: "Bahasa Melayu", region: "Southeast Asia" },
    LanguageInfo { code: "fil", name_en: "Filipino", native_name: "Filipino", region: "Southeast Asia" },
    LanguageInfo { code: "my", name_en: "Burmese", native_name: "မြန်မာ", region: "Southeast Asia" },
    LanguageInfo { code: "km", name_en: "Khmer", native_name: "ខ្មែរ", region: "Southeast Asia" },
    LanguageInfo { code: "lo", name_en: "Lao", native_name: "ລາວ", region: "Southeast Asia" },
    // ——— Middle East / Central Asia ———
    LanguageInfo { code: "ar", name_en: "Arabic", native_name: "العربية", region: "Middle East" },
    LanguageInfo { code: "he", name_en: "Hebrew", native_name: "עברית", region: "Middle East" },
    LanguageInfo { code: "tr", name_en: "Turkish", native_name: "Türkçe", region: "Middle East" },
    LanguageInfo { code: "az", name_en: "Azerbaijani", native_name: "Azərbaycan", region: "Central Asia" },
    LanguageInfo { code: "kk", name_en: "Kazakh", native_name: "Қазақша", region: "Central Asia" },
    LanguageInfo { code: "uz", name_en: "Uzbek", native_name: "Oʻzbekcha", region: "Central Asia" },
    LanguageInfo { code: "ky", name_en: "Kyrgyz", native_name: "Кыргызча", region: "Central Asia" },
    LanguageInfo { code: "tg", name_en: "Tajik", native_name: "Тоҷикӣ", region: "Central Asia" },
    LanguageInfo { code: "tk", name_en: "Turkmen", native_name: "Türkmençe", region: "Central Asia" },
    LanguageInfo { code: "mn", name_en: "Mongolian", native_name: "Монгол", region: "East Asia" },
    // ——— Europe ———
    LanguageInfo { code: "es", name_en: "Spanish", native_name: "Español", region: "Europe" },
    LanguageInfo { code: "fr", name_en: "French", native_name: "Français", region: "Europe" },
    LanguageInfo { code: "pt", name_en: "Portuguese", native_name: "Português", region: "Europe" },
    LanguageInfo { code: "de", name_en: "German", native_name: "Deutsch", region: "Europe" },
    LanguageInfo { code: "it", name_en: "Italian", native_name: "Italiano", region: "Europe" },
    LanguageInfo { code: "ru", name_en: "Russian", native_name: "Русский", region: "Europe" },
    LanguageInfo { code: "uk", name_en: "Ukrainian", native_name: "Українська", region: "Europe" },
    LanguageInfo { code: "pl", name_en: "Polish", native_name: "Polski", region: "Europe" },
    LanguageInfo { code: "nl", name_en: "Dutch", native_name: "Nederlands", region: "Europe" },
    LanguageInfo { code: "ro", name_en: "Romanian", native_name: "Română", region: "Europe" },
    LanguageInfo { code: "hu", name_en: "Hungarian", native_name: "Magyar", region: "Europe" },
    LanguageInfo { code: "cs", name_en: "Czech", native_name: "Čeština", region: "Europe" },
    LanguageInfo { code: "sk", name_en: "Slovak", native_name: "Slovenčina", region: "Europe" },
    LanguageInfo { code: "bg", name_en: "Bulgarian", native_name: "Български", region: "Europe" },
    LanguageInfo { code: "sr", name_en: "Serbian", native_name: "Српски", region: "Europe" },
    LanguageInfo { code: "hr", name_en: "Croatian", native_name: "Hrvatski", region: "Europe" },
    LanguageInfo { code: "bs", name_en: "Bosnian", native_name: "Bosanski", region: "Europe" },
    LanguageInfo { code: "sl", name_en: "Slovenian", native_name: "Slovenščina", region: "Europe" },
    LanguageInfo { code: "el", name_en: "Greek", native_name: "Ελληνικά", region: "Europe" },
    LanguageInfo { code: "sv", name_en: "Swedish", native_name: "Svenska", region: "Europe" },
    LanguageInfo { code: "no", name_en: "Norwegian", native_name: "Norsk", region: "Europe" },
    LanguageInfo { code: "da", name_en: "Danish", native_name: "Dansk", region: "Europe" },
    LanguageInfo { code: "fi", name_en: "Finnish", native_name: "Suomi", region: "Europe" },
    LanguageInfo { code: "is", name_en: "Icelandic", native_name: "Íslenska", region: "Europe" },
    LanguageInfo { code: "ga", name_en: "Irish", native_name: "Gaeilge", region: "Europe" },
    LanguageInfo { code: "cy", name_en: "Welsh", native_name: "Cymraeg", region: "Europe" },
    LanguageInfo { code: "ca", name_en: "Catalan", native_name: "Català", region: "Europe" },
    LanguageInfo { code: "eu", name_en: "Basque", native_name: "Euskara", region: "Europe" },
    LanguageInfo { code: "gl", name_en: "Galician", native_name: "Galego", region: "Europe" },
    LanguageInfo { code: "lt", name_en: "Lithuanian", native_name: "Lietuvių", region: "Europe" },
    LanguageInfo { code: "lv", name_en: "Latvian", native_name: "Latviešu", region: "Europe" },
    LanguageInfo { code: "et", name_en: "Estonian", native_name: "Eesti", region: "Europe" },
    LanguageInfo { code: "mt", name_en: "Maltese", native_name: "Malti", region: "Europe" },
    LanguageInfo { code: "sq", name_en: "Albanian", native_name: "Shqip", region: "Europe" },
    LanguageInfo { code: "mk", name_en: "Macedonian", native_name: "Македонски", region: "Europe" },
    LanguageInfo { code: "be", name_en: "Belarusian", native_name: "Беларуская", region: "Europe" },
    LanguageInfo { code: "ka", name_en: "Georgian", native_name: "ქართული", region: "Europe" },
    LanguageInfo { code: "hy", name_en: "Armenian", native_name: "Հայերեն", region: "Europe" },
    // ——— Africa ———
    LanguageInfo { code: "sw", name_en: "Swahili", native_name: "Kiswahili", region: "Africa" },
    LanguageInfo { code: "am", name_en: "Amharic", native_name: "አማርኛ", region: "Africa" },
    LanguageInfo { code: "ha", name_en: "Hausa", native_name: "Hausa", region: "Africa" },
    LanguageInfo { code: "yo", name_en: "Yoruba", native_name: "Yorùbá", region: "Africa" },
    LanguageInfo { code: "ig", name_en: "Igbo", native_name: "Igbo", region: "Africa" },
    LanguageInfo { code: "zu", name_en: "Zulu", native_name: "isiZulu", region: "Africa" },
    LanguageInfo { code: "xh", name_en: "Xhosa", native_name: "isiXhosa", region: "Africa" },
    LanguageInfo { code: "af", name_en: "Afrikaans", native_name: "Afrikaans", region: "Africa" },
    LanguageInfo { code: "so", name_en: "Somali", native_name: "Soomaali", region: "Africa" },
    LanguageInfo { code: "rw", name_en: "Kinyarwanda", native_name: "Ikinyarwanda", region: "Africa" },
    LanguageInfo { code: "mg", name_en: "Malagasy", native_name: "Malagasy", region: "Africa" },
    // ——— Americas ———
    LanguageInfo { code: "pt-BR", name_en: "Portuguese (Brazil)", native_name: "Português (Brasil)", region: "Americas" },
    LanguageInfo { code: "es-MX", name_en: "Spanish (Mexico)", native_name: "Español (México)", region: "Americas" },
    LanguageInfo { code: "ht", name_en: "Haitian Creole", native_name: "Kreyòl ayisyen", region: "Americas" },
    LanguageInfo { code: "qu", name_en: "Quechua", native_name: "Runa Simi", region: "Americas" },
    LanguageInfo { code: "gn", name_en: "Guarani", native_name: "Avañe'ẽ", region: "Americas" },
];

pub const CURRENCIES: &[CurrencyInfo] = &[
    CurrencyInfo { code: "INR", symbol: "₹", name_en: "Indian Rupee", decimals: 2 },
    CurrencyInfo { code: "USD", symbol: "$", name_en: "US Dollar", decimals: 2 },
    CurrencyInfo { code: "EUR", symbol: "€", name_en: "Euro", decimals: 2 },
    CurrencyInfo { code: "GBP", symbol: "£", name_en: "British Pound", decimals: 2 },
    CurrencyInfo { code: "JPY", symbol: "¥", name_en: "Japanese Yen", decimals: 0 },
    CurrencyInfo { code: "CNY", symbol: "¥", name_en: "Chinese Yuan", decimals: 2 },
    CurrencyInfo { code: "AED", symbol: "د.إ", name_en: "UAE Dirham", decimals: 2 },
    CurrencyInfo { code: "SAR", symbol: "﷼", name_en: "Saudi Riyal", decimals: 2 },
    CurrencyInfo { code: "QAR", symbol: "ر.ق", name_en: "Qatari Riyal", decimals: 2 },
    CurrencyInfo { code: "KWD", symbol: "د.ك", name_en: "Kuwaiti Dinar", decimals: 3 },
    CurrencyInfo { code: "BHD", symbol: ".د.ب", name_en: "Bahraini Dinar", decimals: 3 },
    CurrencyInfo { code: "OMR", symbol: "ر.ع.", name_en: "Omani Rial", decimals: 3 },
    CurrencyInfo { code: "SGD", symbol: "S$", name_en: "Singapore Dollar", decimals: 2 },
    CurrencyInfo { code: "AUD", symbol: "A$", name_en: "Australian Dollar", decimals: 2 },
    CurrencyInfo { code: "CAD", symbol: "C$", name_en: "Canadian Dollar", decimals: 2 },
    CurrencyInfo { code: "NZD", symbol: "NZ$", name_en: "New Zealand Dollar", decimals: 2 },
    CurrencyInfo { code: "CHF", symbol: "CHF", name_en: "Swiss Franc", decimals: 2 },
    CurrencyInfo { code: "HKD", symbol: "HK$", name_en: "Hong Kong Dollar", decimals: 2 },
    CurrencyInfo { code: "TWD", symbol: "NT$", name_en: "New Taiwan Dollar", decimals: 2 },
    CurrencyInfo { code: "KRW", symbol: "₩", name_en: "South Korean Won", decimals: 0 },
    CurrencyInfo { code: "MYR", symbol: "RM", name_en: "Malaysian Ringgit", decimals: 2 },
    CurrencyInfo { code: "THB", symbol: "฿", name_en: "Thai Baht", decimals: 2 },
    CurrencyInfo { code: "IDR", symbol: "Rp", name_en: "Indonesian Rupiah", decimals: 0 },
    CurrencyInfo { code: "PHP", symbol: "₱", name_en: "Philippine Peso", decimals: 2 },
    CurrencyInfo { code: "VND", symbol: "₫", name_en: "Vietnamese Dong", decimals: 0 },
    CurrencyInfo { code: "BRL", symbol: "R$", name_en: "Brazilian Real", decimals: 2 },
    CurrencyInfo { code: "MXN", symbol: "MX$", name_en: "Mexican Peso", decimals: 2 },
    CurrencyInfo { code: "ARS", symbol: "AR$", name_en: "Argentine Peso", decimals: 2 },
    CurrencyInfo { code: "CLP", symbol: "CL$", name_en: "Chilean Peso", decimals: 0 },
    CurrencyInfo { code: "COP", symbol: "CO$", name_en: "Colombian Peso", decimals: 2 },
    CurrencyInfo { code: "PEN", symbol: "S/", name_en: "Peruvian Sol", decimals: 2 },
    CurrencyInfo { code: "ZAR", symbol: "R", name_en: "South African Rand", decimals: 2 },
    CurrencyInfo { code: "NGN", symbol: "₦", name_en: "Nigerian Naira", decimals: 2 },
    CurrencyInfo { code: "KES", symbol: "KSh", name_en: "Kenyan Shilling", decimals: 2 },
    CurrencyInfo { code: "GHS", symbol: "GH₵", name_en: "Ghanaian Cedi", decimals: 2 },
    CurrencyInfo { code: "EGP", symbol: "E£", name_en: "Egyptian Pound", decimals: 2 },
    CurrencyInfo { code: "MAD", symbol: "د.م.", name_en: "Moroccan Dirham", decimals: 2 },
    CurrencyInfo { code: "TRY", symbol: "₺", name_en: "Turkish Lira", decimals: 2 },
    CurrencyInfo { code: "RUB", symbol: "₽", name_en: "Russian Ruble", decimals: 2 },
    CurrencyInfo { code: "UAH", symbol: "₴", name_en: "Ukrainian Hryvnia", decimals: 2 },
    CurrencyInfo { code: "PLN", symbol: "zł", name_en: "Polish Zloty", decimals: 2 },
    CurrencyInfo { code: "CZK", symbol: "Kč", name_en: "Czech Koruna", decimals: 2 },
    CurrencyInfo { code: "SEK", symbol: "kr", name_en: "Swedish Krona", decimals: 2 },
    CurrencyInfo { code: "NOK", symbol: "kr", name_en: "Norwegian Krone", decimals: 2 },
    CurrencyInfo { code: "DKK", symbol: "kr", name_en: "Danish Krone", decimals: 2 },
    CurrencyInfo { code: "HUF", symbol: "Ft", name_en: "Hungarian Forint", decimals: 0 },
    CurrencyInfo { code: "RON", symbol: "lei", name_en: "Romanian Leu", decimals: 2 },
    CurrencyInfo { code: "ILS", symbol: "₪", name_en: "Israeli Shekel", decimals: 2 },
    CurrencyInfo { code: "PKR", symbol: "₨", name_en: "Pakistani Rupee", decimals: 2 },
    CurrencyInfo { code: "BDT", symbol: "৳", name_en: "Bangladeshi Taka", decimals: 2 },
    CurrencyInfo { code: "LKR", symbol: "Rs", name_en: "Sri Lankan Rupee", decimals: 2 },
    CurrencyInfo { code: "NPR", symbol: "रू", name_en: "Nepalese Rupee", decimals: 2 },
    CurrencyInfo { code: "MVR", symbol: "Rf", name_en: "Maldivian Rufiyaa", decimals: 2 },
    CurrencyInfo { code: "BTN", symbol: "Nu.", name_en: "Bhutanese Ngultrum", decimals: 2 },
    CurrencyInfo { code: "AFN", symbol: "؋", name_en: "Afghan Afghani", decimals: 2 },
    CurrencyInfo { code: "MMK", symbol: "K", name_en: "Myanmar Kyat", decimals: 0 },
    CurrencyInfo { code: "KHR", symbol: "៛", name_en: "Cambodian Riel", decimals: 0 },
    CurrencyInfo { code: "LAK", symbol: "₭", name_en: "Lao Kip", decimals: 0 },
];

pub fn language_by_code(code: &str) -> Option<&'static LanguageInfo> {
    LANGUAGES.iter().find(|l| l.code == code)
}

pub fn currency_by_code(code: &str) -> Option<&'static CurrencyInfo> {
    CURRENCIES.iter().find(|c| c.code == code)
}

/// Map a BCP-47 / POSIX locale tag (e.g. `hi_IN.UTF-8`, `zh-TW`) onto a
/// catalogue language code. Unknown tags fall back to `en`.
pub fn match_language_tag(tag: &str) -> String {
    let cleaned = tag.trim();
    if cleaned.is_empty()
        || cleaned.eq_ignore_ascii_case("C")
        || cleaned.eq_ignore_ascii_case("POSIX")
    {
        return "en".into();
    }
    // Strip encoding / modifier: `en_IN.UTF-8` → `en_IN`, `sr@latin` → `sr`.
    let no_encoding = cleaned.split(['.', '@']).next().unwrap_or(cleaned);
    let normalized = no_encoding.replace('_', "-");
    if let Some(info) = language_by_code(&normalized) {
        return info.code.into();
    }
    if let Some(info) = LANGUAGES
        .iter()
        .find(|l| l.code.eq_ignore_ascii_case(&normalized))
    {
        return info.code.into();
    }
    let lower = normalized.to_ascii_lowercase();
    if lower.starts_with("zh-hant")
        || lower.starts_with("zh-tw")
        || lower.starts_with("zh-hk")
        || lower.starts_with("zh-mo")
    {
        return "zh-Hant".into();
    }
    if lower.starts_with("zh") {
        return "zh".into();
    }
    let primary = lower.split('-').next().unwrap_or("en");
    if primary == "tl" {
        return "fil".into();
    }
    if let Some(info) = language_by_code(primary) {
        return info.code.into();
    }
    "en".into()
}

/// Best-effort language from the OS (Android / desktop locale, else `LANG`).
pub fn detect_platform_language() -> String {
    let tag = sys_locale::get_locale()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| std::env::var("LC_ALL").ok())
        .or_else(|| std::env::var("LC_MESSAGES").ok())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_else(|| "en".into());
    match_language_tag(&tag)
}

/// Money in the shop's own number style: `₹1,234.50`, `1.234,50 €`-style
/// separators, Swiss `1'234.50`, or no grouping at all.
pub fn format_money_ex(
    minor_units: i64,
    currency_code: &str,
    decimal_sep: &str,
    thousands_sep: &str,
) -> String {
    let info = currency_by_code(currency_code).unwrap_or(&CURRENCIES[0]);
    let decimal_sep = if decimal_sep.is_empty() { "." } else { decimal_sep };
    let sign = if minor_units < 0 { "-" } else { "" };
    let abs = minor_units.abs();
    let factor = 10i64.pow(info.decimals as u32);
    let major = group_thousands(abs / factor, thousands_sep);
    if info.decimals == 0 {
        return format!("{sign}{}{major}", info.symbol);
    }
    let width = info.decimals as usize;
    let minor = abs % factor;
    format!(
        "{sign}{}{major}{decimal_sep}{minor:0width$}",
        info.symbol,
        minor = minor,
        width = width
    )
}

/// Default number style (`.` decimal, `,` groups) — used wherever no fiscal
/// settings are at hand.
pub fn format_money(minor_units: i64, currency_code: &str) -> String {
    format_money_ex(minor_units, currency_code, ".", ",")
}

/// Three-digit grouping. An empty separator leaves the digits untouched.
fn group_thousands(major: i64, sep: &str) -> String {
    let digits = major.to_string();
    if sep.is_empty() || digits.len() <= 3 {
        return digits;
    }
    let bytes = digits.as_bytes();
    let head = bytes.len() % 3;
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 * sep.len());
    if head > 0 {
        out.push_str(&digits[..head]);
    }
    for (i, chunk) in bytes[head..].chunks(3).enumerate() {
        if i > 0 || head > 0 {
            out.push_str(sep);
        }
        out.push_str(std::str::from_utf8(chunk).unwrap_or_default());
    }
    out
}

/// Major-unit string for editable price fields (no currency symbol).
pub fn format_major_input(minor_units: i64, currency_code: &str) -> String {
    format_major_input_ex(minor_units, currency_code, ".")
}

/// Same, written with the shop's decimal mark. Never grouped: grouping marks
/// in an edit field are noise the seller has to delete.
pub fn format_major_input_ex(
    minor_units: i64,
    currency_code: &str,
    decimal_sep: &str,
) -> String {
    let decimal_sep = if decimal_sep.is_empty() { "." } else { decimal_sep };
    let decimals = currency_by_code(currency_code)
        .map(|c| c.decimals)
        .unwrap_or(2);
    let sign = if minor_units < 0 { "-" } else { "" };
    let abs = minor_units.abs();
    if decimals == 0 {
        return format!("{sign}{abs}");
    }
    let factor = 10i64.pow(decimals as u32);
    let major = abs / factor;
    let minor = abs % factor;
    if minor == 0 {
        format!("{sign}{major}")
    } else {
        let frac = format!("{minor:0width$}", width = decimals as usize);
        let frac = frac.trim_end_matches('0');
        format!("{sign}{major}{decimal_sep}{frac}")
    }
}

/// Parse a seller-typed major amount into minor units.
///
/// Works whatever keyboard and habit the seller has: `50`, `13.5`, `13,5`,
/// `1,234.50`, `1.234,50`, `1 234,50`, `1'234.50`. When both marks appear the
/// *last* one is the decimal; a lone mark is the decimal unless it sits
/// exactly three digits from the end, which is grouping the world over.
pub fn parse_major_input(text: &str, currency_code: &str) -> Result<i64, String> {
    let decimals = currency_by_code(currency_code)
        .map(|c| c.decimals)
        .unwrap_or(2) as usize;
    let text = normalize_major_input(text.trim(), decimals);
    if text.is_empty() {
        return Err("Amount required".into());
    }
    let negative = text.starts_with('-');
    let text = text.trim_start_matches('+').trim_start_matches('-');
    let (major_s, frac_s) = match text.split_once('.') {
        Some((a, b)) => (a, b),
        None => (text, ""),
    };
    if major_s.is_empty() || !major_s.chars().all(|c| c.is_ascii_digit()) {
        return Err("Invalid amount".into());
    }
    if !frac_s.chars().all(|c| c.is_ascii_digit()) {
        return Err("Invalid amount".into());
    }
    if frac_s.len() > decimals {
        return Err("Too many decimal places".into());
    }
    let major: i64 = major_s
        .parse()
        .map_err(|_| "Invalid amount".to_string())?;
    let mut frac = frac_s.to_string();
    while frac.len() < decimals {
        frac.push('0');
    }
    let frac_val: i64 = if decimals == 0 {
        0
    } else {
        frac.parse().map_err(|_| "Invalid amount".to_string())?
    };
    let factor = 10i64.pow(decimals as u32);
    let mut minor = major
        .saturating_mul(factor)
        .saturating_add(frac_val);
    if negative {
        minor = -minor;
    }
    Ok(minor)
}

/// Percent typed by a seller (`10`, `12.5`, `10,5`, `10 %`) as basis points.
/// A percentage carries two decimals, so hundredths of a percent *are* bps.
pub fn parse_percent_bps(text: &str) -> Result<i64, String> {
    let text = text.trim().trim_end_matches('%').trim();
    if text.is_empty() {
        return Ok(0);
    }
    let bps = parse_major_input(text, "USD").map_err(|_| "Invalid percent".to_string())?;
    if !(0..=10_000).contains(&bps) {
        return Err("Percent must be between 0 and 100".into());
    }
    Ok(bps)
}

/// `1_250` → `12.5`, written with the shop's decimal mark. Empty at zero, so
/// the edit field shows its placeholder instead of a meaningless 0.
pub fn format_percent_bps(bps: i64, decimal_sep: &str) -> String {
    if bps <= 0 {
        return String::new();
    }
    format_major_input_ex(bps, "USD", decimal_sep)
}

/// Rewrite a typed amount into plain `-1234.50` form: group marks gone and a
/// single `.` decimal, whichever local convention the seller used.
fn normalize_major_input(text: &str, decimals: usize) -> String {
    // Spaces and apostrophes only ever group, never separate decimals.
    let stripped: String = text
        .chars()
        .filter(|c| !matches!(c, ' ' | '\u{00a0}' | '\u{202f}' | '\''))
        .collect();
    let decimal_at = match (stripped.rfind('.'), stripped.rfind(',')) {
        (Some(dot), Some(comma)) => Some(dot.max(comma)),
        (Some(dot), None) => decimal_or_group(&stripped, '.', dot, decimals),
        (None, Some(comma)) => decimal_or_group(&stripped, ',', comma, decimals),
        (None, None) => None,
    };
    stripped
        .char_indices()
        .filter_map(|(i, c)| match c {
            '.' | ',' if Some(i) == decimal_at => Some('.'),
            '.' | ',' => None,
            other => Some(other),
        })
        .collect()
}

/// A lone mark is the decimal unless it sits exactly three digits from the
/// end — `1,234` and `1.234` are thousands everywhere except in the handful
/// of currencies that really do carry three decimals.
fn decimal_or_group(text: &str, mark: char, at: usize, decimals: usize) -> Option<usize> {
    if text.matches(mark).count() > 1 {
        return None;
    }
    let after = text.len() - at - mark.len_utf8();
    if after == 3 && decimals != 3 {
        return None;
    }
    Some(at)
}

fn en(key: &str) -> String {
    match key {
        "app_tagline" => "Run any business. Free offline.",
        "unlock" => "Unlock",
        "pin_hint" => "Owner PIN starts as 0000",
        "sell" => "Sell",
        "buy" => "Buy",
        "books" => "Books",
        "books_help" => "Today's money in and out, who owes, cash vs bank. Sell and Buy are peers below.",
        "buy_help" => "Money out: goods received (purchase) or running costs (expense). Same books as Sell.",
        "sell_money_in" => "Sell (money in)",
        "buy_money_out" => "Buy (money out)",
        "add_expense" => "Add expense",
        "items" => "Items",
        "more" => "More",
        "your_items" => "Your items",
        "items_help" => "Tap an item to edit it. New item clears the form so you can add a different one. Sales already recorded keep the name they were charged with.",
        "add_item" => "Add",
        "new_item" => "New item",
        "adding_item" => "Adding a new item",
        "editing_item" => "Editing",
        "save" => "Save",
        "remove" => "Remove",
        "empty_catalog" => "No items yet. Open More → Items to add your first product.",
        "counter_sale" => "Counter sale",
        "total" => "Total",
        "cash" => "Cash",
        "upi" => "UPI",
        "card" => "Card",
        "credit" => "Credit",
        "clear" => "Clear",
        "ready_to_sell" => "Ready to sell",
        "today_prefix" => "Today",
        "today_mid" => "today",
        "owed" => "owed",
        "round_word" => "round",
        "back" => "Back",
        "sold_out" => "Sold Out",
        "restock" => "Restock",
        "not_tracked" => "Not tracked",
        "in_stock" => "in stock",
        "marked_sold_out" => "Marked sold out",
        "add_upper" => "ADD",
        "na" => "N/A",
        "qty" => "QTY",
        "unit_at" => "Unit @",
        "actual" => "Actual",
        "applied" => "Applied",
        "note_ph" => "Note (optional)",
        "rounded" => "ROUNDED",
        "split" => "Split",
        "send_kot" => "Send KOT",
        "counter" => "Counter",
        "dine_in" => "Dine-in",
        "takeaway" => "Takeaway",
        "order" => "Order",
        "tables" => "Tables",
        "kitchen" => "Kitchen",
        "sales" => "Sales",
        "sales_refunds" => "Sales & refunds",
        "day_close" => "Day close",
        "reports" => "Reports",
        "reports_expenses" => "Reports & expenses",
        "inventory" => "Inventory",
        "inventory_stock" => "Inventory & stock",
        "branches" => "Branches",
        "branches_org" => "Branches & org",
        "credit_customers" => "Credit customers",
        "customer_name_ph" => "Customer name",
        "new_customer_ph" => "New customer name",
        "start_template" => "Start template",
        "team" => "Team",
        "team_pins" => "Team & PINs",
        "backup_settings" => "Backup & settings",
        "lock_app" => "Lock app",
        "pick" => "Pick",
        "select" => "Select",
        "advance" => "Advance",
        "owner" => "Owner",
        "staff" => "Staff",
        "photo" => "Photo",
        "auto" => "Auto",
        "prev" => "Prev",
        "next" => "Next",
        "ok" => "OK",
        "this_shop" => "Server Mode",
        "extra_till" => "Client Mode",
        "off" => "Off",
        "tables_kitchen" => "Tables & kitchen",
        "server" => "Server",
        "client" => "Client",
        "language_currency" => "Language & currency",
        "language" => "Language",
        "choose_language" => "Choose your language",
        "language_setup_hint" => "Follow this device’s language, or pick one yourself.",
        "lang_follow_platform" => "Use platform native language",
        "lang_pick_user" => "Use user selected language",
        "lang_platform_now" => "Device language now",
        "selected_language" => "Selected",
        "continue" => "Continue",
        "this_device" => "THIS DEVICE",
        "device_use_q" => "How will this device be used?",
        "last_server_mode" => "Last time: Server Mode — tap again to continue, or switch",
        "last_client_mode" => "Last time: Client Mode — tap again to continue, or switch",
        "choose_server" => "Choose a server",
        "choose_server_hint" => "This device already has servers. Open one, or create a new server for another business.",
        "create_new_server" => "Create new server",
        "open_server" => "Open",
        "new_server_name_ph" => "New server name (optional)",
        "device_setup_help" => "Server Mode: hold the books on this device (create or open a local server). Client Mode: pair to a Server. Language and mode are asked every launch. Extra tills stay off until allowed in Settings.",
        "currency" => "Currency",
        "backup" => "Backup",
        "use" => "Use",
        _ => key,
    }
    .into()
}

fn lookup(table: &[(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// Locale string with optional template overlays (concrete feel without forks).
pub fn t_overlay(lang: &str, key: &str, overlays: &[(&str, &str)]) -> String {
    if let Some((_, value)) = overlays.iter().find(|(k, _)| *k == key) {
        return (*value).into();
    }
    t(lang, key)
}

/// Minimal UI string table — expand per language over time.
pub fn t(lang: &str, key: &str) -> String {
    // Normalize regional variants to a parent when we only have one table.
    let lang = match lang {
        "pt-BR" => "pt",
        "es-MX" => "es",
        "zh-Hant" | "yue" => "zh",
        other => other,
    };
    let translated = match lang {
        "hi" => lookup(
            &[
                ("app_tagline", "कोई भी कारोबार चलाएँ। ऑफ़लाइन मुफ़्त।"),
                ("unlock", "खोलें"),
                ("sell", "बेचें"),
                ("items", "सामान"),
                ("more", "और"),
                ("your_items", "आपके सामान"),
                ("items_help", "सामान पर टैप करके बदलें। नया सामान दबाने पर फ़ॉर्म खाली होता है और दूसरा सामान जोड़ा जा सकता है। पुरानी बिक्री का नाम नहीं बदलता।"),
                ("add_item", "जोड़ें"),
                ("new_item", "नया सामान"),
                ("adding_item", "नया सामान जोड़ रहे हैं"),
                ("editing_item", "बदल रहे हैं"),
                ("save", "सहेजें"),
                ("remove", "हटाएँ"),
                ("empty_catalog", "अभी कोई सामान नहीं। सामान टैब में जोड़ें।"),
                ("total", "कुल"),
                ("cash", "नकद"),
                ("credit", "उधार"),
                ("language", "भाषा"),
                ("currency", "मुद्रा"),
                ("use", "चुनें"),
                ("language_setup_hint", "इस डिवाइस की भाषा अपनाएँ, या स्वयं चुनें।"),
                ("lang_follow_platform", "प्लेटफ़ॉर्म की भाषा उपयोग करें"),
                ("lang_pick_user", "उपयोगकर्ता चुनी भाषा"),
                ("lang_platform_now", "अभी डिवाइस की भाषा"),
                ("selected_language", "चयनित"),
                ("this_device", "यह डिवाइस"),
                ("device_use_q", "यह डिवाइस कैसे उपयोग होगा?"),
                ("device_setup_help", "सर्वर मोड: इस डिवाइस पर किताबें (नया सर्वर बनाएँ या मौजूद खोलें)। क्लाइंट मोड: किसी सर्वर से जोड़ें। भाषा और मोड हर लॉन्च पर।"),
                ("last_server_mode", "पिछली बार: सर्वर मोड — जारी रखने के लिए फिर टैप करें, या बदलें"),
                ("last_client_mode", "पिछली बार: क्लाइंट मोड — जारी रखने के लिए फिर टैप करें, या बदलें"),
                ("choose_server", "सर्वर चुनें"),
                ("choose_server_hint", "इस डिवाइस पर सर्वर पहले से हैं। एक खोलें, या दूसरे कारोबार के लिए नया बनाएँ।"),
                ("create_new_server", "नया सर्वर बनाएँ"),
                ("open_server", "खोलें"),
                ("new_server_name_ph", "नए सर्वर का नाम (वैकल्पिक)"),
                ("this_shop", "सर्वर मोड"),
                ("extra_till", "क्लाइंट मोड"),
                ("continue", "आगे"),
            ],
            key,
        ),
        "ta" => lookup(
            &[
                ("sell", "விற்பனை"),
                ("items", "பொருட்கள்"),
                ("more", "மேலும்"),
                ("total", "மொத்தம்"),
                ("cash", "ரொக்கம்"),
                ("credit", "கடன்"),
                ("language", "மொழி"),
                ("currency", "நாணயம்"),
                ("use", "பயன்படுத்து"),
            ],
            key,
        ),
        "te" => lookup(
            &[
                ("sell", "అమ్మకం"),
                ("items", "వస్తువులు"),
                ("more", "మరిన్ని"),
                ("total", "మొత్తం"),
                ("cash", "నగదు"),
                ("credit", "క్రెడిట్"),
                ("language", "భాష"),
                ("currency", "కరెన్సీ"),
                ("use", "ఎంచుకో"),
            ],
            key,
        ),
        "bn" => lookup(
            &[
                ("sell", "বিক্রি"),
                ("items", "পণ্য"),
                ("more", "আরও"),
                ("total", "মোট"),
                ("cash", "নগদ"),
                ("credit", "বাকি"),
                ("language", "ভাষা"),
                ("currency", "মুদ্রা"),
                ("use", "ব্যবহার"),
            ],
            key,
        ),
        "mr" => lookup(
            &[
                ("sell", "विक्री"),
                ("items", "वस्तू"),
                ("more", "अधिक"),
                ("total", "एकूण"),
                ("cash", "रोख"),
                ("credit", "उधार"),
                ("language", "भाषा"),
                ("currency", "चलन"),
                ("use", "वापरा"),
            ],
            key,
        ),
        "gu" => lookup(
            &[
                ("sell", "વેચાણ"),
                ("items", "વસ્તુઓ"),
                ("more", "વધુ"),
                ("total", "કુલ"),
                ("cash", "રોકડ"),
                ("credit", "ઉધાર"),
                ("language", "ભાષા"),
                ("currency", "ચલણ"),
                ("use", "વાપરો"),
            ],
            key,
        ),
        "kn" => lookup(
            &[
                ("sell", "ಮಾರಾಟ"),
                ("items", "ವಸ್ತುಗಳು"),
                ("more", "ಇನ್ನಷ್ಟು"),
                ("total", "ಒಟ್ಟು"),
                ("cash", "ನಗದು"),
                ("credit", "ಸಾಲ"),
                ("language", "ಭಾಷೆ"),
                ("currency", "ಕರೆನ್ಸಿ"),
                ("use", "ಬಳಸಿ"),
            ],
            key,
        ),
        "ml" => lookup(
            &[
                ("sell", "വിൽപ്പന"),
                ("items", "സാധനങ്ങൾ"),
                ("more", "കൂടുതൽ"),
                ("total", "ആകെ"),
                ("cash", "പണം"),
                ("credit", "കടം"),
                ("language", "ഭാഷ"),
                ("currency", "കറൻസി"),
                ("use", "ഉപയോഗിക്കുക"),
            ],
            key,
        ),
        "pa" => lookup(
            &[
                ("sell", "ਵਿਕਰੀ"),
                ("items", "ਸਮਾਨ"),
                ("more", "ਹੋਰ"),
                ("total", "ਕੁੱਲ"),
                ("cash", "ਨਕਦ"),
                ("credit", "ਉਧਾਰ"),
                ("language", "ਭਾਸ਼ਾ"),
                ("currency", "ਮੁਦਰਾ"),
                ("use", "ਵਰਤੋ"),
            ],
            key,
        ),
        "ur" => lookup(
            &[
                ("sell", "فروخت"),
                ("items", "اشیاء"),
                ("more", "مزید"),
                ("total", "کل"),
                ("cash", "نقد"),
                ("credit", "ادھار"),
                ("language", "زبان"),
                ("currency", "کرنسی"),
                ("use", "استعمال"),
            ],
            key,
        ),
        "or" => lookup(
            &[
                ("sell", "ବିକ୍ରୟ"),
                ("items", "ସାମଗ୍ରୀ"),
                ("more", "ଅଧିକ"),
                ("total", "ମୋଟ"),
                ("cash", "ନଗଦ"),
                ("credit", "ଉଧାର"),
                ("language", "ଭାଷା"),
                ("currency", "ମୁଦ୍ରା"),
                ("use", "ବ୍ୟବହାର"),
            ],
            key,
        ),
        "as" => lookup(
            &[
                ("sell", "বিক্ৰী"),
                ("items", "সামগ্ৰী"),
                ("more", "অধিক"),
                ("total", "মুঠ"),
                ("cash", "নগদ"),
                ("credit", "ধাৰ"),
                ("language", "ভাষা"),
                ("currency", "মুদ্ৰা"),
                ("use", "ব্যৱহাৰ"),
            ],
            key,
        ),
        "es" => lookup(
            &[
                ("sell", "Vender"),
                ("items", "Artículos"),
                ("more", "Más"),
                ("total", "Total"),
                ("cash", "Efectivo"),
                ("credit", "Crédito"),
                ("language", "Idioma"),
                ("currency", "Moneda"),
                ("use", "Usar"),
            ],
            key,
        ),
        "fr" => lookup(
            &[
                ("sell", "Vendre"),
                ("items", "Articles"),
                ("more", "Plus"),
                ("total", "Total"),
                ("cash", "Espèces"),
                ("credit", "Crédit"),
                ("language", "Langue"),
                ("currency", "Devise"),
                ("use", "Utiliser"),
            ],
            key,
        ),
        "pt" => lookup(
            &[
                ("sell", "Vender"),
                ("items", "Itens"),
                ("more", "Mais"),
                ("total", "Total"),
                ("cash", "Dinheiro"),
                ("credit", "Crédito"),
                ("language", "Idioma"),
                ("currency", "Moeda"),
                ("use", "Usar"),
            ],
            key,
        ),
        "de" => lookup(
            &[
                ("sell", "Verkauf"),
                ("items", "Artikel"),
                ("more", "Mehr"),
                ("total", "Summe"),
                ("cash", "Bar"),
                ("credit", "Kredit"),
                ("language", "Sprache"),
                ("currency", "Währung"),
                ("use", "Nutzen"),
            ],
            key,
        ),
        "ar" => lookup(
            &[
                ("sell", "بيع"),
                ("items", "أصناف"),
                ("more", "المزيد"),
                ("total", "الإجمالي"),
                ("cash", "نقداً"),
                ("credit", "آجل"),
                ("language", "اللغة"),
                ("currency", "العملة"),
                ("use", "استخدم"),
            ],
            key,
        ),
        "zh" => lookup(
            &[
                ("sell", "销售"),
                ("items", "商品"),
                ("more", "更多"),
                ("total", "合计"),
                ("cash", "现金"),
                ("credit", "赊账"),
                ("language", "语言"),
                ("currency", "货币"),
                ("use", "使用"),
            ],
            key,
        ),
        "ja" => lookup(
            &[
                ("sell", "販売"),
                ("items", "商品"),
                ("more", "その他"),
                ("total", "合計"),
                ("cash", "現金"),
                ("credit", "掛け売り"),
                ("language", "言語"),
                ("currency", "通貨"),
                ("use", "使う"),
            ],
            key,
        ),
        "ko" => lookup(
            &[
                ("sell", "판매"),
                ("items", "상품"),
                ("more", "더보기"),
                ("total", "합계"),
                ("cash", "현금"),
                ("credit", "외상"),
                ("language", "언어"),
                ("currency", "통화"),
                ("use", "사용"),
            ],
            key,
        ),
        "ru" => lookup(
            &[
                ("sell", "Продажа"),
                ("items", "Товары"),
                ("more", "Ещё"),
                ("total", "Итого"),
                ("cash", "Наличные"),
                ("credit", "В долг"),
                ("language", "Язык"),
                ("currency", "Валюта"),
                ("use", "Выбрать"),
            ],
            key,
        ),
        "id" => lookup(
            &[
                ("sell", "Jual"),
                ("items", "Barang"),
                ("more", "Lainnya"),
                ("total", "Total"),
                ("cash", "Tunai"),
                ("credit", "Kredit"),
                ("language", "Bahasa"),
                ("currency", "Mata uang"),
                ("use", "Pakai"),
            ],
            key,
        ),
        "tr" => lookup(
            &[
                ("sell", "Satış"),
                ("items", "Ürünler"),
                ("more", "Daha fazla"),
                ("total", "Toplam"),
                ("cash", "Nakit"),
                ("credit", "Veresiye"),
                ("language", "Dil"),
                ("currency", "Para birimi"),
                ("use", "Kullan"),
            ],
            key,
        ),
        "vi" => lookup(
            &[
                ("sell", "Bán"),
                ("items", "Mặt hàng"),
                ("more", "Thêm"),
                ("total", "Tổng"),
                ("cash", "Tiền mặt"),
                ("credit", "Ghi nợ"),
                ("language", "Ngôn ngữ"),
                ("currency", "Tiền tệ"),
                ("use", "Dùng"),
            ],
            key,
        ),
        "th" => lookup(
            &[
                ("sell", "ขาย"),
                ("items", "สินค้า"),
                ("more", "เพิ่มเติม"),
                ("total", "รวม"),
                ("cash", "เงินสด"),
                ("credit", "เครดิต"),
                ("language", "ภาษา"),
                ("currency", "สกุลเงิน"),
                ("use", "ใช้"),
            ],
            key,
        ),
        "it" => lookup(
            &[
                ("sell", "Vendi"),
                ("items", "Articoli"),
                ("more", "Altro"),
                ("total", "Totale"),
                ("cash", "Contanti"),
                ("credit", "Credito"),
                ("language", "Lingua"),
                ("currency", "Valuta"),
                ("use", "Usa"),
            ],
            key,
        ),
        "nl" => lookup(
            &[
                ("sell", "Verkopen"),
                ("items", "Artikelen"),
                ("more", "Meer"),
                ("total", "Totaal"),
                ("cash", "Contant"),
                ("credit", "Krediet"),
                ("language", "Taal"),
                ("currency", "Valuta"),
                ("use", "Gebruik"),
            ],
            key,
        ),
        "sw" => lookup(
            &[
                ("sell", "Uza"),
                ("items", "Bidhaa"),
                ("more", "Zaidi"),
                ("total", "Jumla"),
                ("cash", "Taslimu"),
                ("credit", "Mkopo"),
                ("language", "Lugha"),
                ("currency", "Sarafu"),
                ("use", "Tumia"),
            ],
            key,
        ),
        _ => None,
    };
    if let Some(hit) = translated {
        return hit.to_string();
    }
    if let Some(hit) = chrome_extra(lang, key) {
        return hit.to_string();
    }
    en(key)
}

fn chrome_extra(lang: &str, key: &str) -> Option<&'static str> {
    include!("locale_chrome.rs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_language_tag_maps_common_os_locales() {
        assert_eq!(match_language_tag("hi_IN.UTF-8"), "hi");
        assert_eq!(match_language_tag("en-US"), "en");
        assert_eq!(match_language_tag("ta_IN"), "ta");
        assert_eq!(match_language_tag("zh-TW"), "zh-Hant");
        assert_eq!(match_language_tag("zh_CN.UTF-8"), "zh");
        assert_eq!(match_language_tag("pt_BR"), "pt-BR");
        assert_eq!(match_language_tag("pt-PT"), "pt");
        assert_eq!(match_language_tag("tl-PH"), "fil");
        assert_eq!(match_language_tag("C"), "en");
        assert_eq!(match_language_tag(""), "en");
        assert_eq!(match_language_tag("xx-YY"), "en");
    }

    #[test]
    fn shipped_languages_translate_chrome() {
        const LANGS: &[&str] = &[
            "hi", "ta", "te", "bn", "mr", "gu", "kn", "ml", "pa", "ur", "or", "as", "es", "fr",
            "pt", "de", "ar", "zh", "ja", "ko", "ru", "id", "tr", "vi", "th", "it", "nl", "sw",
        ];
        const KEYS: &[&str] = &[
            "back", "sold_out", "restock", "not_tracked", "in_stock", "today_prefix", "owed",
            "cash", "sell", "items", "more", "tables", "kitchen", "clear", "add_upper",
        ];
        for lang in LANGS {
            for key in KEYS {
                let value = t(lang, key);
                assert_ne!(value, *key, "{lang} left {key} untranslated");
                assert!(!value.is_empty(), "{lang} {key}");
            }
        }
        assert_eq!(t("en", "sold_out"), "Sold Out");
        assert_eq!(t("gu", "cash"), "રોકડ");
        assert_eq!(t("gu", "owed"), "બાકી");
        assert_eq!(t("hi", "sold_out"), "बिक गया");
    }

    #[test]
    fn format_money_ex_european_comma() {
        // Germany writes the same euro as 1.234,50.
        assert_eq!(format_money_ex(123_450, "EUR", ",", "."), "€1.234,50");
        // Switzerland groups with an apostrophe and keeps the dot decimal.
        assert_eq!(format_money_ex(123_450, "CHF", ".", "'"), "CHF1'234.50");
        // France groups with a space; India keeps the plain default.
        assert_eq!(format_money_ex(123_450, "EUR", ",", " "), "€1 234,50");
        assert_eq!(format_money(123_450, "INR"), "₹1,234.50");
        // Zero-decimal and three-decimal currencies group the same way.
        assert_eq!(format_money_ex(1_234_567, "JPY", ".", ","), "¥1,234,567");
        assert_eq!(format_money_ex(1_234_567, "KWD", ",", "."), "د.ك1.234,567");
        // No grouping mark leaves the digits alone; negatives keep the sign.
        assert_eq!(format_money_ex(123_450, "USD", ".", ""), "$1234.50");
        assert_eq!(format_money_ex(-123_450, "USD", ".", ","), "-$1,234.50");
    }

    #[test]
    fn parse_major_accepts_comma_decimal() {
        // Both habits reach the same paise.
        assert_eq!(parse_major_input("13,5", "EUR").unwrap(), 1_350);
        assert_eq!(parse_major_input("13.5", "EUR").unwrap(), 1_350);
        assert_eq!(parse_major_input("1.234,50", "EUR").unwrap(), 123_450);
        assert_eq!(parse_major_input("1,234.50", "USD").unwrap(), 123_450);
        assert_eq!(parse_major_input("1 234,50", "EUR").unwrap(), 123_450);
        assert_eq!(parse_major_input("1'234.50", "CHF").unwrap(), 123_450);
        // A lone mark three digits from the end is grouping, not a decimal.
        assert_eq!(parse_major_input("1.234", "EUR").unwrap(), 123_400);
        assert_eq!(parse_major_input("1,234", "USD").unwrap(), 123_400);
        // …except where the currency really carries three decimals.
        assert_eq!(parse_major_input("1,234", "KWD").unwrap(), 1_234);
        // Plain input and errors are untouched.
        assert_eq!(parse_major_input("50", "INR").unwrap(), 5_000);
        assert_eq!(parse_major_input("-13,5", "EUR").unwrap(), -1_350);
        assert!(parse_major_input("  ", "INR").is_err());
        assert!(parse_major_input("12.3456", "INR").is_err());
        assert!(parse_major_input("abc", "INR").is_err());
    }

    #[test]
    fn percent_parses_and_prints_as_basis_points() {
        assert_eq!(parse_percent_bps("10").unwrap(), 1_000);
        assert_eq!(parse_percent_bps("12.5").unwrap(), 1_250);
        assert_eq!(parse_percent_bps("12,5 %").unwrap(), 1_250);
        assert_eq!(parse_percent_bps("   ").unwrap(), 0);
        assert!(parse_percent_bps("120").is_err());
        assert!(parse_percent_bps("-5").is_err());
        assert!(parse_percent_bps("ten").is_err());
        assert_eq!(format_percent_bps(1_250, "."), "12.5");
        assert_eq!(format_percent_bps(1_250, ","), "12,5");
        assert_eq!(format_percent_bps(0, "."), "");
    }

    #[test]
    fn major_input_round_trips_through_the_local_decimal_mark() {
        let text = format_major_input_ex(1_350, "EUR", ",");
        assert_eq!(text, "13,5");
        assert_eq!(parse_major_input(&text, "EUR").unwrap(), 1_350);
        assert_eq!(format_major_input(1_350, "EUR"), "13.5");
        assert_eq!(format_major_input_ex(5_000, "EUR", ","), "50");
    }
}
