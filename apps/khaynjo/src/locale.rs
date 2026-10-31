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


/// Languages written right-to-left. Makepad shapes and bidi-orders the
/// glyphs itself; the app uses this to right-align static text (labels,
/// help prose, input fields) while a right-to-left language is active.
pub fn is_rtl(lang: &str) -> bool {
    let primary = lang
        .split(['-', '_'])
        .next()
        .unwrap_or(lang)
        .to_ascii_lowercase();
    matches!(
        primary.as_str(),
        "ar" | "ur" | "fa" | "he" | "dv" | "ps" | "sd" | "ks" | "yi" | "ckb" | "ug"
    )
}

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
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let decimals = currency_by_code(currency_code)
        .map(|c| c.decimals)
        .unwrap_or(2) as u32;
    let text = normalize_major_input(text.trim(), decimals as usize);
    if text.is_empty() {
        return Err("Amount required".into());
    }
    let negative = text.starts_with('-');
    let text = text.trim_start_matches('+').trim_start_matches('-');
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err("Invalid amount".into());
    }
    if text.matches('.').count() > 1 {
        return Err("Invalid amount".into());
    }
    let major = Decimal::from_str(text).map_err(|_| "Invalid amount".to_string())?;
    if major.fract() != Decimal::ZERO {
        let scale = major.normalize().scale();
        if scale > decimals {
            return Err("Too many decimal places".into());
        }
    }
    let mut minor = crate::precise::major_to_minor(major.abs(), decimals)?;
    if negative {
        minor = minor
            .checked_neg()
            .ok_or_else(|| "Amount overflow".to_string())?;
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
        "app_tagline" => "Restaurant orders, kitchen & books. Free offline.",
        "unlock" => "Unlock",
        "pin_hint" => "Owner PIN starts as 0000",
        "sell" => "Order",
        "buy" => "Buy",
        "books" => "Day book",
        "books_help" => "Today’s position. Order = orders in. Buy = purchases & expenses. Parties = credit customers & suppliers.",
        "buy_help" => "Expense vouchers and supplier bills (money out). On-account stays on the party ledger.",
        "sell_money_in" => "Take an order",
        "buy_money_out" => "Expense / supplier bill",
        "record_purchase" => "Record purchase",
        "add_expense" => "Expense voucher",
        "bank_book" => "Bank book",
        "credit_help" => "Party id is unique (names may repeat). A Customer runs a khata tab (owes us); a Supplier sells us stock (we owe them); both is fine. Ledgers stay separate; reports show net.",
        "party_kind_ph" => "Roles: Customer, Supplier, both…",
        "reports_help" => "Day book, debtors & creditors, bank book, expense vouchers, profit & loss.",
        "reports_today" => "Day book (today)",
        "reports_month" => "Profit & loss (month)",
        "reports_aging" => "Outstanding / Dr-Cr",
        "reports_balance_sheet" => "Balance sheet",
        "reports" => "Bank book & P&L",
        "refresh" => "Refresh",
        "undo" => "Undo",
        "start_tracking" => "Start tracking",
        "stock_minus" => "− Stock",
        "stock_plus" => "+ Stock",
        "settle_cash" => "Settle cash",
        "settle_card" => "Settle card",
        "items" => "Menu & items",
        "more" => "More",
        "your_items" => "Your menu",
        "items_help" => "Tap an item to edit it. New item clears the form. Search our dishes or type your own name and photo, set a price (0 is fine), then Add. Sales already recorded keep the name they were charged with.",
        "add_item" => "Add item",
        "new_item" => "New item",
        "adding_item" => "Adding a new item",
        "editing_item" => "Editing",
        "save" => "Save",
        "remove" => "Remove",
        // Remove archives: these rows are the items taken off the menu, and
        // Restore puts one back exactly as it was.
        "removed_items" => "Removed items",
        "removed_help" => "Restore puts the item back on the menu exactly as it was — price, photo and category. Our shelf of dishes is never removed, so a dish can always be added again.",
        "restore_item" => "Restore",
        "empty_catalog" => "No items yet. Open Menu & items, add one (name + price), then take an Order.",
        "total" => "Total",
        "cash" => "Cash",
        "upi" => "UPI",
        "card" => "Card",
        "credit" => "Parties",
        "clear" => "Clear",
        "ready_to_sell" => "Ready to book",
        "today_prefix" => "Today",
        "today_mid" => "today",
        "owed" => "outstanding",
        "round_word" => "round",
        "back" => "Back",
        "sold_out" => "Unavailable",
        "restock" => "Restock",
        "not_tracked" => "Not tracked",
        "in_stock" => "in stock",
        "marked_sold_out" => "Marked unavailable",
        "add_upper" => "ADD",
        "na" => "N/A",
        "qty" => "Wt/Qty",
        "unit_at" => "Rate @",
        "actual" => "Actual",
        "applied" => "Applied",
        "note_ph" => "Note (optional)",
        "rounded" => "ROUNDED",
        "split" => "Split",
        "send_kot" => "Send KOT",
        "send_kot_to" => "Send → {kitchens}",
        "send_kot_again" => "Add dishes for next KOT",
        "send_kot_held" => "Held dishes — unhold to send",
        "kot_dest_hint" => "Each kitchen prints on its own printers: {kitchens}",
        "kot_sent_hint" => "Last KOT went to {kitchens}. Add dishes, then send again.",
        "kot_unassigned" => "Kitchen (no station)",
        "kot_unassigned_held" => "Held {n} dish(es) without a station — unhold on the line to send",
        "kot_station_ph" => "Tap a kitchen in the list — applied to these dishes",
        "kot_set_station" => "Use this station",
        "kot_station_applied" => "Station set on {n} dish(es): {station}",
        "kot_station_cleared" => "Station cleared from {n} dish(es): {station}",
        "kot_station_needed" => "Type or pick a kitchen station first",
        "pos_tab_items" => "Added items",
        "pos_tab_payment" => "Payment",
        "hold_send" => "Hold",
        "hold_send_on" => "Held",
        "hold_send_meta" => "Held — not this KOT",
        "station_remove" => "Remove station",
        "station_remove_ph" => "Tap a station in the list to remove it",
        "station_removed" => "Station removed: {name}",
        "settings_hub_title" => "Settings",
        "settings_hub_help" => "Open one area at a time — less scrolling, clearer choices.",
        "settings_back" => "Back",
        "settings_sec_language" => "Language & currency",
        "settings_sec_country" => "Country & tax",
        "settings_sec_tills" => "Extra tills & product",
        "settings_sec_brand" => "Your brand",
        "settings_sec_units" => "Units & conversions",
        "settings_sec_pickers" => "Areas · stations · tax rates",
        "settings_sec_printers" => "Kitchen printing & printers",
        "settings_sec_bill" => "Bill format & schemes",
        "settings_sec_alerts" => "Alerts",
        "alerts_help" => "How long status messages stay on screen for you (this login).",
        "snack_secs_ph" => "Seconds (3–60, default 10)",
        "snack_secs_save" => "Save alert time",
        "snack_secs_saved" => "Alert time saved — {n} seconds",
        "settings_sec_backup" => "Backup & restore",
        "settings_sec_diag" => "Diagnostics",
        "diag_privacy_help" => "Logs stay on this device. Secrets (PIN, passwords) are never stored. Sharing a package off the device needs your explicit approval each time — never sold or given to third parties.",
        "diag_level" => "Detail level",
        "diag_level_error" => "Errors",
        "diag_level_warn" => "Warnings",
        "diag_level_info" => "Info",
        "diag_level_debug" => "Debug",
        "diag_support_start" => "Start 30-min support session",
        "diag_support_stop" => "End support session",
        "diag_support_started" => "Support session on — Debug detail for 30 minutes",
        "diag_support_ended" => "Support session ended",
        "diag_export_local" => "Export local copy",
        "diag_export_local_done" => "Local diagnostics ready at {path} — pick where to save a copy",
        "diag_approve_send" => "Approve share off this device",
        "diag_send_help" => "Approval is one package, then expires. Temporary — used only to fix or improve the product. Not shared with third parties. Works on Linux, Windows, macOS, and Android.",
        "diag_send_approved" => "Share approved for one package (expires soon)",
        "diag_export_send" => "Create share package (needs approval)",
        "diag_export_send_done" => "Share package ready at {path} — pick where to save, then send only to Kumkaj support",
        "diag_save_bundle" => "Save diagnostics copy",
        "diag_bundle_saved" => "Diagnostics copy saved to {path}",
        "diag_bundle_failed" => "Could not save the diagnostics copy — try again",
        "diag_clear" => "Clear diagnostic logs",
        "diag_cleared" => "Diagnostic logs cleared",
        "sidebar_settings" => "Sidebar",
        "sidebar_settings_help" => "Same icons as the side rail. Drag a row to reorder. Flip the switch to show or hide. Settings and Lock always stay on the bar.",
        "sidebar_always_on_short" => "Always on",
        "sidebar_back_settings" => "Back",
        "sidebar_shown" => "Shown",
        "sidebar_hidden" => "Hidden",
        "sidebar_always_on" => "Always on the bar",
        "sidebar_shown_hint" => "Shown on the side rail / bottom tabs",
        "sidebar_hidden_hint" => "Hidden from the bar",
        "sidebar_menu_items" => "Menu & Items",
        "sidebar_enable_all" => "Enable All",
        "sidebar_disable_all" => "Disable All",
        "sidebar_order_saved" => "Sidebar order saved",
        "sidebar_visibility_saved" => "Sidebar visibility saved",
        "amounts_hide" => "Hide",
        "amounts_show" => "Show",
        "printers_none_hint" => "Add a printer in step 1 first, then assign it in step 2",
        "counter" => "Counter",
        "dine_in" => "Dine-in",
        "takeaway" => "Takeaway",
        "order" => "Order",
        "tables" => "Tables",
        "kitchen" => "Kitchen",
        "sales" => "Sales",
        "sales_refunds" => "Sales & refunds",
        "day_close" => "Day close",
        "reports_expenses" => "Bank book & P&L",
        "nav_reports" => "Reports",
        "nav_settings" => "Settings",
        "nav_lock" => "Lock",
        "no_sale_yet" => "No sale yet",
        "favorites" => "Favorites",
        "favorites_help" => "Items you sell often, saved for your login. Add one to the bill, or build your list from all items below.",
        "your_favorites" => "Your favorites",
        "all_items" => "All items",
        "frequent" => "Frequent",
        "frequent_config_help" => "Most sold dishes from completed bills. Set how many days to look at and how many dishes to show.",
        "frequent_days_ph" => "Days (1)",
        "frequent_limit_ph" => "Items (24)",
        "frequent_apply" => "Apply",
        "frequent_saved" => "Frequent: last {d} day(s), top {n} — {c} dish(es)",
        "frequent_empty" => "No completed sales in this period yet — settle a bill (₹0 allowed when Settings says so) and they show here.",
        "zero_sales" => "Zero-value sales",
        "zero_sales_help" => "Allow dishes priced ₹0 and bills settled at ₹0 (complimentary / owner wish). On by default so free covers still appear in Frequent and day books.",
        "zero_sales_on" => "Allow ₹0",
        "zero_sales_off" => "Require price",
        "zero_sales_saved_on" => "Zero-value sales allowed",
        "zero_sales_saved_off" => "Sales must have a price above ₹0",
        "floor_areas_help" => "Areas are rooms or zones on the floor (Dine In, Parcel, AC …). Add an area first, then put tables in it — each area is a separate group on the floor.",
        "area_new_ph" => "New area / room (e.g. AC, Garden)",
        "add_area" => "Add area",
        "area_added" => "Area “{name}” ready — add tables into it",
        "no_favorites" => "No favorites yet — add items from the list below.",
        "add_to_bill" => "Add to bill",
        "added" => "Added",
        "owes_word" => "owes",
        "selected_word" => "Selected",
        "chip_clear_credit" => "clear — Credit or take advance",
        "chip_missing_customer" => "Selected customer missing — pick again",
        "chip_no_customer" => "No customer yet — type a name, then tap Credit",
        "credit_summary_fmt" => "{} customers · open credit {} · advances held {}",
        "recent_label" => "Recent:",
        "kot_new_waiting" => "NEW — waiting",
        "kot_preparing" => "PREPARING",
        "kot_ready_serve" => "READY — serve it",
        "kot_done_cap" => "DONE",
        "kot_cancelled" => "CANCELLED",
        "kot_start_btn" => "Start",
        "kot_mark_ready" => "Mark ready",
        "kot_done_btn" => "Done",
        "table_free" => "Free — tap to start an order",
        "start_order" => "Start order",
        "open_bill" => "Open bill",
        "collect_payment" => "Collect payment",
        "table_running" => "Running",
        "payment_pending" => "Payment pending",
        "collect_at_counter" => "collect at counter",
        "inventory" => "Inventory",
        "inventory_stock" => "Inventory & stock",
        "branches" => "Branches",
        "branches_org" => "Branches & org",
        "credit_customers" => "Parties (khata & suppliers)",
        "customer_name_ph" => "Party name",
        "new_customer_ph" => "New party name",
        "item_name_ph" => "Item name (e.g. Masala Dosa)",
        "item_price_ph" => "Price",
        "start_template" => "Product",

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
        "this_shop" => "Server",
        "extra_till" => "Client",
        "off" => "Off",
        "tables_kitchen" => "Tables & kitchen",
        "server" => "Server",
        "client" => "Client",
        "language_currency" => "Language & currency",
        "language" => "Language",
        "choose_language" => "Choose your language",
        "language_setup_hint" => "Follow this device’s language, or pick one yourself.",
        "lang_follow_platform" => "Device language",
        "lang_pick_user" => "Choose language",
        "lang_platform_now" => "Device language now",
        "selected_language" => "Selected",
        "continue" => "Continue",
        "this_device" => "THIS DEVICE",
        "device_use_q" => "How will this device be used?",
        "last_server_mode" => "Was Server — tap to continue",
        "last_client_mode" => "Was Client — tap to continue",
        "choose_server" => "Choose a server",
        "choose_server_hint" => "Open an existing server, or create one for another business.",
        "create_new_server" => "Create new server",
        "open_server" => "Open",
        "new_server_name_ph" => "New server name (optional)",
        "device_setup_help" => "Server keeps the books here. Client pairs to a Server.",
        "currency" => "Currency",
        "backup" => "Backup",
        "use" => "Use",
        "dont_use" => "Don't use",
        "add_branch" => "Add branch",
        "add_conversion" => "Add conversion",
        "add_dishes_items" => "Add dishes & items",
        "add_item_plain" => "Add item",
        "add_note" => "Add note",
        "add_opening_balance" => "Add opening balance",
        "add_staff" => "Add staff",
        "add_station" => "Add station",
        "add_table" => "Add table",
        "add_unit" => "Add unit",
        "add_use_name" => "Add / use this name",
        "allow_extra_tills" => "Allow extra tills",
        "amend" => "Amend",
        "apply_country_pack" => "Apply country pack",
        "attach_bill_photo" => "Attach bill photo",
        "attach_pdf" => "Attach PDF",
        "attach_photo" => "Attach photo",
        "attachments_last_expense" => "Attachments (last expense)",
        "bank" => "Bank",
        "bank_in" => "Bank in",
        "bank_out" => "Bank out",
        "bank_reconciliation" => "Bank reconciliation (tick off against your bank statement)",
        "bank_to_till" => "Bank → till",
        "book_add_dish" => "Book — add dish / item",
        "book_opening_credit" => "Book — opening credit",
        "branch_name_sell_ph" => "Branch name to sell from",
        "bring_my_book" => "Bring my book",
        "business_tax_id_ph" => "Business tax id",
        "caption_bank" => "BANK",
        "caption_bills" => "BILLS",
        "caption_outstanding" => "OUTSTANDING",
        "caption_payments_today" => "PAYMENTS TODAY",
        "caption_today" => "TODAY",
        "card_amount_ph" => "Card amount",
        "cash_amount_ph" => "Cash amount",
        "cash_memo" => "Cash memo",
        "choose_icon" => "Choose icon",
        "choose_logo" => "Choose logo",
        "clear_icon" => "Clear icon",
        "clear_photo" => "Clear photo",
        "copy_day_book_csv" => "Copy day book CSV",
        "copy_day_close" => "Copy day close",
        "copy_items_csv" => "Copy items CSV",
        "copy_outstanding_csv" => "Copy outstanding CSV",
        "copy_parties_csv" => "Copy parties CSV",
        "copy_party_ledger_csv" => "Copy party ledger CSV",
        "copy_receipt" => "Copy receipt",
        "share_whatsapp" => "Share WhatsApp",
        "lan_pages_heading" => "Guest pages on Wi‑Fi",
        "lan_pages_help" => "On this shop’s Wi‑Fi, guest pages work for display board, tokens, expo, kiosk, table QR, and feedback. Guests on mobile data need Cloud later. Card terminal / payment link is optional when the internet is up — it never blocks the counter.",
        "diag_send_heading" => "Share off this device",
        "kot_dest_heading" => "Kitchen station",
        "optional_login_heading" => "Optional login",
        "rights_heading" => "Rights",
        "ops_panel" => "Local ops",
        "crm_heading" => "Guests & messages",
        "crm_help" => "Birthday, loyalty, inactive and top guests — queue SMS or WhatsApp intents. Use {name} and {points} in the message. Owner taps send; nothing blasts by itself.",
        "crm_queue_sms" => "Queue SMS",
        "crm_queue_wa" => "Queue WhatsApp",
        "crm_seg_bday_week" => "Birthday week",
        "crm_seg_bday_month" => "Birthday month",
        "crm_seg_phone" => "With phone",
        "crm_seg_loyalty" => "Loyalty",
        "crm_seg_inactive" => "Inactive",
        "crm_seg_top" => "Top spenders",
        "birthday_ph" => "Birthday (MM-DD or YYYY-MM-DD)",
        "crm_message_ph" => "Happy birthday {name}! You have {points} points.",
        "till_apply_redeem" => "Apply on bill",
        "till_clear_redeem" => "Clear redeem",
        "till_wallet_topup" => "Top up wallet",
        "till_issue_voucher" => "Issue voucher",
        "till_points_ph" => "Points to redeem",
        "till_wallet_ph" => "Wallet amount to spend",
        "till_voucher_code_ph" => "Voucher code",
        "till_voucher_amt_ph" => "Voucher amount (blank = all)",
        "till_wallet_topup_ph" => "Top-up wallet amount",
        "till_voucher_issue_amt_ph" => "Issue voucher amount",
        "till_voucher_issue_code_ph" => "New voucher code",
        "split_evenly" => "Split evenly",
        "waitlist" => "Waitlist",
        "fire_course" => "Fire course",
        "hold_item" => "Hold",
        "waiter_call" => "Call waiter",
        "waitlist_add" => "Add to waitlist",
        "floor_view_grid" => "Grid",
        "floor_view_layout" => "Layout",
        "floor_layout_x" => "X",
        "floor_layout_y" => "Y",
        "floor_layout_w" => "W",
        "floor_layout_h" => "H",
        "floor_layout_save" => "Save layout",
        "floor_layout_open" => "Open table",
        "floor_layout_edit" => "Edit layout",
        "floor_layout_pick" => "Tap a table to edit its layout",
        "floor_layout_saved" => "Table layout saved",
        "floor_layout_nudge_left" => "← 20",
        "floor_layout_nudge_right" => "→ 20",
        "floor_layout_nudge_up" => "↑ 20",
        "floor_layout_nudge_down" => "↓ 20",
        "sibling_pickers_help" => "Tap On/Off like units — hide areas, stations, and tax rates from day-to-day pickers; pack defaults stay.",
        "bulk_price_apply" => "Apply bulk price",
        "card_terminal_optional" => "Card terminal / payment link (optional when internet available)",
        "copy_report" => "Copy report",
        "counted_cash_ph" => "Counted cash in drawer",
        "country_tax" => "Country & tax",
        "create_backup" => "Create backup",
        "credit_limit_heading" => "Credit limit (selected / named party)",
        "day_close_summary" => "Day close summary",
        "delete_party" => "Delete selected party",
        "delivery" => "Delivery",
        "done_open_books" => "Done — open books",
        "drawer_count" => "Drawer count (optional)",
        "expense_heading" => "Expense",
        "export_day" => "Export day",
        "extra_note_bill_ph" => "Extra note on this bill / sale",
        "extra_note_expense_ph" => "Extra note on last expense",
        "extra_tills_heading" => "Extra tills",
        "other_devices_help_heading" => "Set up other devices",
        "other_devices_help" => "On this device (Server): Extra tills → Allow extra tills. Note the pairing code and Wi‑Fi address. Tap Allow other devices to connect if it appears. On a computer, tap Start with this computer. On each other device: install the app, choose Client Mode, Find the shop's server (or type the address), enter the pairing code, Pair now. Orders wait on the device if Wi‑Fi drops and go when the Server answers again.",
        "forget_shop" => "Forget this shop",
        "goods_received" => "Goods received",
        "import_day" => "Import day",
        "import_items_csv_plain" => "Import items CSV…",
        "import_parties_csv" => "Import parties CSV…",
        "item_name" => "Item name",
        "kitchen_printing" => "Kitchen printing",
        "login_id_ph" => "Login id",
        "mark_cleared" => "Mark cleared",
        "new_party_clear" => "New party (clear form)",
        "new_party_kind_supplier_ph" => "New party kind (Supplier)",
        "next_country" => "Next country",
        "next_order" => "Next order (clear desk)",
        "note_gstin_ph" => "Note / GSTIN / agreement reminder",
        "notes_attach_party" => "Notes & attachments (selected party)",
        "notes_attach_sale" => "Notes & attachments (selected sale)",
        "on_account_ap" => "On account (AP)",
        "org_optional_ph" => "Organization (optional)",
        "paid_by" => "Paid by",
        "paid_now" => "Paid now",
        "pair_now" => "Pair now",
        "pairing_code_ph" => "Pairing code",
        "party_name_selected_ph" => "Party name (or use selected)",
        "password_ph" => "Password",
        "pay_ap_card" => "Pay AP card",
        "pay_ap_cash" => "Pay AP cash",
        "pay_ap_heading" => "Pay what we owe them (AP)",
        "pay_ap_upi" => "Pay AP UPI",
        "pay_selected_heading" => "Pay selected (settle debt / take advance)",
        "pick_from_items" => "Pick from our items",
        "prev_country" => "Previous country",
        "print_open" => "Print / open",
        "reason_amend_ph" => "Reason for amend / void / refund",
        "reason_note_ph" => "Reason / note (wastage, count, …)",
        "record_day_close" => "Record day close",
        "refresh_from_shop" => "Refresh from shop",
        "refund" => "Refund",
        "remove_selected" => "Remove selected",
        "request_bill" => "Request bill",
        "restaurant_name_ph" => "Restaurant name (optional)",
        "restore_backup" => "Restore backup",
        "role_both" => "Both",
        "role_cashier" => "Cashier",
        "role_customer" => "Customer",
        "role_manager" => "Manager",
        "rollup" => "Roll-up",
        "save_brand" => "Save brand",
        "save_business_id" => "Save business id",
        "save_business_name" => "Save business name",
        "save_features" => "Save features",
        "save_names" => "Save names",
        "second_way" => "Second way",
        "server_ip_ph" => "Server IP (e.g. 192.168.1.10)",
        "set_credit_limit" => "Apply limit to party",
        "set_login_selected" => "Apply login to selected",
        "shop" => "Shop",
        "show" => "Show",
        "spreadsheet_csv" => "Spreadsheet (CSV)",
        "start_fresh" => "Start fresh",
        "stop_extra_tills" => "Stop extra tills",
        "backup_auto" => "Backed up automatically every day · last {when} · the last 14 days are kept in {dir}.",
        "backup_auto_none" => "Backed up automatically every day — the first one runs in a moment.",
        "backup_copy_to" => "Copies also go to {dir}.",
        "backup_copy_btn" => "Also copy backups to a folder…",
        "autostart_off" => "Start with this computer",
        "autostart_on" => "Starts with this computer ✓ — tap to stop",
        "autostart_done_on" => "This computer now opens the app when it starts — the shop's server is back after a power cut.",
        "autostart_done_off" => "This computer no longer opens the app when it starts.",
        "device_forgotten" => "The shop forgot this device — pair it again with the shop's current code. Orders waiting on it are kept.",
        "lan_offline" => "Shop server unreachable",
        "forget_device" => "Forget",
        "last_seen" => "seen",
        "new_pair_code" => "New pairing code",
        "kot_waiting" => "Waiting to send",
        "lan_offline_waiting" => "Shop server unreachable · {n} waiting",
        "lan_connecting" => "Connecting to shop…",
        "lan_sending" => "{n} sending",
        "lan_sign_in_to_send" => "Sign in again to send {n}",
        "lan_pair_again" => "Pair this device again",
        "lan_update_app" => "Update the app — this device and the shop's server have different versions",
        "allow_firewall" => "Allow other devices to connect",
        "firewall_blocked" => "This computer's firewall is blocking other devices — tap Allow other devices to connect.",
        "firewall_waiting" => "Waiting for the admin prompt…",
        "lan_join_help" => "On the other device: Client Mode → Find the shop's server (or type {addr}) → pairing code {code}.",
        "lan_pair_status" => "Wi‑Fi {addr}",
        "lan_join_help_no_addr" => "Connect this device to Wi‑Fi for an address.",
        "lan_no_address" => "No Wi‑Fi address yet — connect to the shop Wi‑Fi.",
        "lan_not_listening" => "Not accepting devices (port {port} busy). Restart the app.",
        "lan_tills_off" => "Extra tills are off. Allow them to let another device join.",
        "paired_with" => "Paired · {url}",
        "not_paired" => "Not paired",
        "choose_device_use" => "Choose Server or Client.",
        "paired_note" => "Paired. Unlock to sell, or Refresh for the latest.",
        "last_update_at" => "Last update: {time}.",
        "pair_intro" => "Find the server, or type its address and pairing code.",
        "queue_waiting" => "Waiting to reach the shop ({n}):",
        "queue_tries" => "Tried {n} times · last answer: {e}",
        "save_backup_copy" => "Save a backup copy…",
        "backup_copy_saved" => "Backup copy saved — keep it off this device (a USB stick or another device)",
        "backup_copy_overdue" => "No backup copy has left this device for {n} days — Settings → Backup",
        "backup_copy_failed" => "The backup copy could not be saved there — pick another place",
        "clock_off" => "This device's clock is {min} min off the shop's server — set the right time on this device",
        "printers_heading" => "Kitchen printing & printers",
        "printers_help" => "Add each printer once — network host:9100 or a system name from Find. Pick how to send (thermal ESC/POS or system print) per printer. Then assign any printer to Bill or a kitchen — your choice, not fixed by job type.",
        "printers_pool_heading" => "1. Your printers",
        "printers_pool_empty" => "(none yet — add one below)",
        "printers_routes_help" => "Tap Bill or a kitchen, then tap a printer name to assign it there (replaces the previous printer for that place). Clear printers for this place removes every printer from the selected Bill/kitchen so that place has none until you assign again.",
        "station_manage_heading" => "Add or remove a kitchen station",
        "printers_routes_heading" => "2. What prints where",
        "route_editing" => "Assigning: {name} — tap a printer below",
        "route_assigned" => "{place} → {printer}",
        "printer_label_ph" => "Name (e.g. Kitchen)",
        "printer_target_ph" => "Address or system name",
        "printer_target_default" => "this computer's usual printer (blank address)",
        "printer_wire_heading" => "How to send",
        "printer_wire_escpos" => "Thermal ESC/POS",
        "printer_wire_os" => "System print",
        "printer_wire_escpos_short" => "thermal",
        "printer_wire_os_short" => "system",
        "add_printer" => "Add printer",
        "remove_printer" => "Remove",
        "route_pick_ph" => "Bill or a kitchen station",
        "route_bill" => "Bill",
        "route_default_kitchen" => "No station (unassigned dishes)",
        "route_kot_station" => "Kitchen",
        "route_dont_print" => "No printer assigned",
        "stations_heading" => "Kitchen stations",
        "stations_help" => "Each station can have its own printers (step 2 on Kitchen printing & printers). Add Tandoor, Fryer, Cold…",
        "stations_none" => "No stations yet — add one below. Menu items can use a station so their KOT prints there.",
        "route_up" => "Up",
        "route_down" => "Down",
        "route_remove" => "Remove last printer",
        "route_clear" => "Clear printers for this place",
        "save_printers" => "Save printers",
        "print_test_page" => "Print a test page",
        "printers_saved" => "Saved",
        "test_page_sent" => "Test page sent — check the printer",
        "printers_found" => "Found — tap one to fill the form: {list}",
        "find_printers" => "Find printers",
        "printers_looking" => "Looking…",
        "printers_tap_find" => "Tap Find to list printers this device can use.",
        "printers_none_found" => "None found — type an IP (e.g. 192.168.1.50) or a system printer name.",
        "printer_from_find" => "{name} — tap Add printer to save it",
        "kot_printer_ph" => "Printer for KOTs",
        "bill_printer_ph" => "Printer for bills",
        "kot_print_on" => "Kitchen printing: On",
        "kot_print_off" => "Kitchen printing: Off",
        "queue_refused" => "Not accepted by the shop ({n}) — check each one at the counter:",
        "clear_not_accepted" => "Checked — clear this list",
        "device_name_ph" => "This device's name",
        "lan_last_update" => "last update {time}",
        "lan_not_accepted" => "{n} not accepted",
        "connected_now" => "connected now",
        "backup_failed" => "The last automatic backup failed: {e}",
        "backup_overdue" => "No backup for {n} days — keep the app open on this device so it can back up.",
        "disk_low" => "Only {mb} MB free on this device — backups will soon fail. Free up some space.",
        "last_seen_devices" => "Last seen: {time}",
        "sync_interval" => "Sync every {secs}s",
        "connection_to_shop" => "Connection to the shop",
        "supplier_name_ph" => "Supplier name (or party)",
        "supplier_ph" => "Supplier",
        "switch_device" => "Switch this device",
        "take_payment" => "Take payment",
        "tax_added_till" => "Tax added at till",
        "tax_inside_price" => "Tax inside price",
        "tax_invoice" => "Tax invoice",
        "till_to_bank" => "Till → bank",
        "units_heading" => "Units",
        "unlock_after_pairing" => "Enter PIN",
        "unlock_with_login" => "Unlock with login",
        "unmark" => "Unmark",
        "update_my_password" => "Update my password",
        "update_my_pin" => "Update my PIN",
        "duress_heading" => "Emergency unlock PIN (Owner only)",
        "duress_help" => "Optional. If this PIN is typed at unlock instead of yours, the till opens with no sales history — menu, prices, tables and staff stay. Use only if you are forced to unlock. Leave blank and save to turn off. Never the same as anyone's PIN.",
        "duress_pin_ph" => "Emergency PIN (4–12 digits)",
        "duress_confirm_ph" => "Type again to confirm",
        "save_duress_pin" => "Save emergency PIN",
        "clear_duress_pin" => "Turn off emergency PIN",
        "duress_on" => "Emergency unlock PIN is on",
        "duress_off" => "Emergency unlock PIN is off",
        "duress_saved" => "Emergency unlock PIN saved",
        "duress_cleared" => "Emergency unlock PIN turned off",
        "upi_amount_ph" => "UPI amount",
        "void" => "Void",
        "write_off_ap" => "Write off AP",
        "write_off_heading" => "Write off (clear without payment)",
        "write_off_nil" => "Write off / nil balance",
        "your_brand" => "Your brand (in-app)",
        "your_categories" => "Your categories",
        "account_kind" => "Account kind (tap one)",
        "actual_total_cap" => "ACTUAL TOTAL",
        "amend_help" => "Select a sale to amend, void, or refund. Amendments keep full history. Rounded Total starts equal to Applied — edit only to round. Void and refund may need an Owner or Manager PIN (typed above; recorded with your name).",
        "applied_total_cap" => "APPLIED TOTAL",
        "bank_balance" => "Bank balance",
        "bank_net_today" => "Bank net today",
        "branch_export_help" => "Export writes branch_snapshot_CODE_DATE.json into the backups folder and copies it to the clipboard. To import another outlet's day, save its file as import_branch_snapshot.json in the same folder, then tap Import.",
        "brand_shown_help" => "Shown on screens inside the app. The installed app name and icon on this device stay as the platform listing.",
        "app_icon" => "App icon",
        "logo_help" => "Optional shop mark on screens in the app.",
        "onboard_layout" => "What you get",
        "item_photo_help" => "Optional. Without a photo we show initials from the item name.",
        "stamp_heading" => "Stamp",
        "stamp_image_help" => "Optional stamp image on printed bills and the guest display.",
        "payment_qr_heading" => "Payment QR",
        "payment_qr_image_help" => "Static QR on bills and guest display; otherwise paying uses a dynamic UPI QR from the VPA above.",
        "cash_in_hand" => "Cash in hand (till)",
        "change_my_pin_heading" => "Change my PIN (only yours)",
        "connect_to_server" => "Connect to the Server",
        "counter_locked" => "Counter 01 · Locked",
        "country_pack_help" => "A country pack sets the tax name and maths, the instant-pay rail, and the business id printed on bills. Kumkaj does not file taxes or issue government e-invoices anywhere.",
        "currency_use_help" => "Scroll and tap Use for any supported currency.",
        "day_close_help" => "Recording a day close saves this summary (and drawer variance if counted) to the audit log. It does not stop selling.",
        "expenses_word" => "Expenses",
        "feature_access_help" => "Feature access for selected staff (tap after adding)",
        "footer_local" => "Local SQLite · Free Forever · Zero Cloud",
        "fresh_book_khaynjo" => "",
        "fresh_selected" => "Fresh selected — catalogue stays empty. Tap Done when ready.",
        "gross_word" => "Gross",
        "how_start" => "How do you want to start?",
        "kot_print_help_khaynjo" => "When On, each kitchen prints its own KOT paper as soon as you Send.",
        "later_country_pack" => "",
        "local_badge" => "LOCAL",
        "mix_type_hint_khaynjo" => "Split across cash / UPI / card / credit if needed. Credit needs a customer above.",
        "money_in_sales" => "Money in (sales)",
        "net_word" => "Net",
        "offline_badge" => "OFFLINE",
        "optional_login_help" => "Optional login (id + password). Owner sets once; only the user can change password later.",
        "or_unlock_login" => "Or unlock with login id + password",
        "permission_preset" => "Permission preset (what they can access)",
        "preset_help_khaynjo" => "Cashier: sell. Manager: sell + items + void/amend.",
        "product_locked_khaynjo" => "Khaynjo",
        "product_only_khaynjo" => "This app is Khaynjo only — restaurant operating system. Other trade faces ship as separate Kumkaj products.",
        "purchases_cogs" => "Purchases (COGS)",
        "qty_cap" => "QTY",
        "restore_help" => "Restore: pick a backup. The books open as they were then; the books as they are now are saved first, so this can be undone.",
        "rounded_total" => "Rounded Total",
        "setup_app_khaynjo" => "Set up Khaynjo",
        "shop_logo_optional" => "Shop logo (optional)",
        "stock_type_help" => "Type the item name (part of it is enough), a quantity, then adjust. Items that are not counted yet need Start tracking once.",
        "team_owner_help" => "Owner creates each person with a name, role title, permission preset, and starting PIN. Owner cannot change anyone's PIN later — only that person can update their own PIN.",
        "they_owe_us" => "They owe us",
        "unit_at_cap" => "Unit @",
        "units_help" => "Codes for item labels (kg, piece, bag). Every unit must link — directly or through other units — to a standard (kg · cm · ml · pc), e.g. 1 crate = 2 box and 1 box = 20 cm. Conversions: 1 from = factor to. Currencies are never units.",
        "units_picker_help" => "Tap On/Off to show a unit in Buy and Menu pickers. Off units still convert.",
        "unit_picker_on" => "On",
        "unit_picker_off" => "Off",
        "we_owe_them" => "We owe them",
        "who_unlocking" => "Who is unlocking?",
        "locked_word" => "Locked",
        "show_pin" => "Show PIN",
        "hide_pin" => "Hide PIN",
        "set_owner_pin" => "SET OWNER PIN",
        "enter_owner_passcode" => "ENTER OWNER PASSCODE",
        "enter_staff_passcode" => "ENTER STAFF PASSCODE",
        "pin_count_zero" => "0 / 12 digits · at least 4, then OK",
        "pin_count_need" => "{} / 12 digits · need {} more",
        "pin_count_full" => "12 / 12 digits · press OK",
        "pin_count_ready" => "{} / 12 digits · press OK when ready",
        "pin_hint_choose" => "Choose 4–12 digits · Enter or Unlock to save",
        "pin_hint_new" => "New 4–12 digit Owner PIN · Enter or Unlock",
        "pin_hint_shop_code" => "Shop device · pairing code {}",
        "pin_hint_books" => "This device holds the books",
        "pin_hint_paired" => "Extra till paired — unlock to sell",
        "pin_hint_pair_first" => "Extra till — pair to the shop device first",
        "unlock_to_continue" => "Unlock to continue",
        "no_staff_yet" => "No staff yet — unlock as Owner, then Team",
        "status_server_set_pin" => "Server Mode — set Owner PIN",
        "status_server_tills" => "Server Mode — extra tills can join. Unlock",
        "status_server_unlock" => "Server Mode — unlock as Owner",
        "status_client_unlock" => "Client Mode — unlock to sell",
        "icon_new_selected" => "Icon: new file selected — tap Save brand",
        "icon_custom" => "Icon: custom venue mark",
        "icon_default" => "Icon: Kumkaj mark (default) — choose a custom one",
        "logo_yours" => "Logo: your shop logo — shown on screens and receipts",
        "logo_default" => "Logo: Kumkaj mark (default) — choose your own if you have one",
        "tmpl_current" => "Current: {} — {}",
        "tmpl_selected" => "Selected: {} — {}",
        "setup_complete" => "Setup complete",
        "onboard_fresh_tmpl" => "Fresh · {} — catalogue starts empty; add items in Items, then Done.",
        "onboard_book_tmpl" => "Book · {} — add items and opening credit, then Done.",
        "onboard_template_tmpl" => "Name the place if you want, then pick how to start.",
        "tmpl_blurb" => "Tables, ordering, kitchen tickets, billing, and books.",
        "tmpl_how" => "Sell at the counter; Floor for tables & KOT; Books for the day book.",
        "reports_facet_title" => "Reports",
        "reports_facet_help" => "Day books, party aging, bank posts, and period P&L.",
        "against_bill_ph" => "Against bill # above (optional)",
        "aliases_ph" => "Aliases — other names, comma separated",
        "amount_blank_full_ph" => "Amount (blank = full balance)",
        "amount_owed_ph" => "Amount owed",
        "amount_ph" => "Amount",
        "amount_received_ph" => "Amount received",
        "amount_we_pay_ph" => "Amount we pay",
        "business_name_ph" => "Business name",
        "category_new_ph" => "Category (optional, created if new)",
        "code_bag_ph" => "Code (bag)",
        "conv_factor_ph" => "Factor (25 = 1 bag → 25 kg)",
        "country_code_ph" => "Country code e.g. IN, GB, US, AE, BR",
        "credit_khata_amount_ph" => "Credit / khata amount",
        "credit_limit_ph" => "Credit limit (0 = none)",
        "from_code_ph" => "From code (bag)",
        "item_name_lang_ph" => "Item name — type it in your language",
        "login_id_opt_ph" => "Login id (optional)",
        "manager_pin_ph" => "Manager PIN to approve (only if you cannot void)",
        "new_branch_name_ph" => "New branch name",
        "new_password_ph" => "New password (optional)",
        "new_pin_ph" => "New PIN (4–12 digits)",
        "note_bank_ph" => "Note (deposit, withdrawal, …)",
        "note_buy_expense_ph" => "Note (fuel, rent, …)",
        "note_expense_ph" => "Note (fuel, office, …)",
        "options_khaynjo_ph" => "Options — Size*: Small=199, Large=399 | Toppings+3: Cheese=40, Olive=30 | Extra cheese=40",
        "part_second_way_ph" => "Part by a second way (optional)",
        "phone_ph" => "Phone",
        "place_name_ph" => "Name of this place",
        "qty_example_ph" => "Qty (e.g. 1.5 for kg)",
        "quantity_ph" => "Quantity",
        "reason_required_ph" => "Reason (required)",
        "role_title_ph" => "Role title (e.g. Barista)",
        "service_pct_ph" => "Service % optional 0/10/12",
        "short_code_ph" => "Short code (MG02)",
        "staff_name_ph" => "Staff name / id",
        "starting_password_ph" => "Starting password (optional)",
        "starting_pin_ph" => "Starting PIN (4–12 digits)",
        "station_name_ph" => "Station name (e.g. Tandoor, Fryer, Cold)",
        "station_new_ph" => "Kitchen station (optional, e.g. Tandoor — created if new)",
        "table_name_ph" => "Table name (e.g. Patio 1, Counter 2)",
        "tax_rate_ph" => "Tax rate name or % — Standard, Reduced, 0, 5, 12, 18, 20",
        "to_code_ph" => "To code (kg)",
        "uncleared_no_ph" => "Uncleared # above",
        "unit_cost_ph" => "Unit cost",
        "unit_name_bag_ph" => "Name (Bag)",
        "unit_plate_ph" => "Unit (plate / pc — blank = each)",
        "pay_credit" => "Credit",
        "book_kind_today" => "Today books",
        "book_kind_month" => "This month books",
        "book_kind_aging" => "Outstanding / debit-credit",
        "book_kind_balance" => "Balance sheet",
        "order_help" => "Tap an item to add it.",
        "floor_help" => "Tap a table to take or change the order. Colours show free → seated → kitchen → ready → eating → bill → paid. Waitlist, reservations, and add-tables are in the sections below.",
        "kitchen_help" => "Kitchen tickets (KOT), newest last. Start → Ready → Done moves a ticket along. Tickets never show prices.",
        "kitchen_empty" => "No open tickets. Send a KOT from the Ordering screen.",
        "add_a_table" => "Add a table",
        "add_station_heading" => "Add a kitchen station",
        "categories_help" => "Tap Use to file the item above under that category. Photo gives the category a picture; without one it shows its initials.",
        "dish_search_ph" => "Item name — search our dishes or type your own",
        "tile_free" => "Free",
        "tile_occupied" => "Occupied",
        "tile_food_ready" => "Food ready",
        "tile_bill_printed" => "Bill printed",
        "tile_paid" => "Paid",
        "tile_bill_btn" => "Bill",
        "tile_occupy_btn" => "Seat guests",
        "tile_free_btn" => "Free table",
        "tile_ready_btn" => "Free table",
        "tile_undo_bill" => "Undo bill",
        "summary_busy" => "tables busy",
        "summary_to_collect" => "to collect",
        "summary_on_floor" => "on the floor",
        "floor" => "Floor",
        "sizes_as_items_ph" => "Sizes as separate items (optional) — Half=120, Full=200",
        "dish_group_ph" => "Dish group (optional) — e.g. Paneer Tikka",
        "discount" => "Discount",
        "remove_discount" => "Remove discount",
        "apply_discount" => "Apply discount",
        "save_opening_cash" => "Save opening cash",
        "close_shift_print" => "Close shift and print slip",
        "area_prices_ph" => "Area prices (optional) — AC=220, Parcel=210",
        "discount_ph" => "10% or 50",
        "discount_reason_ph" => "Reason (e.g. regular guest, late food)",
        "opening_float_ph" => "Cash put in the drawer this morning",
        "shift_counted_ph" => "Cash counted at handover",
        "opening_float" => "Opening cash (float)",
        "shift_handover" => "Shift handover",
        "shift_since" => "This shift since",
        "expected_in_drawer" => "Expected in drawer",
        "right_discount" => "Give a discount",
        "floor_in_kitchen" => "In kitchen",
        "floor_eating" => "Served",
        "pick_category" => "Pick a category",
        "close_category_list" => "Close category list",
        "new_category_row" => "New category — created when you save the item",
        "your_category" => "Your category",
        "suggested_category" => "Suggested — from our dishes",
        "category_search_ph" => "Category — search yours or type a new one",
        "line_sent_state" => "Sent to kitchen",
        "line_nc_state" => "Complimentary (NC) — not charged",
        "cancel_item_short" => "Cancel",
        "nc_short" => "NC",
        "charge_again_short" => "Charge",
        "move_short" => "Move",
        "cancel_item_title" => "Cancel this item — the kitchen gets a cancel slip",
        "nc_item_title" => "Make this item complimentary (NC)",
        "charge_again_title" => "Charge this item again",
        "move_item_title" => "Move this item to another table",
        "already_cooked" => "Already cooked (wastage)",
        "not_cooked_yet" => "Not cooked yet",
        "return_to_customer" => "Return to customer",
        "short_by" => "Short by",
        "tip_not_sale" => "Tip (staff's, not a sale)",
        "right_settle" => "Take payment",
        "right_cancel_sent" => "Cancel a sent item",
        "right_complimentary" => "Complimentary (NC)",
        "right_move_table" => "Move / merge tables",
        "right_reprint" => "Print bill again",
        "right_allowed" => "Allowed",
        "right_with_approval" => "With approval",
        "right_denied" => "Not allowed",
        "editing_staff" => "Rights for",
        "finding_server" => "Looking for the shop's server on this Wi‑Fi…",
        "no_server_found" => "None found. On the Server: Settings → Extra tills → Allow extra tills (same Wi‑Fi). Or type the IP shown there.",
        "print_bill" => "Print bill",
        "move_merge_table" => "Move / merge table",
        "move_help" => "Move this table's order to a free table, or merge it into a running one.",
        "move_target_ph" => "Table name (e.g. D5, P2)",
        "approval_pin_ph" => "Manager PIN (only if asked)",
        "move_here" => "Move here",
        "merge_into" => "Merge into it",
        "line_reason_ph" => "Reason (e.g. guest changed mind, out of stock)",
        "line_qty_ph" => "How many to cancel (blank = all)",
        "line_target_ph" => "Move to table (e.g. D5)",
        "confirm" => "Confirm",
        "menu_search_ph" => "Search item or short code",
        "customer_paid_ph" => "Customer paid",
        "tip_ph" => "Tip",
        "table_area_ph" => "Area (e.g. Dine In, Parcel, AC)",
        "table_range_ph" => "Many at once: D1-D20",
        "add_these_tables" => "Add these tables",
        "remove_these_tables" => "Remove these tables",
        "table_remove_ph" => "Remove: T1-T24, or one table's name",
        "find_server" => "Find the shop's server",
        "bill_format" => "Bill format",
        "bill_format_help" => "Printed under your restaurant's name on every bill. Separate lines with | (e.g. MG Road, Rajkot | Ph 98250 00000).",
        "bill_header_ph" => "Address | phone",
        "fssai_ph" => "FSSAI licence no. (India)",
        "bill_footer_ph" => "Footer (e.g. Thank you, visit again)",
        "save_bill_format" => "Save bill format",
        "print_brand_kit" => "Print brand kit",
        "print_brand_kit_help" => "Optional stamp and payment QR images on bill faces and the LAN guest display. Dynamic UPI QR from the VPA above still works when no payment QR image is set.",
        "choose_stamp" => "Choose stamp",
        "clear_stamp" => "Clear stamp",
        "stamp_set" => "Stamp: on file — shown on guest display and named on printed bills",
        "stamp_none" => "Stamp: none",
        "choose_payment_qr" => "Choose payment QR",
        "clear_payment_qr" => "Clear payment QR",
        "payment_qr_set" => "Payment QR: on file — shown on guest display (replaces amount UPI QR there)",
        "payment_qr_none" => "Payment QR: none — amount QR from UPI ID when paying",
        "aggregator_import" => "Aggregator orders (file)",
        "aggregator_import_help" => "Optional Local import: put import_aggregator_orders.json in the backups folder (array or {\"orders\":[…]}), then tap Import. Never blocks the counter; live Zomato/Swiggy is Cloud.",
        "import_aggregator_file" => "Import aggregator file",
        "team_pick_ph" => "Staff name to edit",
        "select_staff" => "Select this staff",
        "rights_help" => "Rights — tap to change: Allowed → With approval (a manager's PIN at that moment) → Not allowed.",
        "dish_picker_help" => "Search or browse our dishes and tap one to fill its name and photo — or keep your own name. Set your price (0 is fine), then Add item.",
        "close_dish_list" => "Close dish list",
        "own_item_name" => "Use this as your own item name",
        "on_your_menu" => "On your menu",
        "edit" => "Edit",
        "schemes" => "Schemes / offers",
        "schemes_help" => "BOGO, percent-off, and fixed combos. Combos expand to kitchen lines on Send KOT; components stay held until the combo fires. Example: Buy 2 Thali get 1 Chaas",
        "scheme_bogo_ph" => "Buy 2 Thali get 1 Chaas",
        "add_scheme" => "Add BOGO",
        "schedule_ph" => "Hours / happy hour — Mon-Fri 07:00-11:00 or HappyHour 17:00-19:00=150",
        "upi_vpa_ph" => "UPI ID (VPA)",
        "upi_payee_ph" => "UPI payee name (optional)",
        "upi_qr" => "UPI QR",
        "show_upi_qr" => "Show UPI QR",
        "close_upi_qr" => "Close",
        "upi_qr_help" => "Guest scans and pays. Tap Take payment after they pay. Second screen: open Server URL /display on LAN.",
        "kot_recall" => "Recall",
        "recently_done" => "Recently done",
        "recall_kot_ph" => "KOT # to recall",
        "role_help_customer" => "Role: Customer — khata (they owe us). Same party can also supply stock.",
        "role_help_both" => "Roles: Customer + Supplier — one party, khata and payable ledgers, net in Reports.",
        "role_help_supplier" => "Role: Supplier — we owe them for purchases.",
        "dine_in_pick_table" => "Dine-in — pick a table on Floor",
        "choose_table" => "Choose a table",
        "change_table" => "Change table",
        "pick_table_banner" => "Pick a table for this order",
        "items_word" => "items",
        "items_section" => "Items",
        "back_to_order" => "Back to order",
        "new_items_head" => "Not sent yet",
        "price_ph" => "Price",
        "start_with_template" => "Start with our menu template",
        "template_heading" => "Our menu template",
        "template_info" => "{c} categories and {d} dishes with photos, ready to add. Prices start at ₹0 — set yours after.",
        "template_import" => "Import complete template",
        "template_confirm" => "Add {d} dishes with their categories? Dishes you already have stay as they are.",
        "template_yes" => "Yes, import",
        "template_done" => "Every dish from our template is on your menu.",
        "template_imported" => "Added {d} dishes and {c} categories — set their prices on this screen",
        "resv_heading" => "Reservations",
        "resv_name_ph" => "Guest name",
        "resv_phone_ph" => "Phone (optional)",
        "resv_guests_ph" => "Guests",
        "resv_when_ph" => "When — 20:30, tomorrow 13:00",
        "resv_table_ph" => "Table (optional)",
        "resv_add" => "Book",
        "resv_seat" => "Seated",
        "resv_none" => "No bookings coming up.",
        "guests_word" => "guests",
        "delivery_by_ph" => "Delivery by (rider or staff name)",
        "kitchen_all_day" => "To cook now: {items}",
        "split_bill" => "Split bill",
        "split_go" => "Make separate bill",
        "cancel" => "Cancel",
        "split_help" => "Tap the dishes one guest pays for · {n} picked",
        "split_send_first" => "Send KOT first — only dishes the kitchen has can go to a separate bill",
        "split_pick_some" => "Tap the dishes for the separate bill first",
        "split_done" => "Separate bill made — it's on the floor next to this table",
        "split_marked" => "separate bill",
        "order_empty_pick" => "Order empty — tap Add dishes above to pick dishes.",
        "add_dishes" => "Add dishes",
        "order_so_far" => "On this order: {n} · {total}",
        "no_dish_match" => "No dish matches “{q}”",
        "pick_dishes_help" => "Tap a dish to add it · pick a category to narrow the list",
        "pick_done" => "Done",
        "dishes_n" => "{n} dishes",
        "recipe_ph" => "Recipe (optional) — one portion uses: Rice=0.2, Paneer=0.15",
        "reorder_ph" => "Buy more at",
        "set_reorder" => "Set reorder level",
        "low_stock_alert" => "Running low — buy soon: {items}",
        "unpriced_note" => "{n} dishes on your menu have no price yet — they bill at 0 until you set one.",
        "show_unpriced" => "Show only dishes without a price",
        "show_all_dishes" => "Show all dishes",
        "role_waiter" => "Waiter",
        "role_kitchen" => "Kitchen",
        "role_title_rest_ph" => "Role title (e.g. Captain, Head cook)",
        "preset_help_rest" => "Waiter: takes orders and runs tables. Cashier: also takes payment. Kitchen: the kitchen screen. Manager: also the menu, cancels and discounts.",
        "kind_raw" => "Raw material",
        "kind_resale" => "Bought & resold",
        "kind_dish" => "Dish we make",
        "buy_unit_ph" => "Unit (kg, litre, pcs)",
        "add_another_way" => "+ Add another way",
        "parts_status" => "Entered {a} of {b} · {c} left",
        "save_expense" => "Save expense",
        "rest_ph" => "Rest",
        "low_badge" => "Low",
        "stock_empty" => "Nothing in stock yet — what you record on Buy shows up here.",
        "recent_changes" => "Recent changes",
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
                ("device_setup_help", "सर्वर यहाँ किताबें रखता है। क्लाइंट सर्वर से जुड़ता है।"),
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
    fn rtl_languages_detected() {
        for code in ["ar", "ur", "fa", "he", "dv", "ps", "sd", "ks", "ar-EG", "ur_PK"] {
            assert!(is_rtl(code), "{code} should be RTL");
        }
        for code in ["en", "hi", "ta", "zh", "zh-Hant", "pt-BR", "es-MX", "sw", ""] {
            assert!(!is_rtl(code), "{code} should be LTR");
        }
    }

    #[test]
    fn shipped_languages_translate_chrome() {
        const LANGS: &[&str] = &[
            "hi", "ta", "te", "bn", "mr", "gu", "kn", "ml", "pa", "ur", "or", "as", "es", "fr",
            "pt", "de", "ar", "zh", "ja", "ko", "ru", "id", "tr", "vi", "th", "it", "nl", "sw",
        ];
        // Every English key must have a real translation in every
        // shipped language — static UI must never fall back to English.
        const KEYS: &[&str] = &[
            "backup_auto", "backup_auto_none", "backup_copy_to", "backup_copy_btn", "autostart_off", "autostart_on", "autostart_done_on", "autostart_done_off",
            "device_forgotten",
            "lan_offline",
            "forget_device", "last_seen", "new_pair_code",
            "kot_waiting", "lan_offline_waiting", "lan_connecting", "lan_sending", "lan_sign_in_to_send", "lan_pair_again", "lan_update_app",
            "allow_firewall", "firewall_blocked", "firewall_waiting",
            "lan_join_help", "lan_pair_status", "lan_join_help_no_addr", "lan_no_address", "lan_not_listening", "lan_tills_off", "paired_with", "not_paired", "choose_device_use", "paired_note", "last_update_at", "pair_intro", "queue_waiting", "queue_tries", "save_backup_copy", "backup_copy_saved", "backup_copy_overdue", "backup_copy_failed", "clock_off", "printers_heading", "printers_help", "printers_pool_heading", "printers_pool_empty", "printers_routes_heading", "printers_routes_help", "station_manage_heading", "printer_label_ph", "printer_target_ph", "printer_target_default", "add_printer", "remove_printer", "route_pick_ph", "route_bill", "route_default_kitchen", "route_kot_station", "route_dont_print", "route_up", "route_down", "route_remove", "route_clear", "save_printers", "print_test_page", "printers_saved", "test_page_sent", "printers_found", "find_printers", "printers_looking", "printers_tap_find", "printers_none_found", "kot_printer_ph", "bill_printer_ph", "kot_print_on", "kot_print_off", "queue_refused", "clear_not_accepted", "device_name_ph", "lan_last_update", "lan_not_accepted", "connected_now", "backup_failed", "backup_overdue", "disk_low",
            "connection_to_shop",
            "other_devices_help_heading", "other_devices_help",
            "account_kind", "actual", "actual_total_cap", "add_a_table",
            "add_branch", "add_conversion", "add_dishes_items", "add_expense",
            "add_item", "add_item_plain", "add_note", "add_opening_balance",
            "add_staff", "add_station", "add_station_heading", "add_table",
            "add_to_bill", "add_unit", "add_upper", "add_use_name", "added",
            "adding_item", "advance", "against_bill_ph", "aliases_ph", "all_items",
            "frequent", "frequent_config_help", "frequent_days_ph", "frequent_limit_ph", "frequent_apply", "frequent_saved",
            "frequent_empty", "zero_sales", "zero_sales_help", "zero_sales_on", "zero_sales_off",
            "zero_sales_saved_on", "zero_sales_saved_off",
            "floor_areas_help", "area_new_ph", "add_area", "area_added",
            "allow_extra_tills", "amend", "amend_help", "amount_blank_full_ph",
            "amount_owed_ph", "amount_ph", "amount_received_ph", "amount_we_pay_ph",
            "app_tagline", "applied", "applied_total_cap", "apply_country_pack",
            "attach_bill_photo", "attach_pdf", "attach_photo",
            "attachments_last_expense", "auto", "back", "backup", "backup_settings",
            "bank", "bank_balance", "bank_book", "bank_in", "bank_net_today",
            "bank_out", "bank_reconciliation", "bank_to_till", "book_add_dish",
            "book_kind_aging", "book_kind_balance", "book_kind_month",
            "book_kind_today", "book_opening_credit", "books", "books_help",
            "branch_export_help", "branch_name_sell_ph", "branches", "branches_org",
            "brand_shown_help", "bring_my_book", "business_name_ph",
            "business_tax_id_ph", "buy", "buy_help", "buy_money_out",
            "caption_bank", "caption_bills", "caption_outstanding",
            "caption_payments_today", "caption_today", "card", "card_amount_ph",
            "cash", "cash_amount_ph", "cash_in_hand", "cash_memo", "close_dish_list", "dish_picker_help", "dish_search_ph", "tile_free", "tile_occupied", "tile_food_ready", "tile_bill_printed", "tile_paid", "tile_bill_btn", "tile_occupy_btn", "tile_free_btn", "tile_ready_btn", "tile_undo_bill", "summary_busy", "summary_to_collect", "summary_on_floor", "unit_picker_on", "unit_picker_off", "units_picker_help", "sizes_as_items_ph", "dish_group_ph",             "discount", "remove_discount", "apply_discount", "save_opening_cash", "close_shift_print", "area_prices_ph", "discount_ph", "discount_reason_ph", "opening_float_ph", "shift_counted_ph", "opening_float", "shift_handover", "shift_since", "expected_in_drawer", "right_discount", "floor_in_kitchen", "floor_eating", "pick_category", "close_category_list", "new_category_row", "your_category", "suggested_category", "category_search_ph", "line_sent_state", "line_nc_state", "cancel_item_short", "nc_short", "charge_again_short", "move_short", "cancel_item_title", "nc_item_title", "charge_again_title", "move_item_title", "already_cooked", "not_cooked_yet", "return_to_customer", "short_by", "tip_not_sale", "right_settle", "right_cancel_sent", "right_complimentary", "right_move_table", "right_reprint", "right_allowed", "right_with_approval", "right_denied", "editing_staff", "finding_server", "no_server_found", "print_bill", "move_merge_table", "move_help", "move_target_ph", "approval_pin_ph", "move_here", "merge_into", "line_reason_ph", "line_qty_ph", "line_target_ph", "confirm", "menu_search_ph", "customer_paid_ph", "tip_ph", "table_area_ph", "table_range_ph", "add_these_tables", "remove_these_tables", "table_remove_ph", "find_server", "bill_format", "bill_format_help", "bill_header_ph", "fssai_ph", "bill_footer_ph", "save_bill_format", "print_brand_kit", "print_brand_kit_help", "choose_stamp", "clear_stamp", "stamp_set", "stamp_none", "choose_payment_qr", "clear_payment_qr", "payment_qr_set", "payment_qr_none", "aggregator_import", "aggregator_import_help", "import_aggregator_file", "schemes", "schemes_help", "scheme_bogo_ph", "add_scheme", "schedule_ph", "upi_vpa_ph", "upi_payee_ph", "upi_qr", "show_upi_qr", "close_upi_qr", "upi_qr_help", "kot_recall", "recently_done", "recall_kot_ph",  "team_pick_ph", "select_staff", "rights_help", "edit", "on_your_menu", "own_item_name",
            "duress_heading", "duress_help", "duress_pin_ph", "duress_confirm_ph", "save_duress_pin", "clear_duress_pin", "duress_on", "duress_off", "duress_saved", "duress_cleared",
            "categories_help", "category_new_ph", "change_my_pin_heading",
            "chip_clear_credit", "chip_missing_customer", "chip_no_customer",
            "choose_icon", "choose_language", "choose_logo", "choose_server",
            "choose_server_hint", "clear", "clear_icon", "clear_photo", "client",
            "code_bag_ph", "collect_at_counter", "collect_payment",
            "connect_to_server", "continue", "conv_factor_ph", "copy_day_book_csv",
            "copy_day_close", "copy_items_csv", "copy_outstanding_csv",
            "copy_parties_csv", "copy_party_ledger_csv", "copy_receipt", "share_whatsapp", "lan_pages_heading", "lan_pages_help", "diag_send_heading", "kot_dest_heading", "optional_login_heading", "rights_heading", "ops_panel", "split_evenly", "waitlist", "fire_course", "hold_item", "waiter_call", "waitlist_add", "floor_view_grid", "floor_view_layout", "floor_layout_x", "floor_layout_y", "floor_layout_w", "floor_layout_h", "floor_layout_save", "floor_layout_open", "floor_layout_edit", "floor_layout_pick", "floor_layout_saved", "floor_layout_nudge_left", "floor_layout_nudge_right", "floor_layout_nudge_up", "floor_layout_nudge_down", "sibling_pickers_help", "bulk_price_apply", "card_terminal_optional", "crm_heading", "crm_help", "crm_queue_sms", "crm_queue_wa", "crm_seg_bday_week", "crm_seg_bday_month", "crm_seg_phone", "crm_seg_loyalty", "crm_seg_inactive", "crm_seg_top", "birthday_ph", "crm_message_ph", "till_apply_redeem", "till_clear_redeem", "till_wallet_topup", "till_issue_voucher", "till_points_ph", "till_wallet_ph", "till_voucher_code_ph", "till_voucher_amt_ph", "till_wallet_topup_ph", "till_voucher_issue_amt_ph", "till_voucher_issue_code_ph",
            "copy_report", "counted_cash_ph", "counter", "counter_locked",
            "country_code_ph", "country_pack_help", "country_tax", "create_backup",
            "create_new_server", "credit", "credit_customers", "credit_help",
            "credit_khata_amount_ph", "credit_limit_heading", "credit_limit_ph",
            "credit_summary_fmt", "currency", "currency_use_help",
            "customer_name_ph", "day_close", "day_close_help", "day_close_summary",
            "delete_party", "delivery", "device_setup_help", "device_use_q",
            "dine_in", "dine_in_pick_table", "done_open_books", "drawer_count",
            "choose_table", "change_table", "pick_table_banner", "items_word", "items_section", "back_to_order", "new_items_head", "price_ph", "start_with_template", "template_heading", "unpriced_note", "show_unpriced", "show_all_dishes", "recipe_ph", "reorder_ph", "set_reorder", "low_stock_alert", "pick_dishes_help", "pick_done", "dishes_n", "order_empty_pick", "add_dishes", "order_so_far", "no_dish_match", "resv_heading", "resv_name_ph", "resv_phone_ph", "resv_guests_ph", "resv_when_ph", "resv_table_ph", "resv_add", "resv_seat", "resv_none", "guests_word", "delivery_by_ph", "kitchen_all_day", "split_bill", "split_go", "cancel", "split_help", "split_send_first", "split_pick_some", "split_done", "split_marked", "template_info", "template_import", "template_confirm", "template_yes", "template_done", "template_imported", "role_waiter", "role_kitchen", "role_title_rest_ph", "preset_help_rest",
            "kind_raw", "kind_resale", "kind_dish", "buy_unit_ph", "add_another_way", "parts_status", "save_expense", "rest_ph", "low_badge", "stock_empty", "recent_changes",
            "editing_item", "empty_catalog", "enter_owner_passcode",
            "enter_staff_passcode", "expense_heading", "expenses_word",
            "export_day", "extra_note_bill_ph", "extra_note_expense_ph",
            "extra_till", "extra_tills_heading", "favorites", "favorites_help",
            "feature_access_help", "floor_help", "footer_local", "forget_shop",
            "fresh_book_khaynjo", "fresh_selected", "from_code_ph",
            "goods_received", "gross_word", "hide_pin", "how_start", "icon_custom",
            "icon_default", "icon_new_selected", "import_day",
            "import_items_csv_plain", "import_parties_csv", "in_stock", "inventory",
            "inventory_stock", "item_name", "item_name_lang_ph", "item_name_ph",
            "item_price_ph", "items", "items_help", "kitchen", "kitchen_empty",
            "kitchen_help", "kitchen_printing", "kot_cancelled", "kot_done_btn",
            "kot_done_cap", "kot_mark_ready", "kot_new_waiting", "kot_preparing",
            "kot_print_help_khaynjo", "kot_ready_serve", "kot_start_btn",
            "kot_unassigned_held", "pos_tab_items", "pos_tab_payment",
            "lang_follow_platform", "lang_pick_user", "lang_platform_now",
            "language", "language_currency", "language_setup_hint",
            "last_client_mode", "last_server_mode", "later_country_pack",
            "local_badge", "lock_app", "locked_word", "login_id_opt_ph",
            "login_id_ph", "logo_default", "logo_yours", "manager_pin_ph",
            "mark_cleared", "marked_sold_out", "mix_type_hint_khaynjo",
            "money_in_sales", "more", "na", "nav_lock", "nav_reports",
            "nav_settings", "net_word", "new_branch_name_ph", "new_customer_ph",
            "new_item", "new_party_clear", "new_party_kind_supplier_ph",
            "new_password_ph", "new_pin_ph", "new_server_name_ph", "next",
            "next_country", "next_order", "no_favorites", "no_sale_yet",
            "no_staff_yet", "not_tracked", "note_bank_ph", "note_buy_expense_ph",
            "note_expense_ph", "note_gstin_ph", "note_ph", "notes_attach_party",
            "notes_attach_sale", "off", "offline_badge", "ok", "on_account_ap",
            "onboard_book_tmpl", "onboard_fresh_tmpl", "onboard_template_tmpl",
            "open_bill", "open_server", "optional_login_help", "options_khaynjo_ph",
            "or_unlock_login", "order", "order_help", "org_optional_ph", "owed",
            "owes_word", "owner", "paid_by", "paid_now", "pair_now",
            "pairing_code_ph", "part_second_way_ph", "party_kind_ph",
            "party_name_selected_ph", "password_ph", "pay_ap_card", "pay_ap_cash",
            "pay_ap_heading", "pay_ap_upi", "pay_credit", "pay_selected_heading",
            "payment_pending", "permission_preset", "phone_ph", "photo", "pick",
            "pick_from_items", "pin_count_full", "pin_count_need",
            "pin_count_ready", "pin_count_zero", "pin_hint", "pin_hint_books",
            "pin_hint_choose", "pin_hint_new", "pin_hint_pair_first",
            "pin_hint_paired", "pin_hint_shop_code", "place_name_ph",
            "preset_help_khaynjo", "prev", "prev_country", "print_open",
            "product_locked_khaynjo", "product_only_khaynjo", "purchases_cogs",
            "qty", "qty_cap", "qty_example_ph", "quantity_ph", "ready_to_sell",
            "reason_amend_ph", "reason_note_ph", "reason_required_ph",
            "recent_label", "record_day_close", "record_purchase", "refresh",
            "refresh_from_shop", "refund", "remove", "remove_selected",
            "removed_help", "removed_items", "reports",
            "reports_aging", "reports_balance_sheet", "reports_expenses",
            "reports_facet_help", "reports_facet_title", "reports_help",
            "reports_month", "reports_today", "request_bill", "restaurant_name_ph",
            "restock", "restore_backup", "restore_help", "restore_item", "role_both",
            "role_cashier", "role_customer", "role_help_both", "role_help_customer",
            "role_help_supplier", "role_manager", "role_title_ph", "rollup",
            "round_word", "rounded", "rounded_total", "sales", "sales_refunds",
            "save", "save_brand", "save_business_id", "save_business_name",
            "save_features", "save_names", "second_way", "select",
            "selected_language", "selected_word", "sell", "sell_money_in",
            "send_kot", "server", "server_ip_ph", "service_pct_ph",
            "set_credit_limit", "set_login_selected", "set_owner_pin",
            "settle_card", "settle_cash", "setup_app_khaynjo", "setup_complete",
            "shop", "shop_logo_optional", "short_code_ph", "show", "show_pin",
            "sold_out", "split", "spreadsheet_csv", "staff", "staff_name_ph",
            "start_fresh", "start_order", "start_template", "start_tracking",
            "starting_password_ph", "starting_pin_ph", "station_name_ph",
            "station_new_ph", "status_client_unlock", "status_server_set_pin",
            "status_server_tills", "status_server_unlock", "stock_minus",
            "stock_plus", "stock_type_help", "stop_extra_tills", "supplier_name_ph",
            "supplier_ph", "switch_device", "table_free", "table_name_ph",
            "table_running", "tables", "tables_kitchen", "take_payment", "takeaway",
            "tax_added_till", "tax_inside_price", "tax_invoice", "tax_rate_ph",
            "team", "team_owner_help", "team_pins", "they_owe_us", "this_device",
            "this_shop", "till_to_bank", "tmpl_blurb", "tmpl_current", "tmpl_how",
            "tmpl_selected", "to_code_ph", "today_mid", "today_prefix", "total",
            "uncleared_no_ph", "undo", "unit_at", "unit_at_cap", "unit_cost_ph",
            "unit_name_bag_ph", "unit_plate_ph", "units_heading", "units_help",
            "units_picker_help", "unit_picker_on", "unit_picker_off",
            "unlock", "unlock_after_pairing", "unlock_to_continue",
            "unlock_with_login", "unmark", "update_my_password", "update_my_pin",
            "upi", "upi_amount_ph", "use", "dont_use", "void", "we_owe_them", "who_unlocking",
            "write_off_ap", "write_off_heading", "write_off_nil", "your_brand",
            "your_categories", "your_favorites", "your_items",
        ];
        // Every gap at once, so one run says everything still to translate.
        let mut gaps = Vec::new();
        for lang in LANGS {
            for key in KEYS {
                let value = t(lang, key);
                if value == *key {
                    gaps.push(format!("{lang} left {key} untranslated"));
                } else if value.is_empty() && !en(key).is_empty() {
                    gaps.push(format!("{lang} {key} is empty"));
                } else if chrome_extra(lang, key).is_none() && value == en(key) && !en(key).is_empty() {
                    // A chrome row (or the per-language table) must cover the
                    // key — falling back to the English text is a failure.
                    gaps.push(format!("{lang} falls back to English for {key}"));
                }
            }
        }
        assert!(gaps.is_empty(), "{} gaps:\n{}", gaps.len(), gaps.join("\n"));
        assert_eq!(t("en", "sold_out"), "Unavailable");
        assert_eq!(t("gu", "cash"), "રોકડ");
        assert_eq!(t("gu", "owed"), "બાકી");
        assert_eq!(t("hi", "sold_out"), "बिक गया");
    }

    /// Buttons must never show snake_case keys to end users.
    #[test]
    fn english_chrome_never_returns_raw_keys() {
        const KEYS: &[&str] = &[
            // Nav labels — a missing English entry shows the raw key ("floor").
            "floor", "kitchen", "favorites",
            "record_purchase",
            "add_expense",
            "refresh",
            "undo",
            "start_tracking",
            "stock_minus",
            "stock_plus",
            "settle_cash",
            "settle_card",
            "buy_money_out",
            "sell_money_in",
            "inventory_stock",
            "backup_settings",
            "lock_app",
            "language_currency",
            "nav_reports",
            "nav_settings",
            "nav_lock",
            "credit",
            "sales",
        ];
        for key in KEYS {
            let value = t("en", key);
            assert_ne!(value, *key, "English left {key} untranslated (button would show the key)");
            assert!(!value.contains('_'), "English {key} still looks like a code id: {value}");
        }
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
