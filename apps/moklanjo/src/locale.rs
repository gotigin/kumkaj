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
        "app_tagline" => "Courier-agency books + booking desk. Free offline.",
        "unlock" => "Unlock",
        "pin_hint" => "Owner PIN starts as 0000",
        "sell" => "Booking",
        "buy" => "Pay out",
        "books" => "Day book",
        "books_help" => "Today’s position. Booking = parcels in. Pay out = carrier bills & expenses. Parties = consignors & carriers.",
        "buy_help" => "Expense vouchers and carrier bills (money out). On-account stays on the party ledger.",
        "sell_money_in" => "Book a parcel",
        "buy_money_out" => "Expense / carrier bill",
        "record_purchase" => "Record purchase",
        "add_expense" => "Expense voucher",
        "bank_book" => "Bank book",
        "credit_help" => "Party id is unique (names may repeat). Roles: Consignor (AR), Carrier (AP), or both on that id. Ledgers stay separate; reports show net.",
        "party_kind_ph" => "Roles: Consignor, Carrier, both…",
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
        "items" => "Rates & services",
        "more" => "More",
        "your_items" => "Your rates",
        "items_help" => "Each row is a rate (e.g. Surface Zone A). Unit is usually kg. Tap to edit.",
        "add_item" => "Add rate",
        "new_item" => "New rate",
        "adding_item" => "Adding a new rate",
        "editing_item" => "Editing",
        "save" => "Save",
        "remove" => "Remove",
        "empty_catalog" => "No rates yet. Open Rates & services, add one (name + price, unit kg), then Booking.",
        "counter_sale" => "Counter booking",
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
        "counter" => "Counter",
        "order" => "Order",
        "sales" => "Bookings",
        "sales_refunds" => "Bookings & refunds",
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
        "inventory" => "Inventory",
        "inventory_stock" => "Inventory & stock",
        "branches" => "Branches",
        "branches_org" => "Branches & org",
        "credit_customers" => "Parties (debtors & creditors)",
        "customer_name_ph" => "Party name",
        "new_customer_ph" => "New party name",
        "item_name_ph" => "Rate name (e.g. Surface Zone A)",
        "item_price_ph" => "Per kg / first slab",
        "start_template" => "Product",
        "next_booking" => "Next booking",
        "destination_ph" => "Destination city / zone",
        "consignee_ph" => "Consignee (receiver)",
        "weight_kg_ph" => "Actual kg",
        "length_cm_ph" => "L cm",
        "breadth_cm_ph" => "B cm",
        "height_cm_ph" => "H cm",
        "chargeable_kg" => "Chargeable kg",
        "awb_label" => "AWB",
        "booking_help" => "Walk-in: destination → weigh → pick rate → Account/Cash booking → pay. AWB prints on the receipt. Next booking clears the desk.",
        "cart_empty_hint" => "Desk empty — set destination & weight, tap ADD on a rate, then pay.",

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
        "add_branch" => "Add branch",
        "add_conversion" => "Add conversion",
        "add_item_plain" => "Add item",
        "add_note" => "Add note",
        "add_opening_balance" => "Add opening balance",
        "add_rider" => "Add rider",
        "add_staff" => "Add staff",
        "add_unit" => "Add unit",
        "add_use_name" => "Add / use this name",
        "allow_extra_tills" => "Allow extra tills",
        "amend" => "Amend",
        "apply_country_pack" => "Apply country pack",
        "attach_bill_photo" => "Attach bill photo",
        "attach_pdf" => "Attach PDF",
        "attach_photo" => "Attach photo",
        "bank" => "Bank",
        "bank_in" => "Bank in",
        "bank_out" => "Bank out",
        "bank_to_till" => "Bank → till",
        "till_to_bank" => "Till → bank",
        "mark_cleared" => "Mark cleared",
        "unmark" => "Unmark",
        "role_both" => "Both",
        "role_carrier" => "Carrier",
        "role_cashier" => "Cashier",
        "role_consignor" => "Consignor",
        "role_manager" => "Manager",
        "role_supplier" => "Supplier",
        "branch_name_sell_ph" => "Branch name to sell from",
        "bring_my_book" => "Bring my book",
        "business_tax_id_ph" => "Business tax id",
        "card_amount_ph" => "Card amount",
        "cash_amount_ph" => "Cash amount",
        "upi_amount_ph" => "UPI amount",
        "carrier_cost_ph" => "Carrier cost",
        "carrier_party_ph" => "Carrier party",
        "choose_icon" => "Choose icon",
        "choose_logo" => "Choose logo",
        "clear_icon" => "Clear icon",
        "clear_photo" => "Clear photo",
        "cod_register" => "COD register",
        "collect_cod" => "Collect COD (AWB)",
        "consignments_dispatch" => "Consignments & dispatch",
        "consignor_party_ph" => "Consignor / party name",
        "copy_day_book_csv" => "Copy day book CSV",
        "copy_day_close" => "Copy day close",
        "copy_items_csv" => "Copy items CSV",
        "copy_outstanding_csv" => "Copy outstanding CSV",
        "copy_parties_csv" => "Copy parties CSV",
        "copy_party_ledger_csv" => "Copy party ledger CSV",
        "copy_receipt" => "Copy receipt",
        "copy_report" => "Copy report",
        "counted_cash_ph" => "Counted cash in drawer",
        "counter_agency_ph" => "Counter / agency name (optional)",
        "create_backup" => "Create backup",
        "day_close_summary" => "Day close summary",
        "delete_party" => "Delete selected party",
        "dispatch_to_rider" => "Dispatch booked → rider",
        "docket_charge_ph" => "Docket charge",
        "declared_value_ph" => "Declared value",
        "contents_ph" => "Contents",
        "eway_ph" => "E-way bill no.",
        "insurance_label" => "Insurance (yes/no)",
        "done_open_books" => "Done — open books",
        "export_day" => "Export day",
        "extra_note_expense_ph" => "Extra note on last expense",
        "extra_note_awb_ph" => "Extra note on this AWB / sale",
        "failed_delivery" => "Failed delivery",
        "find_scan_awb" => "Find / Scan AWB",
        "forget_shop" => "Forget this shop",
        "fuel_ph" => "Fuel",
        "handover_carrier" => "Hand booked → carrier",
        "import_day" => "Import day",
        "import_items_csv" => "Import items / rates CSV…",
        "import_parties_csv" => "Import parties CSV…",
        "import_pincodes_csv" => "Import pincodes CSV…",
        "import_carrier_bill_csv" => "Import carrier bill CSV…",
        "copy_carrier_bill_report" => "Copy last carrier bill report",
        "use_carrier_cost" => "Use carrier cost",
        "rate_carrier_ph" => "Carrier (blank = our sell rate)",
        "rate_party_ph" => "Party (blank = all customers)",
        "carrier_pick_ph" => "Carrier for cost (from compare)",
        "item_name" => "Item name",
        "load_bulk_row" => "Load bulk row → desk",
        "login_id_ph" => "Login id",
        "mark_delivered" => "Mark delivered",
        "manifest_cod_total" => "COD total",
        "manifest_received_by" => "Received by",
        "manifest_sign" => "Sign",
        "ndr_address_wrong" => "Address wrong",
        "ndr_customer_unavailable" => "Unavailable",
        "ndr_door_locked" => "Door locked",
        "ndr_other" => "Other",
        "ndr_refused" => "Refused",
        "out_for_delivery" => "Out for delivery",
        "print_drs" => "Print DRS",
        "print_run_sheet" => "Print run sheet",
        "reattempt" => "Reattempt",
        "rider_cod_handed_ph" => "COD handed in",
        "settle_rider_cod" => "Settle rider COD",
        "mode_desk" => "Desk",
        "mode_rider" => "Rider",
        "waiting_office" => "waiting to reach office",
        "capture_signature" => "Capture signature",
        "incoming_manifest" => "Incoming manifest",
        "pickup_request" => "Pickup request",
        "pickup_run" => "Pickup run",
        "auto_assign_area" => "Auto-assign area",
        "reorder_stops" => "Reorder stops",
        "settle_rider_pay" => "Settle rider pay",
        "vehicle_inspect" => "Vehicle inspection",
        "print_docket" => "Print docket",
        "franchise_statement" => "Franchise statement",
        "tally_export" => "Tally voucher export",
        "open_whatsapp" => "Open WhatsApp",
        "copy_sms" => "Copy SMS text",
        "pick_barcode_image" => "Pick image to scan",
        "share_receipt_whatsapp" => "Share receipt on WhatsApp",
        "delivery_feedback" => "Delivery feedback",
        "lan_self_book" => "LAN self-booking",
        "new_party_clear" => "New party (clear form)",
        "new_party_kind_ph" => "New party kind (Supplier / Carrier)",
        "next_country" => "Next country",
        "prev_country" => "Previous country",
        "note_gstin_ph" => "Note / GSTIN / agreement reminder",
        "note_pod_ph" => "Note / POD (optional)",
        "on_account_ap" => "On account (AP)",
        "org_optional_ph" => "Organization (optional)",
        "paid_now" => "Paid now",
        "pairing_code_ph" => "Pairing code",
        "pair_now" => "Pair now",
        "party_name_selected_ph" => "Party name (or use selected)",
        "password_ph" => "Password",
        "pay_ap_card" => "Pay AP card",
        "pay_ap_cash" => "Pay AP cash",
        "pay_ap_upi" => "Pay AP UPI",
        "period_bill" => "Period bill",
        "period_bill_party" => "Period bill (party)",
        "pick_pod_photo" => "Pick POD photo",
        "print_awb_label" => "Print AWB label",
        "print_open" => "Print / open",
        "reason_amend_ph" => "Reason for amend / void / refund",
        "reason_note_ph" => "Reason / note (wastage, count, …)",
        "record_day_close" => "Record day close",
        "refresh_from_shop" => "Refresh from shop",
        "refund" => "Refund",
        "remit_cod" => "Remit COD (party)",
        "remove_selected" => "Remove selected",
        "restore_backup" => "Restore backup",
        "rider_name_ph" => "Rider name",
        "rider_phone_ph" => "Rider phone",
        "riders" => "Riders",
        "riders_own_fleet" => "Riders (own fleet)",
        "rollup" => "Roll-up",
        "save_booking_measures" => "Save booking measures",
        "save_brand" => "Save brand",
        "save_business_id" => "Save business id & service %",
        "save_business_name" => "Save business name",
        "save_features" => "Save features for selected staff",
        "save_names" => "Save names",
        "server_ip_ph" => "Server IP (e.g. 192.168.1.10)",
        "set_credit_limit" => "Set credit limit",
        "set_login_selected" => "Set login on selected",
        "shop" => "Shop",
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
        "right_allowed" => "Allowed",
        "right_with_approval" => "With approval",
        "right_denied" => "Not allowed",
        "editing_staff" => "Rights for",
        "team_pick_ph" => "Staff name to edit",
        "select_staff" => "Select this staff",
        "rights_help" => "Rights — tap to change: Allowed → With approval (a manager's PIN at that moment) → Not allowed.",
        "right_void_refund" => "Void or refund a booking",
        "right_amend_sale" => "Amend a recorded booking",
        "find_server" => "Find the shop's server",
        "finding_server" => "Looking for the shop's server on this Wi‑Fi…",
        "no_server_found" => "No server found. Check the shop device is on the same Wi‑Fi with extra tills allowed, or type its IP.",
        "device_forgotten" => "The shop forgot this device — pair it again with the shop's current code. Orders waiting on it are kept.",
        "forget_device" => "Forget",
        "last_seen" => "seen",
        "new_pair_code" => "New pairing code",
        "lan_offline" => "Offline",
        "lan_offline_waiting" => "Offline · {n} waiting",
        "lan_sending" => "{n} sending",
        "lan_sign_in_to_send" => "Sign in again to send {n}",
        "lan_pair_again" => "Pair this device again",
        "allow_firewall" => "Allow other devices to connect",
        "firewall_blocked" => "This computer's firewall is blocking other devices — tap Allow other devices to connect.",
        "firewall_waiting" => "Waiting for the admin prompt…",
        "lan_join_help" => "Other devices can join this one — pairing code {code}. On the other device choose Client Mode, then Find the shop's server, or type this address: {addr}",
        "lan_join_help_no_addr" => "Other devices can join this one — pairing code {code}.",
        "lan_no_address" => "This device has no Wi‑Fi address yet — connect it to the shop's Wi‑Fi.",
        "lan_not_listening" => "Not accepting devices right now — another program may be using port {port}. Restart the app.",
        "lan_tills_off" => "Extra tills are off. This office's books stay on this device. Allow extra tills if another device should book parcels or take payments too.",
        "paired_with" => "Paired with the shop at {url}",
        "not_paired" => "Not paired — connect to the shop's server.",
        "choose_device_use" => "Choose how this device is used.",
        "paired_note" => "Paired. Unlock to sell, or Refresh to pull the latest from the shop.",
        "last_update_at" => "Last update from the shop: {time}.",
        "pair_intro" => "Tap Find the shop's server, or type its address and the 6-digit pairing code shown on it (Settings → Extra tills).",
        "queue_waiting" => "Waiting to reach the shop ({n}):",
        "queue_tries" => "Tried {n} times · last answer: {e}",
        "save_backup_copy" => "Save a backup copy…",
        "backup_copy_saved" => "Backup copy saved — keep it off this device (a USB stick or another device)",
        "backup_copy_overdue" => "No backup copy has left this device for {n} days — Settings → Backup",
        "backup_copy_failed" => "The backup copy could not be saved there — pick another place",
        "clock_off" => "This device's clock is {min} min off the shop's server — set the right time on this device",
        "printers_heading" => "Printers on this device",
        "printers_help" => "Add real printers (name + address or system name). Then under Where things print, pick Receipt or AWB label and add printers in try-order — first tried first; if it fails, the next. Leave a route empty to not print. Each device keeps its own printers.",
        "printers_pool_heading" => "Printers",
        "printers_routes_heading" => "Where things print",
        "printer_label_ph" => "Name (e.g. Desk)",
        "printer_target_ph" => "Address or system name",
        "add_printer" => "Add printer",
        "remove_printer" => "Remove last",
        "route_pick_ph" => "receipt or label",
        "route_add_printer_ph" => "Add printer to this route",
        "route_receipt" => "Receipt",
        "route_label" => "AWB label",
        "route_dont_print" => "Don't print",
        "route_add" => "Add to route",
        "route_up" => "Up",
        "route_down" => "Down",
        "route_remove" => "Remove last",
        "route_clear" => "Don't print",
        "save_printers" => "Save printers",
        "print_test_page" => "Print a test page",
        "printers_saved" => "Printers saved for this device",
        "test_page_sent" => "Test page sent — check the printer",
        "printers_found" => "Found on this device: {list}",
        "find_printers" => "Find printers",
        "printers_looking" => "Looking for printers on this device and Wi‑Fi…",
        "printers_tap_find" => "Tap Find printers for queues that accept jobs and network receipt printers on this Wi‑Fi (not removed or offline ones).",
        "printers_none_found" => "No printers found — type a system printer name, or a network address (e.g. 192.168.1.50).",
        "receipt_printer_ph" => "Printer for receipts",
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
        "today_books" => "Today books",
        "unlock_after_pairing" => "Unlock (after pairing)",
        "unlock_with_login" => "Unlock with login",
        "update_my_password" => "Update my password",
        "update_my_pin" => "Update my PIN",
        "duress_heading" => "Emergency unlock PIN (Owner only)",
        "duress_help" => "Optional. If this PIN is typed at unlock instead of yours, the desk opens with no booking history — rates, parties, zones and staff stay. Use only if you are forced to unlock. Leave blank and save to turn off. Never the same as anyone's PIN.",
        "duress_pin_ph" => "Emergency PIN (4–12 digits)",
        "duress_confirm_ph" => "Type again to confirm",
        "save_duress_pin" => "Save emergency PIN",
        "clear_duress_pin" => "Turn off emergency PIN",
        "duress_on" => "Emergency unlock PIN is on",
        "duress_off" => "Emergency unlock PIN is off",
        "duress_saved" => "Emergency unlock PIN saved",
        "duress_cleared" => "Emergency unlock PIN turned off",
        "vehicle_ph" => "Vehicle",
        "void" => "Void",
        "write_off_ap" => "Write off AP",
        "write_off_nil" => "Write off / nil balance",
        "zone_ph" => "Zone (name from Rates)",
        "caption_today" => "TODAY",
        "caption_bookings" => "BOOKINGS",
        "caption_payments_today" => "PAYMENTS TODAY",
        "caption_outstanding" => "OUTSTANDING",
        "caption_bank" => "BANK",
        "paid_by" => "Paid by",
        "second_way" => "Second way",
        "weight_in" => "Weight in",
        "size_in" => "Size in",
        "goods_received" => "Goods received",
        "expense_heading" => "Expense",
        "notes_attach_sale" => "Notes & attachments (selected sale)",
        "notes_attach_party" => "Notes & attachments (selected party)",
        "attachments_last_expense" => "Attachments (last expense)",
        "credit_limit_heading" => "Credit limit (selected / named party)",
        "pay_selected_heading" => "Pay selected (settle debt / take advance)",
        "write_off_heading" => "Write off (clear without payment)",
        "pay_ap_heading" => "Pay what we owe them (AP)",
        "country_tax" => "Country & tax",
        "extra_tills_heading" => "Extra tills",
        "other_devices_help_heading" => "Set up other devices",
        "other_devices_help" => "On this device (Server): Extra tills → Allow extra tills. Note the pairing code and Wi‑Fi address. Tap Allow other devices to connect if it appears. On a computer, tap Start with this computer. On each other device: install the app, choose Client Mode, Find the shop's server (or type the address), enter the pairing code, Pair now. Orders wait on the device if Wi‑Fi drops and go when the Server answers again.",
        "your_brand" => "Your brand (in-app)",
        "units_heading" => "Units",
        "booking_measures" => "Booking measures",
        "drawer_count" => "Drawer count (optional)",
        "spreadsheet_csv" => "Spreadsheet (CSV)",
        "bank_reconciliation" => "Bank reconciliation (tick off against your bank statement)",
        "book_add_product" => "Book — add product / rate",
        "book_opening_credit" => "Book — opening credit",
        "account_kind" => "Account kind (tap one)",
        "active_ledger" => "Active ledger",
        "actual_total_cap" => "ACTUAL TOTAL",
        "amend_help" => "Select a sale to amend, void, or refund. Amendments keep full history.",
        "applied_total_cap" => "APPLIED TOTAL",
        "bank_balance" => "Bank balance",
        "bank_net_today" => "Bank net today",
        "branch_export_help" => "Export writes branch_snapshot_CODE_DATE.json into the backups folder and copies it to the clipboard. To import another outlet's day, save its file as import_branch_snapshot.json in the same folder, then tap Import.",
        "brand_shown_help" => "Shown on screens inside the app. The installed app name and icon on this device stay as the platform listing.",
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
        "fresh_book_moklanjo" => "Fresh = empty catalogue (add rates later in Items). Book = type opening rates and who owes you below.",
        "fresh_selected" => "Fresh selected — catalogue stays empty. Tap Done when ready.",
        "gross_word" => "Gross",
        "how_start" => "How do you want to start? (tap one — it stays highlighted)",
        "later_country_pack" => "Later: Settings → apply your country pack for tax and pay labels.",
        "local_badge" => "LOCAL",
        "measures_help_moklanjo" => "How parcels are measured at the booking desk. Type dimensions and weight in the units you use — standard (cm, m, in, ft · kg, g, lb, oz) or your own unit linked to cm / kg above (directly or through other units). Books always record kg. Leave a field empty to keep it.",
        "mix_type_hint_moklanjo" => "Type how much in each (cash / UPI / card / credit). Must add up to the bill. Credit needs a customer above.",
        "money_in_sales" => "Money in (sales)",
        "more_help_moklanjo" => "Catalogue, sales, credit, team, backup — plus stock and extra tills when needed.",
        "net_word" => "Net",
        "offline_badge" => "MAKEPAD 2 · OFFLINE",
        "optional_login_help" => "Optional login (id + password). Owner sets once; only the user can change password later.",
        "or_unlock_login" => "Or unlock with login id + password",
        "permission_preset" => "Permission preset (what they can access)",
        "preset_help_moklanjo" => "Cashier: sell. Manager: sell + items + void/amend.",
        "product_locked_moklanjo" => "Product: Moklanjo — books + booking desk (locked for this app).",
        "product_only_moklanjo" => "This app is Moklanjo only — books + booking desk. Other trade faces ship as separate products later.",
        "purchases_cogs" => "Purchases (COGS)",
        "qty_cap" => "QTY",
        "restore_help" => "Restore: pick a backup. The books open as they were then; the books as they are now are saved first, so this can be undone.",
        "rounded_total" => "Rounded Total",
        "setup_app_moklanjo" => "Set up Moklanjo",
        "shop_logo_optional" => "Shop logo (optional)",
        "stock_type_help" => "Type the item name (part of it is enough), a quantity, then adjust. Items that are not counted yet need Start tracking once.",
        "team_owner_help" => "Owner creates each person with a name, role title, permission preset, and starting PIN. Owner cannot change anyone's PIN later — only that person can update their own PIN.",
        "they_owe_us" => "They owe us",
        "unit_at_cap" => "Unit @",
        "units_help" => "Codes for item labels (kg, piece, bag). Every unit must link — directly or through other units — to a standard (kg · cm · ml · pc), e.g. 1 crate = 2 box and 1 box = 20 cm. Conversions: 1 from = factor to. Currencies are never units.",
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
        "pin_hint_choose" => "Choose 4–12 digits (keyboard or pad) · Enter/OK to save",
        "pin_hint_new" => "New 4–12 digit Owner PIN (keyboard or pad) · Enter/OK",
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
        "onboard_template_tmpl" => "Template: {} — {}. Name the place if you want, then Start fresh or Bring my book.",
        "tmpl_blurb" => "Courier-agency books + booking desk: consignors, carriers, day book, AWB.",
        "tmpl_how" => "Booking = walk-in parcels (weigh → rate → cash/account → AWB). Day book = today. Pay out = carrier bills & expenses. More → Rates, Parties, Bank book & P&L, Team.",
        "facet_bank_title" => "Reports",
        "facet_bank_help" => "Day books, party aging, bank posts, and period P&L.",
        "facet_ops_title" => "Consignments & dispatch",
        "facet_ops_help" => "Scan or type AWB → Find, then advance / print label / COD. Dispatch to rider or carrier.",
        "facet_cod_title" => "COD register",
        "facet_cod_help" => "Collect on delivery, then remit to the consignor.",
        "facet_riders_title" => "Riders (own fleet)",
        "facet_riders_help" => "Add riders, then dispatch booked AWBs from Ops.",
        "against_bill_ph" => "Against bill # above (optional)",
        "aliases_ph" => "Aliases — other names, comma separated",
        "amount_blank_full_ph" => "Amount (blank = full balance)",
        "amount_owed_ph" => "Amount owed",
        "amount_ph" => "Amount",
        "amount_received_ph" => "Amount received",
        "amount_we_pay_ph" => "Amount we pay",
        "awb_scan_ph" => "AWB / scan barcode",
        "bulk_csv_ph" => "Bulk CSV — destination,consignee,kg,cod_paise,note",
        "business_name_ph" => "Business name",
        "category_new_ph" => "Category (optional, created if new)",
        "cod_amount_ph" => "COD amount",
        "code_bag_ph" => "Code (bag)",
        "conv_factor_ph" => "Factor (25 = 1 bag → 25 kg)",
        "country_code_ph" => "Country code e.g. IN, GB, US, AE, BR",
        "credit_khata_amount_ph" => "Credit / khata amount",
        "credit_limit_ph" => "Credit limit (0 = none)",
        "dim_unit_code_ph" => "Dimension unit code (cm)",
        "divisor_ph" => "Volumetric divisor (5000)",
        "extra_per_kg_ph" => "Extra / kg",
        "first_amount_ph" => "First amount",
        "first_kg_ph" => "First kg (slab)",
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
        "notify_phone_ph" => "Notify phone (WhatsApp/SMS hook)",
        "options_moklanjo_ph" => "Options (optional) — Large=20, Gift wrap=10",
        "part_second_way_ph" => "Part by a second way (optional)",
        "phone_ph" => "Phone",
        "place_name_ph" => "Name of this place",
        "price_ph" => "Price",
        "qty_example_ph" => "Qty (e.g. 1.5 for kg)",
        "quantity_ph" => "Quantity",
        "reason_required_ph" => "Reason (required)",
        "role_title_ph" => "Role title (e.g. Barista)",
        "service_pct_ph" => "Service % optional 0/10/12",
        "short_code_ph" => "Short code (MG02)",
        "slab_kg_ph" => "Chargeable slab in kg (0.5)",
        "staff_name_ph" => "Staff name / id",
        "starting_password_ph" => "Starting password (optional)",
        "starting_pin_ph" => "Starting PIN (4–12 digits)",
        "tax_rate_ph" => "Tax rate name or % — Standard, Reduced, 0, 5, 12, 18, 20",
        "to_code_ph" => "To code (kg)",
        "uncleared_no_ph" => "Uncleared # above",
        "unit_cost_ph" => "Unit cost",
        "unit_kg_blank_ph" => "Unit (kg or blank)",
        "unit_name_bag_ph" => "Name (Bag)",
        "weight_unit_code_ph" => "Weight unit code (kg)",
        "zone_name_ph" => "Zone name",
        "pay_credit" => "Credit",
        "book_kind_today" => "Today books",
        "book_kind_month" => "This month books",
        "book_kind_aging" => "Outstanding / debit-credit",
        "book_kind_balance" => "Balance sheet",
        "ops_desk_help" => "Scan AWB with a USB barcode scanner (or type it) → Find. Then advance, print label, or collect COD. Labels print Code 128.",
        "role_help_consignor" => "Role: Consignor — AR (they owe us). Same firm can also be Carrier.",
        "role_help_carrier" => "Role: Carrier — AP (we owe them). Same firm can also be Consignor.",
        "role_help_both" => "Roles: Consignor + Carrier — one party, AR and AP ledgers, net in Reports.",
        "role_help_supplier" => "Role: Supplier — AP for purchases.",
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
            "right_allowed", "right_with_approval", "right_denied", "editing_staff", "team_pick_ph", "select_staff", "rights_help", "right_void_refund", "right_amend_sale",
            "find_server", "finding_server", "no_server_found",
            "device_forgotten",
            "forget_device", "last_seen", "new_pair_code",
            "lan_offline",
            "lan_offline_waiting", "lan_sending", "lan_sign_in_to_send", "lan_pair_again",
            "allow_firewall", "firewall_blocked", "firewall_waiting",
            "lan_join_help", "lan_join_help_no_addr", "lan_no_address", "lan_not_listening", "lan_tills_off", "paired_with", "not_paired", "choose_device_use", "paired_note", "last_update_at", "pair_intro", "queue_waiting", "queue_tries", "save_backup_copy", "backup_copy_saved", "backup_copy_overdue", "backup_copy_failed", "clock_off", "printers_heading", "printers_help", "printers_pool_heading", "printers_routes_heading", "printer_label_ph", "printer_target_ph", "add_printer", "remove_printer", "route_pick_ph", "route_add_printer_ph", "route_receipt", "route_label", "route_dont_print", "route_add", "route_up", "route_down", "route_remove", "route_clear", "save_printers", "print_test_page", "printers_saved", "test_page_sent", "printers_found", "find_printers", "printers_looking", "printers_tap_find", "printers_none_found", "receipt_printer_ph", "queue_refused", "clear_not_accepted", "device_name_ph", "lan_last_update", "lan_not_accepted", "connected_now", "backup_failed", "backup_overdue", "disk_low",
            "connection_to_shop",
            "other_devices_help_heading", "other_devices_help",
            "account_kind", "active_ledger", "actual", "actual_total_cap",
            "add_branch", "add_conversion", "add_expense", "add_item",
            "add_item_plain", "add_note", "add_opening_balance", "add_rider",
            "add_staff", "add_to_bill", "add_unit", "add_upper", "add_use_name",
            "added", "adding_item", "advance", "against_bill_ph", "aliases_ph",
            "all_items", "allow_extra_tills", "amend", "amend_help",
            "amount_blank_full_ph", "amount_owed_ph", "amount_ph",
            "amount_received_ph", "amount_we_pay_ph", "app_tagline", "applied",
            "applied_total_cap", "apply_country_pack", "attach_bill_photo",
            "attach_pdf", "attach_photo", "attachments_last_expense", "auto",
            "awb_label", "awb_scan_ph", "back", "backup", "backup_settings", "bank",
            "bank_balance", "bank_book", "bank_in", "bank_net_today", "bank_out",
            "bank_reconciliation", "bank_to_till", "book_add_product",
            "book_kind_aging", "book_kind_balance", "book_kind_month",
            "book_kind_today", "book_opening_credit", "booking_help",
            "booking_measures", "books", "books_help", "branch_export_help",
            "branch_name_sell_ph", "branches", "branches_org", "brand_shown_help",
            "breadth_cm_ph", "bring_my_book", "bulk_csv_ph", "business_name_ph",
            "business_tax_id_ph", "buy", "buy_help", "buy_money_out",
            "caption_bank", "caption_bookings", "caption_outstanding",
            "caption_payments_today", "caption_today", "card", "card_amount_ph",
            "carrier_cost_ph", "carrier_party_ph", "carrier_pick_ph", "cart_empty_hint", "cash",
            "copy_carrier_bill_report",
            "cash_amount_ph", "cash_in_hand", "category_new_ph",
            "change_my_pin_heading", "chargeable_kg", "chip_clear_credit",
            "duress_heading", "duress_help", "duress_pin_ph", "duress_confirm_ph", "save_duress_pin", "clear_duress_pin", "duress_on", "duress_off", "duress_saved", "duress_cleared",
            "chip_missing_customer", "chip_no_customer", "choose_icon",
            "choose_language", "choose_logo", "choose_server", "choose_server_hint",
            "clear", "clear_icon", "clear_photo", "client", "cod_amount_ph",
            "cod_register", "code_bag_ph", "collect_cod", "connect_to_server",
            "consignee_ph", "consignments_dispatch", "consignor_party_ph",
            "continue", "conv_factor_ph", "copy_day_book_csv", "copy_day_close",
            "copy_items_csv", "copy_outstanding_csv", "copy_parties_csv",
            "copy_party_ledger_csv", "copy_receipt", "copy_report",
            "counted_cash_ph", "counter", "counter_agency_ph", "counter_locked",
            "counter_sale", "country_code_ph", "country_pack_help", "country_tax",
            "create_backup", "create_new_server", "credit", "credit_customers",
            "credit_help", "credit_khata_amount_ph", "credit_limit_heading",
            "credit_limit_ph", "credit_summary_fmt", "currency",
            "currency_use_help", "customer_name_ph", "day_close", "day_close_help",
            "day_close_summary", "declared_value_ph", "delete_party", "destination_ph",
            "device_setup_help", "device_use_q", "dim_unit_code_ph",
            "dispatch_to_rider", "divisor_ph", "docket_charge_ph",
            "done_open_books", "drawer_count", "editing_item", "empty_catalog",
            "enter_owner_passcode", "enter_staff_passcode", "eway_ph", "expense_heading",
            "expenses_word", "export_day", "extra_note_awb_ph",
            "extra_note_expense_ph", "extra_per_kg_ph", "extra_till",
            "extra_tills_heading", "facet_bank_help", "facet_bank_title",
            "facet_cod_help", "facet_cod_title", "facet_ops_help",
            "facet_ops_title", "facet_riders_help", "facet_riders_title",
            "failed_delivery", "favorites", "favorites_help", "feature_access_help", "find_scan_awb",
            "first_amount_ph", "first_kg_ph", "footer_local", "forget_shop",
            "fresh_book_moklanjo", "fresh_selected", "from_code_ph", "fuel_ph",
            "goods_received", "gross_word", "handover_carrier", "height_cm_ph",
            "hide_pin", "how_start", "icon_custom", "icon_default",
            "icon_new_selected", "import_carrier_bill_csv", "import_day", "import_items_csv",
            "import_parties_csv", "import_pincodes_csv", "in_stock", "insurance_label", "inventory", "inventory_stock",
            "item_name", "item_name_lang_ph", "item_name_ph", "item_price_ph",
            "items", "items_help", "lang_follow_platform", "lang_pick_user",
            "lang_platform_now", "language", "language_currency",
            "language_setup_hint", "last_client_mode", "last_server_mode",
            "later_country_pack", "length_cm_ph", "load_bulk_row", "local_badge",
            "lock_app", "locked_word", "login_id_opt_ph", "login_id_ph",
            "logo_default", "logo_yours", "manager_pin_ph", "manifest_cod_total",
            "manifest_received_by", "manifest_sign", "mark_cleared",
            "mark_delivered", "marked_sold_out", "measures_help_moklanjo",
            "mix_type_hint_moklanjo", "money_in_sales", "more", "more_help_moklanjo",
            "na", "nav_lock", "nav_reports", "nav_settings", "ndr_address_wrong",
            "ndr_customer_unavailable", "ndr_door_locked", "ndr_other", "ndr_refused",
            "net_word",
            "new_branch_name_ph", "new_customer_ph", "new_item", "new_party_clear",
            "new_party_kind_ph", "new_password_ph", "new_pin_ph",
            "new_server_name_ph", "next", "next_booking", "next_country",
            "no_favorites", "no_sale_yet", "no_staff_yet", "not_tracked",
            "note_bank_ph", "note_buy_expense_ph", "note_expense_ph",
            "note_gstin_ph", "note_ph", "note_pod_ph", "notes_attach_party",
            "notes_attach_sale", "notify_phone_ph", "off", "offline_badge", "ok",
            "on_account_ap", "onboard_book_tmpl", "onboard_fresh_tmpl",
            "onboard_template_tmpl", "open_server", "ops_desk_help",
            "optional_login_help", "options_moklanjo_ph", "or_unlock_login", "order",
            "org_optional_ph", "out_for_delivery", "owed", "owes_word", "owner",
            "paid_by", "paid_now", "pair_now", "pairing_code_ph",
            "part_second_way_ph", "party_kind_ph", "party_name_selected_ph",
            "password_ph", "pay_ap_card", "pay_ap_cash", "pay_ap_heading",
            "pay_ap_upi", "pay_credit", "pay_selected_heading", "period_bill",
            "period_bill_party", "permission_preset", "phone_ph", "photo", "pick",
            "pick_pod_photo", "pin_count_full", "pin_count_need", "pin_count_ready",
            "pin_count_zero", "pin_hint", "pin_hint_books", "pin_hint_choose",
            "pin_hint_new", "pin_hint_pair_first", "pin_hint_paired",
            "pin_hint_shop_code", "place_name_ph", "preset_help_moklanjo", "prev",
            "prev_country", "price_ph", "print_awb_label", "print_drs", "print_open",
            "print_run_sheet",
            "product_locked_moklanjo", "product_only_moklanjo", "purchases_cogs",
            "qty", "qty_cap", "qty_example_ph", "quantity_ph", "ready_to_sell",
            "rate_carrier_ph", "rate_party_ph",
            "reason_amend_ph", "reason_note_ph", "reason_required_ph",
            "reattempt", "recent_label", "record_day_close", "record_purchase", "refresh",
            "refresh_from_shop", "refund", "remit_cod", "remove", "remove_selected",
            "reports", "reports_aging", "reports_balance_sheet", "reports_expenses",
            "reports_help", "reports_month", "reports_today", "restock",
            "restore_backup", "restore_help", "rider_cod_handed_ph", "rider_name_ph", "rider_phone_ph",
            "riders", "riders_own_fleet", "role_both", "role_carrier",
            "role_cashier", "role_consignor", "role_help_both", "role_help_carrier",
            "role_help_consignor", "role_help_supplier", "role_manager",
            "role_supplier", "role_title_ph", "rollup", "round_word", "rounded",
            "rounded_total", "sales", "sales_refunds", "save",
            "save_booking_measures", "save_brand", "save_business_id",
            "save_business_name", "save_features", "save_names", "second_way",
            "select", "selected_language", "selected_word", "sell", "sell_money_in",
            "server", "server_ip_ph", "service_pct_ph", "set_credit_limit",
            "set_login_selected", "set_owner_pin", "settle_card", "settle_cash",
            "settle_rider_cod", "settle_rider_pay", "share_receipt_whatsapp",
            "setup_app_moklanjo", "setup_complete", "shop", "shop_logo_optional",
            "mode_desk", "mode_rider", "waiting_office", "capture_signature",
            "incoming_manifest", "pickup_request", "pickup_run", "auto_assign_area",
            "reorder_stops", "vehicle_inspect", "print_docket", "franchise_statement",
            "tally_export", "open_whatsapp", "copy_sms", "pick_barcode_image",
            "delivery_feedback", "lan_self_book",
            "short_code_ph", "show_pin", "size_in", "slab_kg_ph", "sold_out",
            "split", "spreadsheet_csv", "staff", "staff_name_ph", "start_fresh",
            "start_template", "start_tracking", "starting_password_ph",
            "starting_pin_ph", "status_client_unlock", "status_server_set_pin",
            "status_server_tills", "status_server_unlock", "stock_minus",
            "stock_plus", "stock_type_help", "stop_extra_tills", "supplier_name_ph",
            "supplier_ph", "switch_device", "take_payment", "tax_added_till",
            "tax_inside_price", "tax_rate_ph", "team", "team_owner_help",
            "team_pins", "they_owe_us", "this_device", "this_shop", "till_to_bank",
            "tmpl_blurb", "tmpl_current", "tmpl_how", "tmpl_selected", "to_code_ph",
            "today_books", "today_mid", "today_prefix", "total", "uncleared_no_ph",
            "undo", "unit_at", "unit_at_cap", "unit_cost_ph", "unit_kg_blank_ph",
            "unit_name_bag_ph", "units_heading", "units_help", "unlock",
            "unlock_after_pairing", "unlock_to_continue", "unlock_with_login",
            "unmark", "update_my_password", "update_my_pin", "upi", "upi_amount_ph",
            "use", "use_carrier_cost", "vehicle_ph", "void", "we_owe_them", "weight_in", "weight_kg_ph",
            "weight_unit_code_ph", "who_unlocking", "write_off_ap",
            "write_off_heading", "write_off_nil", "your_brand", "your_favorites",
            "your_items", "zone_name_ph", "zone_ph", "contents_ph",
        ];
        for lang in LANGS {
            for key in KEYS {
                let value = t(lang, key);
                assert_ne!(value, *key, "{lang} left {key} untranslated");
                assert!(!value.is_empty(), "{lang} {key}");
                // A chrome row (or the per-language table) must cover the
                // key — falling back to the English text is a failure.
                assert!(
                    chrome_extra(lang, key).is_some() || value != en(key),
                    "{lang} falls back to English for {key}"
                );
            }
        }
        assert_eq!(t("en", "sold_out"), "Unavailable");
        assert_eq!(t("gu", "cash"), "રોકડ");
        assert_eq!(t("gu", "owed"), "બાકી");
        assert_eq!(t("hi", "sold_out"), "बिक गया");
    }

    /// Buttons must never show snake_case keys to end users.
    #[test]
    fn english_chrome_never_returns_raw_keys() {
        const KEYS: &[&str] = &[
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
