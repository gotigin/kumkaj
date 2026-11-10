//! Locale: languages, currencies and number styles for the whole world.
//!
//! Every catalogue language ships the whole static UI translated: English
//! is `en` below, every other language is `translations/<code>.tsv` in this
//! crate (see `t`). Tests fail on any untranslated key, broken placeholder
//! or character the shipped fonts cannot draw.
//!
//! **New UI string:** add it to `en()`, then
//! `python3 scripts/scaffold-ui-translations.py --app khaynjo --standin-english`
//! and `python3 scripts/fill-ui-translations.py --app khaynjo --redo-english`
//! (see `docs/DEV.md` § Languages). Never ship a label only in English.

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

/// Worldwide language catalogue — every entry is fully translated (`t`).
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
    // Other names for catalogue languages: Tagalog, Norwegian Bokmål /
    // Nynorsk, and the legacy ISO codes older Android still reports.
    let primary = match primary {
        "tl" => "fil",
        "nb" | "nn" => "no",
        "iw" => "he",
        "in" => "id",
        "ji" => "yi",
        "mo" => "ro",
        "sh" => "sr",
        other => other,
    };
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
        "party_kind_ph" => "Roles (Customer, Supplier…)",
        "reports_help" => "Day book, debtors & creditors, bank book, expense vouchers, profit & loss.",
        // Short chip labels — long titles wrap into irregular SegBtn ovals on phone.
        "reports_today" => "Today",
        "reports_month" => "This month",
        "reports_aging" => "Outstanding",
        "reports_balance_sheet" => "Balance",
        "reports" => "Bank book & P&L",
        "refresh" => "Refresh",
        "undo" => "Undo",
        "start_tracking" => "Start tracking",
        "stock_minus" => "− Stock",
        "stock_plus" => "+ Stock",
        "settle_cash" => "Settle cash",
        "settle_card" => "Settle card",
        "items" => "Menu & items",
        // Short bottom-nav / rail chrome — never the long page title.
        "nav_items" => "Menu",
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
        "empty_catalog" => "No dishes yet. Open Menu, add a dish (name + price), then take an order.",
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
        "reprint_kot" => "Print selected again",
        "reprint_kot_ok" => "Printed {n} dish(es) again · {slips} slip(s) → {kitchens}",
        "reprint_kot_needed" => "Send a KOT first — then include dishes and print selected again",
        "reprint_kot_none_selected" => "Every sent dish is skipped — tap In print on the ones to reprint",
        "reprint_kot_client" => "Print again ({n}) for {place} sent — kitchen prints at the Server",
        "print_cancelled_kot" => "Print all cancelled",
        "print_cancelled_kot_ok" => "Printed {n} cancelled dish(es) · {slips} slip(s) → {kitchens}",
        "print_cancelled_kot_needed" => "Open a table with cancelled dishes first",
        "print_cancelled_kot_none" => "No cancelled dishes on this order",
        "print_cancelled_kot_client" => "Print cancelled ({n}) for {place} sent — kitchen prints at the Server",
        "kot_banner" => "Kind banner",
        "kot_banner_ph" => "Banner (blank = none)",
        "kot_item_prefix" => "Item prefix",
        "kot_item_prefix_ph" => "e.g. [CANCELLED] ",
        "kot_strike" => "Strike dish name",
        "kot_strike_note" => "Strike dish note",
        "kot_strike_on" => "On",
        "kot_strike_off" => "Off",
        "bill_item_layout" => "Item layout",
        "bill_item_layout_columns" => "Columns",
        "bill_item_layout_stacked" => "Stacked",
        "bill_item_header" => "Column headers",
        "bill_item_hdr_item" => "Item header",
        "bill_item_hdr_qty" => "Qty header",
        "bill_item_hdr_price" => "Price header",
        "bill_item_hdr_amount" => "Amount header",
        "bill_item_name" => "Dish name",
        "bill_item_qty" => "Qty",
        "bill_item_price" => "Price",
        "bill_item_amount" => "Amount",
        "bill_item_note" => "Dish note",
        "bill_item_col_gap" => "Column gap",
        "bill_item_col_order" => "Column order",
        "bill_item_col_sep" => "Column rules",
        "bill_item_gap_tight" => "Tight",
        "bill_item_gap_comfort" => "Comfortable",
        "bill_item_gap_wide" => "Wide",
        "bill_item_num_width" => "Number columns",
        "bill_item_num_compact" => "Compact",
        "bill_item_num_comfort" => "Comfortable",
        "bill_item_num_wide" => "Wide",
        "bill_item_num_align" => "Numbers align",
        "bill_item_gap_em" => "Gap (em)",
        "bill_item_qty_min" => "Qty min (em)",
        "bill_item_price_min" => "Price min (em)",
        "bill_item_amount_min" => "Amount min (em)",
        "kot_item_qty_width" => "Qty column",
        "kot_item_num_align" => "Qty align",
        "bill_total_qty_label" => "Total qty label",
        "bill_total_subtotal_label" => "Sub total label",
        "bill_total_grand_label" => "Grand total label",
        "bill_total_paid_label" => "Paid total label",
        "bill_meta_pair" => "Share line with",
        "bill_pair_alone" => "Alone",
        "bill_item_piece_on" => "On",
        "bill_item_piece_off" => "Off",
        "bill_total_qty" => "Total qty",
        "bill_total_subtotal" => "Sub total",
        "bill_total_grand" => "Grand total",
        "reprint_incl" => "In print",
        "reprint_skip" => "Skip",
        "reprint_skip_meta" => "Skip recovery print",
        "kot_dest_hint" => "Prints on the printers you assigned to {kitchens} in Settings → Printers",
        "kot_sent_hint" => "Last KOT went to {kitchens}. Tap to add dishes for the next ticket — or Print KOT again for the same one.",
        // Send KOT outcomes (shop device) — the counter reads these mid-rush, in
        // the staff's language; the snack kind is set in code, not guessed
        // from English words (`kot_sent_status`).
        "kot_sent_printing" => "KOT sent to {kitchens} — printing",
        "kot_sent_not_printed" => "KOT sent to {kitchens} but NOT printed: {why}. Tell the kitchen, then fix the printer and print it again.",
        "kot_sent_screen" => "KOT sent to {kitchens}",
        // Waiter's tablet: queued for the Server, which prints.
        "kot_queued_shop" => "KOT for {place} → {kitchens} saved; the shop device prints it",
        "kot_job_label" => "KOT for {place} → {kitchens}",
        "kot_job_reached" => "KOT for {place} reached {kitchens}",
        "kot_unassigned" => "Kitchen (no station)",
        "kot_unassigned_held" => "Held {n} dish(es) without a station — unhold on the line to send",
        "kot_station_ph" => "Kitchen",
        "kot_station_dd" => "Kitchen",
        "kot_station_dd_n" => "Kitchen · {stations}",
        "kot_set_station" => "Use this station",
        "kot_station_applied" => "Station set on {n} dish(es): {station}",
        "kot_station_cleared" => "Station cleared from {n} dish(es): {station}",
        "kot_station_needed" => "Pick a kitchen station first",
        "kot_station_not_allowed" => "Not allowed to use kitchen: {station}",
        "kot_station_none" => "No kitchen stations yet — add one under Kitchen or Settings → Areas · stations",
        "pos_tab_items" => "Added items",
        "pos_tab_payment" => "Payment",
        "hold_send" => "Hold",
        "hold_send_on" => "Held",
        "hold_send_meta" => "Held — not this KOT",
        "station_remove" => "Remove station",
        "station_remove_ph" => "Tap a station to remove it",
        "station_removed" => "Station removed: {name}",
        "settings_hub_title" => "Settings",
        "settings_hub_help" => "Open one area at a time — less scrolling, clearer choices.",
        "settings_back" => "Back",
        "settings_sec_language" => "Language & currency",
        "settings_sec_country" => "Country & tax",
        // Owner language (not "LAN" / "Server/Client" jargon). DeviceSetup
        // still says Server Mode / Client Mode for the one-time choice.
        "settings_sec_tills" => "Other devices",
        "settings_sec_brand" => "Your brand",
        "settings_sec_units" => "Units & conversions",
        "settings_sec_pickers" => "Areas · stations · tax rates",
        "settings_sec_printers" => "Printers",
        "settings_sec_bill" => "Bill & KOT format",
        "settings_sec_selling" => "Schemes & sales",
        "bill_kot_tab_bill" => "Bill",
        "bill_kot_tab_kot" => "KOT",
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
        "sidebar_settings_help" => "Same icons as the side rail. Use ↑ ↓ to reorder. Flip the switch to show or hide. Settings and Lock always stay on the bar.",
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
        "printers_none_hint" => "Save a printer on this device first, then add it to Bill or a kitchen",
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
        "added_items" => "Added Item(s)",
        "added_items_empty" => "Nothing on this order yet — add a dish from All items or a category.",
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
        "area_new_ph" => "New area (e.g. AC, Garden)",
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
        "my_pin_settings" => "My PIN",
        "settings_hub_help_my_pin" => "Change the unlock PIN for the person signed in on this device.",
        "team_self_help" => "One sign-in: a new PIN, a new password, or both. Leave a box blank to keep what you have. Ask the Owner if you need different screens or rights.",
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
        "server_remove_pin_ph" => "Owner PIN to remove a server",
        "only_owner_remove_server" => "Only the Owner can remove a server",
        "owner_pin_remove_server" => "Owner PIN required to remove this server",
        "device_setup_help" => "Server keeps the books here. Client pairs to a Server.",
        // One-shot after create server (before Owner PIN). Menu-only v1.
        "app_template_heading" => "Rajdhani App Template",
        "app_template_blurb" => "Loads the Rajdhani menu; Owner Zahir (zahir), Admin Asif (asif, PIN 1111), Waiter Nawaz (nawaz, PIN 0000); FOH sidebar; history clears after 4 hours (and Clear now in Backup); Today strip off; compact KOT + Standard margins for narrow trays. Sidebar peers can be turned back on in Settings.",
        "app_template_apply" => "Rajdhani App Template",
        "app_template_skip" => "Skip for now",
        "app_template_status" => "Server ready — apply Rajdhani App Template, or skip",
        "app_template_importing" => "Applying {title}…",
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
        "datetime_appearance" => "Date & time appearance",
        "datetime_appearance_help" => "Books keep year-first RFC3339 inside.\n\nThese only change how dates and clocks look on bills, KOT, reports, and the screen.",
        "datetime_example" => "Example now: {ex}",
        "date_order" => "Date order",
        "date_order_dmy" => "D/M/Y",
        "date_order_mdy" => "M/D/Y",
        "date_order_ymd" => "Y-M-D",
        "time_format" => "Clock",
        "time_format_24h" => "24-hour",
        "time_format_12h" => "12-hour",
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
        // Short: long form crops inside FieldInput on phone (no wrap on empty_text).
        "birthday_ph" => "Birthday (MM-DD)",
        "crm_message_ph" => "Birthday message ({name}, {points}…)",
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
        "tax_chrome" => "Tax labels on till",
        "tax_chrome_help" => "When off, the order desk hides GST/tax estimate lines and GST-in-price placeholders. Tax rates and Country & tax settings stay — turn labels back on any time. Rajdhani App Template starts with this off.",
        "tax_chrome_on" => "Show",
        "tax_chrome_off" => "Hide",
        "tax_chrome_saved_on" => "Tax labels on till",
        "tax_chrome_saved_off" => "Tax labels hidden on till",
        "tax_invoice_till" => "Tax invoice on till",
        "tax_invoice_help" => "When off, bills are Cash memo only and the Tax invoice / Cash memo chips are hidden on the order desk. Rajdhani App Template starts with this off — turn Show back on any time to pick Tax invoice again.",
        "tax_invoice_on" => "Show",
        "tax_invoice_off" => "Hide",
        "tax_invoice_saved_on" => "Tax invoice on till",
        "tax_invoice_saved_off" => "Tax invoice hidden — Cash memo only",
        "pay_methods" => "Payment methods",
        "pay_methods_help" => "Which chips appear on Take payment. Cash always stays. Rajdhani App Template starts with Cash only — turn UPI / Card / Credit / Split back on here.",
        "create_backup" => "Create backup",
        "credit_limit_heading" => "Credit limit (selected / named party)",
        "day_close_summary" => "Day close summary",
        "delete_party" => "Delete selected party",
        "delivery" => "Delivery",
        "done_open_books" => "Done — open books",
        "drawer_count" => "Drawer count (optional)",
        "expense_heading" => "Expense",
        "export_day" => "Export day",
        "extra_note_bill_ph" => "Extra note on this bill",
        "extra_note_expense_ph" => "Extra note on last expense",
        "extra_tills_heading" => "Extra tills",
        "other_devices_help_heading" => "Set up other devices",
        "other_devices_help" => "On this device (Server): Other devices → Allow extra tills. Note the pairing code and Wi‑Fi address. Tap Allow other devices to connect if it appears. On a computer, tap Start with this computer. On each other device: install the app, choose Client Mode, Find the shop's server (or type the address), enter the pairing code, Pair now. Orders wait on the device if Wi‑Fi drops and go when the Server answers again. Kitchen printers are set on the Server (Settings → Printers) — extra tills send KOTs there.",
        "forget_shop" => "Forget this shop",
        "goods_received" => "Goods received",
        "import_day" => "Import day",
        "import_items_csv_plain" => "Import items CSV…",
        "import_parties_csv" => "Import parties CSV…",
        "item_name" => "Item name",
        "kitchen_printing" => "Printers",
        "login_id_ph" => "Username",
        "mark_cleared" => "Mark cleared",
        "new_party_clear" => "New party (clear form)",
        "new_party_kind_supplier_ph" => "New party kind (Supplier)",
        "next_country" => "Next country",
        "next_order" => "Next order (clear desk)",
        "note_gstin_ph" => "Note / GSTIN / reminder",
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
        "reason_amend_ph" => "Reason (amend / void / refund)",
        "reason_note_ph" => "Reason / note (wastage…)",
        "record_day_close" => "Record day close",
        "refresh_from_shop" => "Refresh from shop",
        "refund" => "Refund",
        "remove_selected" => "Remove selected",
        "request_bill" => "Request bill",
        "restaurant_name_ph" => "Restaurant name (optional)",
        "restore_backup" => "Restore backup",
        "role_both" => "Both",
        "role_admin" => "Admin",
        "role_cashier" => "Cashier",
        "role_customer" => "Customer",
        "role_manager" => "Manager",
        "history_retention" => "Trading history",
        "history_retention_help" => "Auto-clear old sales and KOTs. Menu, staff, and settings stay. Clear now wipes all trading rows and keeps setup.",
        "history_off" => "Keep",
        "history_day_close" => "On day close",
        "history_after_hours" => "After hours",
        "history_hours_ph" => "Hours (1–168, default 4)",
        "history_hours_save" => "Save hours",
        "history_clear_now" => "Clear history now",
        "history_cleared" => "Trading history cleared — setup kept",
        "history_retention_saved" => "History clearing saved",
        "history_hours_saved" => "History hours saved",
        "history_owner_only" => "Only Owner or Admin can clear history",
        "rollup" => "Roll-up",
        "save_brand" => "Save brand",
        "save_business_id" => "Save business id",
        "save_business_name" => "Save business name",
        "save_features" => "Save",
        "save_features_named" => "Save {name}",
        "save_features_ok" => "Saved setup for {name}",
        "save_names" => "Save names",
        "second_way" => "Second way",
        "server_ip_ph" => "Server IP (e.g. 192.168.1.10)",
        "set_credit_limit" => "Apply limit to party",
        "set_login_selected" => "Apply login to selected",
        "set_login_named" => "Apply login to {name}",
        "shop" => "Shop",
        "show" => "Show",
        "spreadsheet_csv" => "Spreadsheet (CSV)",
        "start_fresh" => "Start fresh",
        "stop_extra_tills" => "Stop extra tills",
        "floor_cols_ph" => "Columns",
        "floor_cols_fit" => "Fit screen",
        "floor_cols_fit_sub" => "{n} fit here",
        "floor_cols_fits" => "fits",
        "floor_cols_scroll" => "scrolls sideways",
        "floor_cols_min" => "Columns: at least 1 — or leave it empty to fit the screen",
        "floor_zoom_auto" => "Auto",
        "floor_zoom_out" => "Zoom out",
        "floor_zoom_in" => "Zoom in",
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
        "copy_pair_code" => "Copy pairing code",
        "new_pair_code" => "New pairing code",
        "pair_finishing" => "Finishing pair… applying the shop's books",
        "pair_opening" => "Opening the till…",
        "pair_copied" => "Pairing code copied",
        "kot_waiting" => "Waiting to send",
        "lan_offline_waiting" => "Shop server unreachable · {n} waiting",
        "lan_connecting" => "Connecting to shop…",
        "lan_live" => "Live",
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
        "printers_heading" => "Printers",
        "printers_help" => "Find printers, tap one to select it, then Print a test page on that device only. Bill printers: Bill & KOT format → Bill. Kitchen printers: Kitchen → Kitchen stations (add a station first — Find does not create stations).",
        "printers_host_only" => "Printers are set on the shop device (Server). This till sends bills and KOTs there — Find / add / assign stay on the Server. Test page and bill print size go to the Server.",
        "printers_find_heading" => "Find printers",
        "printers_find_tip" => "Find scans this device and keeps the list up to date.",
        "printers_pool_heading" => "Printers",
        "printers_pool_tip" => "Find scans this device and keeps this list up to date.",
        "printers_pool_empty" => "(none yet — tap Find)",
        "printers_routes_help" => "Find printers the operating system already knows. Assign them to the bill and to each kitchen under Bill & KOT format.",
        "printer_assign_dd" => "Real printer: {place}",
        "printer_assign_pick" => "Select real printer",
        "printer_profile_heading" => "Layout",
        "printer_send_heading" => "Delivery",
        "vp_delivery_heading" => "Delivery",
        "vp_delivery_help" => "How this virtual printer sends jobs — OS document, raw bytes, or ESC/POS. Slip content (type size, wrap, margins) is decided by the bill / KOT format, not here.",
        "vp_type_size_heading" => "Type size",
        "vp_wrap_heading" => "Wrap width",
        "vp_margin_heading" => "Margins",
        "vp_delivery_spooler" => "OS document (system spooler)",
        "vp_delivery_raw" => "Raw bytes to device",
        "vp_delivery_escpos" => "ESC/POS thermal commands",
        "virtual_printer_new" => "New virtual printer",
        "virtual_printer_selected" => "Editing “{name}” — tap another in the list to switch",
        "virtual_printer_tap_list" => "Tap a virtual printer in the list to edit it",
        "config_layout_portrait" => "Portrait",
        "config_layout_landscape" => "Landscape",
        "config_send_os" => "System spooler",
        "config_send_escpos" => "ESC/POS",
        "profiles_heading" => "Layouts",
        "profiles_empty" => "(no layouts yet)",
        "profile_builtin" => "built-in",
        "profile_custom" => "custom",
        "profile_name_ph" => "Layout name",
        "profile_new" => "New layout",
        "profile_delete" => "Delete layout",
        "profile_created" => "Layout created",
        "profile_deleted" => "Layout deleted",
        "profile_builtin_keep" => "Built-in layouts cannot be deleted",
        "config_saved" => "Updated",
        "config_selected" => "Selected “{name}”",
        "printer_output_heading" => "Virtual printers",
        "virtual_printers_heading" => "Virtual printers",
        "virtual_printers_empty" => "No virtual printers yet — set values below, then Save. After you have some, tap one in the list to edit.",
        "virtual_printer_name_ph" => "Virtual printer name",
        "virtual_printer_create" => "Create virtual printer",
        "virtual_printer_save" => "Save virtual printer",
        "virtual_printer_delete" => "Delete virtual printer",
        "virtual_printer_created" => "Virtual printer “{name}” created",
        "virtual_printer_deleted" => "Deleted virtual printer “{name}”",
        "virtual_printer_updated" => "Virtual printer updated",
        "virtual_printer_updated_named" => "Virtual printer “{name}” updated",
        "virtual_draft_layout" => "Values set for next virtual printer",
        "virtual_draft_send" => "Value set for next virtual printer",
        "virtual_need_real" => "Select a real printer first",
        "virtual_need_select" => "Select a virtual printer to delete",
        "real_printer_selected" => "Selected: {name}",
        "station_group_heading" => "Virtual printers",
        "station_group_apply" => "Delete virtual printer",
        "station_group_applied" => "Deleted",
        "station_group_empty" => "Select a virtual printer",
        "station_group_name_ph" => "Virtual printer name",
        "station_group_save" => "Create virtual printer",
        "station_group_saved" => "Virtual printer saved",
        "station_manage_heading" => "Add or remove a kitchen station",
        "printers_routes_heading" => "Where things print",
        "route_pick_heading" => "Place",
        "route_editing" => "Editing: {name}",
        "route_assigned" => "Added {printer} to {place}",
        "route_already_assigned" => "Already on this place: {printer}",
        "route_removed" => "Removed {printer} from {place}",
        "route_cleared" => "Cleared {name}",
        "route_assigned_empty" => "None on this place",
        "route_tap_remove" => "tap to remove",
        "route_tap_select" => "tap to select",
        "route_tap_selected" => "selected",
        "route_pick_printer_first" => "Tap a printer first",
        "route_assigned_heading" => "Printers",
        "route_add_heading" => "Assign",
        "printer_label_ph" => "Name (e.g. Kitchen)",
        "printer_target_ph" => "Printer name or host:9100",
        "printer_target_default" => "this computer's usual printer (blank address)",
        "printer_wire_heading" => "Profile",
        "printer_wire_escpos" => "ESC/POS",
        "printer_wire_os" => "Portrait",
        "printer_wire_os_portrait" => "Portrait",
        "printer_wire_os_landscape" => "Landscape",
        "printer_wire_escpos_short" => "ESC/POS",
        "printer_wire_os_short" => "Portrait",
        "printer_wire_os_landscape_short" => "Landscape",
        "printer_format_custom" => "New custom profile",
        "printer_format_custom_label" => "Custom",
        "printer_format_custom_added" => "Custom profile added for this place",
        "add_printer" => "Add printer",
        "remove_printer" => "Remove",
        "route_pick_ph" => "Bill or a kitchen station",
        "route_bill" => "Bill",
        "route_default_kitchen" => "No station (unassigned dishes)",
        "route_kot_station" => "Kitchen",
        "route_dont_print" => "No printer",
        "stations_heading" => "Kitchen stations",
        "stations_help" => "Add kitchen stations for dishes and KOT. On each station, attach its printers (found under Settings → Printers), set priority, and choose First that works or Print on all. Changes save as you go — no Save button.",
        "station_print_heading" => "Printers for this station",
        "station_print_help" => "Pick a station, then attach one or more printers. ↑↓ sets try order. First that works stops after the first success; Print on all sends to every attached printer. Each change saves immediately.",
        "station_print_pick_ph" => "Station to set printers",
        "station_print_pick_first" => "Pick a kitchen station first",
        "station_print_editing" => "Editing printers for “{name}”",
        "station_vps_dd" => "Printers for this station",
        "station_vps_dd_n" => "{n} printer(s) · tap to change",
        "station_vps_empty" => "No printers yet — Find them under Settings → Printers",
        "bill_printers_dd" => "Printers for the bill",
        "bill_printers_dd_n" => "{n} printer(s) for the bill · tap to change",
        "job_layout" => "Layout",
        "job_delivery" => "How the job is sent",
        "job_delivery_help" => "OS document, raw bytes, or ESC/POS. The bill and the kitchen ticket each keep their own. Either job can go to any printer Find has saved.",
        "station_vp_attached" => "Printer attached",
        "station_vp_detached" => "Printer removed from station",
        "station_print_summary" => "{mode} · {n} printer(s)",
        "station_print_summary_none" => "No printers on this station yet — it will not print",
        "stations_none" => "No stations yet — add one below. Menu items can use a station so their KOT prints there.",
        "route_up" => "Up",
        "route_down" => "Down",
        "route_remove" => "Remove",
        "route_clear" => "Clear",
        "save_printers" => "Save printers",
        "print_test_page" => "Print a test page",
        "no_printer_assigned" => "No printer",
        "test_page_no_printer" => "No printer for {place} — attach a printer to it, then try again",
        "test_page_pick_first" => "Pick printer(s) for a test page first — then Print a test page",
        "test_printers_dd" => "Printers for a test page",
        "test_printers_dd_n" => "{n} printer(s) for a test page · tap to change",
        "kot_test_printers_dd" => "Printers for Test KOT",
        "kot_test_printers_dd_n" => "{n} printer(s) for Test KOT · tap to change",
        "test_printer_picked" => "Test on: {name}",
        "test_printer_cleared" => "Not testing on: {name}",
        "test_page_sent_n" => "Test page sent to {n} printer(s)",
        "test_kot_pick_first" => "Pick printer(s) for Test KOT first — then Test KOT",
        "test_kot_sent_n" => "Test KOT sent to {n} printer(s)",
        "cancel_pick" => "Mark",
        "cancel_pick_on" => "Marked",
        "cancel_pick_meta" => "Marked for cancel",
        "batch_cancel" => "Cancel marked",
        "batch_cancel_n" => "Cancel marked ({n})",
        "batch_cancel_none" => "Mark one or more sent dishes first — then Cancel marked",
        "batch_cancel_title" => "Cancel marked dishes — one reason, one cancel slip per kitchen",
        "batch_cancel_ok" => "{n} dish(es) cancelled · {slips} cancel slip(s) → {kitchens}",
        "batch_cancel_ok_noprint" => "{n} dish(es) cancelled — KOT auto-print is off, so no cancel slip",
        "no_printer_kitchen_station" => "No printer for that kitchen station — Kitchen → Kitchen stations → Printers for this station",
        "printers_saved" => "Saved",
        "test_page_sent" => "Test page sent — check the printer",
        "test_page_sent_one" => "Test page sent to {name} — check that printer",
        "printers_found" => "Found {n}: {list}",
        "printers_find_synced" => "Found {n} — {new} new",
        "printer_on_device" => "saved",
        "printer_already_added" => "already added",
        "printer_already_saved" => "Already saved: {name}",
        "find_printers" => "Find printers",
        "printers_looking" => "Looking…",
        "saving_printer" => "Adding printer…",
        "cancelled" => "Cancelled",
        "cancelling" => "Cancelling…",
        "removing" => "Removing…",
        "removed" => "Removed",
        "updating_order" => "Updating…",
        "order_updated" => "Order updated",
        "printers_tap_find" => "Tap Find to list printers this device can use.",
        "printers_none_found" => "None found — type a system printer name or an IP (e.g. 192.168.1.50).",
        "printer_from_find" => "Added {name}",
        "kot_printer_ph" => "Printer for KOTs",
        "bill_printer_ph" => "Printer for bills",
        "kot_print_on" => "Auto-print kitchen tickets: On",
        "kot_print_off" => "Auto-print kitchen tickets: Off",
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
        "tax_added_till" => "{tax} exclusive",
        "tax_inside_price" => "{tax} inclusive",
        "tax_invoice" => "Tax invoice",
        "till_to_bank" => "Till → bank",
        "units_heading" => "Units",
        "unlock_after_pairing" => "Enter PIN",
        "unlock_with_login" => "Unlock with login",
        "unmark" => "Unmark",
        "update_my_password" => "Update my password",
        "update_my_pin" => "Update my sign-in",
        "duress_heading" => "Emergency unlock PIN (Owner only)",
        "duress_help" => "Optional. If this PIN is typed at unlock instead of yours, all books entries are wiped (sales, ledgers, history) and the till opens as Owner looking normal — menu, prices, tables, staff and settings stay. Schema and config are not deleted. Use only if you are forced to unlock. Leave blank and save to turn off. Never the same as anyone's PIN.",
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
        "change_my_pin_heading" => "My sign-in",
        "connect_to_server" => "Connect to the Server",
        "counter_locked" => "Counter 01 · Locked",
        "country_pack_help" => "A country pack sets the tax name and maths, the instant-pay rail, and the business id printed on bills. Kumkaj does not file taxes or issue government e-invoices anywhere.",
        "currency_use_help" => "Scroll and tap Use for any supported currency.",
        "day_close_help" => "Recording a day close saves this summary (and drawer variance if counted) to the audit log. It does not stop selling.",
        "expenses_word" => "Expenses",
        "feature_access_help" => "What they can open — optional; the job above already sets a default",
        "footer_local" => "Local SQLite · Free Forever · Zero Cloud",
        "fresh_book_khaynjo" => "Fresh = empty menu (add dishes later in Menu). Book = type opening items and who owes you below.",
        "fresh_selected" => "Fresh selected — catalogue stays empty. Tap Done when ready.",
        "gross_word" => "Gross",
        "how_start" => "How do you want to start?",
        "kot_print_help_khaynjo" => "When On, each kitchen prints its own KOT paper as soon as you Send.",
        "later_country_pack" => "Later: Settings → apply your country pack for tax and pay labels.",
        "local_badge" => "LOCAL",
        "mix_type_hint_khaynjo" => "Split across cash / UPI / card / credit if needed. Credit needs a customer above.",
        "money_in_sales" => "Money in (sales)",
        "net_word" => "Net",
        "offline_badge" => "OFFLINE",
        "optional_login_help" => "Optional login (id + password). Owner sets once; only the user can change password later.",
        "or_unlock_login" => "Or unlock with login",
        "permission_preset" => "Job (sets screens and money actions)",
        "preset_help_khaynjo" => "Cashier: sell. Manager: sell + items + void/amend.",
        "product_locked_khaynjo" => "Khaynjo",
        "product_only_khaynjo" => "This app is Khaynjo only — restaurant operating system. Other trade faces ship as separate Kumkaj products.",
        "purchases_cogs" => "Purchases (COGS)",
        "qty_cap" => "QTY",
        "restore_help" => "Restore: pick a backup. The books open as they were then; the books as they are now are saved first, so this can be undone.",
        "wipe_data_heading" => "Delete local data",
        "wipe_data_help" => "Owner only — same choices as Windows uninstall. Business records clears trading history but keeps menu and setup (like the emergency unlock PIN). Shop setup removes the books database. Screen preferences clears window/language. Confirm with the Owner PIN. The emergency unlock PIN on Team is separate (covert wipe at unlock).",
        "wipe_trading" => "Business records",
        "wipe_schema" => "Shop setup",
        "wipe_ui" => "Screen preferences",
        "wipe_owner_pin_ph" => "Owner PIN to delete",
        "wipe_delete_selected" => "Delete selected",
        "wipe_owner_only" => "Only the Owner can delete local data.",
        "restore_owner_only" => "Only the Owner can restore a backup.",
        "wipe_owner_pin_needed" => "Enter the Owner PIN to delete local data.",
        "wipe_nothing_selected" => "Choose what to delete first.",
        "wipe_done" => "Selected local data deleted.",
        "wipe_schema_done" => "Shop setup wiped — choose Server or Client to continue.",
        "rounded_total" => "Rounded Total",
        "setup_app_khaynjo" => "Set up Khaynjo",
        "shop_logo_optional" => "Shop logo (optional)",
        "stock_type_help" => "Type the item name (part of it is enough), a quantity, then adjust. Items that are not counted yet need Start tracking once.",
        "team_owner_help" => "Leave the person blank to add someone: name, job, username, PIN, and an optional password, then Add. Pick a person to edit that same kind of form — name, username, password, screens, and rights — then Save. Your own PIN and password are one sign-in at the bottom. The Owner does not change anyone else's PIN.",
        "staff_name_ph" => "Staff name",
        "staff_copy_hire_ph" => "Copy setup from (optional)",
        "staff_copy_hire" => "Use this person's setup",
        "staff_edit_name_ph" => "Display name",
        "staff_rename" => "Save name",
        "staff_copy_edit_ph" => "Copy setup from someone",
        "staff_copy_edit" => "Copy their setup onto selected",
        "staff_copy_edit_onto" => "Copy their setup onto {name}",
        "staff_deactivate_named" => "Remove {name}",
        "staff_renamed" => "Name saved",
        "staff_setup_copied_hire" => "Setup copied from {name} — set a new name and PIN, then Add staff",
        "staff_setup_copied_edit" => "Copied {from}'s setup onto {to}",
        "staff_copy_need_selected" => "Select who to edit first",
        "staff_copy_need_source" => "Pick whose setup to copy",
        "staff_copy_same" => "That is the same person — pick someone else",
        "staff_rename_need_selected" => "Select who to rename first",
        "team_pick_hint" => "Pick who to edit — screens, money rights, and kitchen stations appear for that person only.",
        "team_stations_help" => "Kitchen stations they may use",
        "team_stations_changed" => "Kitchen stations changed — Save setup for this person",
        "team_features_changed" => "Screens changed — Save setup for this person",
        "team_rights_changed" => "Rights changed — Save setup for this person",
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
        "enter_owner_passcode" => "ENTER OWNER PIN",
        "enter_staff_passcode" => "ENTER STAFF PIN",
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
        "against_bill_ph" => "Against bill # (optional)",
        "aliases_ph" => "Aliases (other names)",
        "amount_blank_full_ph" => "Amount (blank = full balance)",
        "amount_owed_ph" => "Amount owed",
        "amount_ph" => "Amount",
        "amount_received_ph" => "Amount received",
        "amount_we_pay_ph" => "Amount we pay",
        "business_name_ph" => "Business name",
        "category_new_ph" => "Category (created if new)",
        "code_bag_ph" => "Code (bag)",
        "conv_factor_ph" => "Factor (25 = 1 bag → 25 kg)",
        "country_code_ph" => "Country code (IN, GB…)",
        "credit_khata_amount_ph" => "Credit / khata amount",
        "credit_limit_ph" => "Credit limit (0 = none)",
        "from_code_ph" => "From code (bag)",
        "item_name_lang_ph" => "Item name (your language)",
        "login_id_opt_ph" => "Username (required, unique)",
        "manager_pin_ph" => "Manager PIN (if required)",
        "new_branch_name_ph" => "New branch name",
        "new_password_ph" => "New password (optional)",
        "new_pin_ph" => "New PIN (4–12 digits)",
        "note_bank_ph" => "Note (deposit, withdrawal, …)",
        "note_buy_expense_ph" => "Note (fuel, rent, …)",
        "note_expense_ph" => "Note (fuel, office, …)",
        "options_khaynjo_ph" => "Options (Size*: Small=199…)",
        "part_second_way_ph" => "Second pay way (optional)",
        "phone_ph" => "Phone",
        "place_name_ph" => "Name of this place",
        "qty_example_ph" => "Qty (e.g. 1.5 for kg)",
        "quantity_ph" => "Quantity",
        "reason_required_ph" => "Reason (required)",
        "role_title_ph" => "Role title (e.g. Barista)",
        "service_pct_ph" => "Service % (0 / 10 / 12)",
        "short_code_ph" => "Short code (MG02)",
        "starting_password_ph" => "Starting password (optional)",
        "starting_pin_ph" => "Starting PIN (4–12 digits)",
        "station_name_ph" => "Station name (e.g. Tandoor)",
        "station_new_ph" => "Kitchen station (optional)",
        "table_name_ph" => "Table name (e.g. Patio 1)",
        "tax_rate_ph" => "Tax rate or % (e.g. 5, 18)",
        "to_code_ph" => "To code (kg)",
        "uncleared_no_ph" => "Uncleared # above",
        "unit_cost_ph" => "Unit cost",
        "unit_name_bag_ph" => "Name (Bag)",
        "unit_plate_ph" => "Unit (blank = each)",
        "pay_credit" => "Credit",
        "book_kind_today" => "Today books",
        "book_kind_month" => "This month books",
        "book_kind_aging" => "Outstanding / debit-credit",
        "book_kind_balance" => "Balance sheet",
        "order_help" => "Tap an item to add it.",
        "floor_help" => "Tap a table to take or change the order. Colours show free → seated → kitchen → ready → eating → bill → paid. Waitlist, reservations, and add-tables are in the sections below.",
        "kitchen_help" => "Kitchen tickets (KOT), newest last. Start → Ready → Done moves a ticket along. Tickets never show prices.",
        "kitchen_empty" => "No open tickets. A KOT sent from Order or Floor shows up here.",
        "add_a_table" => "Add a table",
        "add_station_heading" => "Add a kitchen station",
        "categories_help" => "Tap Use to file the item above under that category. Photo gives the category a picture; without one it shows its initials.",
        "dish_search_ph" => "Item name",
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
        "sizes_as_items_ph" => "Sizes (Half=120, Full=200)",
        "dish_group_ph" => "Dish group (optional)",
        "discount" => "Discount",
        "remove_discount" => "Remove discount",
        "apply_discount" => "Apply discount",
        "save_opening_cash" => "Save opening cash",
        "close_shift_print" => "Close shift and print slip",
        "area_prices_ph" => "Area prices (AC=220…)",
        "discount_ph" => "10% or 50",
        "discount_reason_ph" => "Reason (regular guest, late…)",
        "opening_float_ph" => "Opening cash in the drawer",
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
        "category_search_ph" => "Category",
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
        "editing_staff" => "Editing",
        "finding_server" => "Looking for the shop's server on this Wi‑Fi…",
        "no_server_found" => "None found. On the Server: Settings → Other devices → Allow extra tills (same Wi‑Fi). Or type the IP shown there.",
        "servers_found_pick" => "Tap the shop you want to connect to.",
        "server_host_selected" => "Using {host} — enter the pairing code, then Pair now.",
        "print_bill" => "Print bill",
        "table_actions" => "Table actions",
        "move_merge_table" => "Move / merge table",
        "move_help" => "Move this table's order to a free table, or merge it into a running one.",
        "move_target_ph" => "Table name (e.g. D5, P2)",
        "approval_pin_ph" => "Manager PIN (only if asked)",
        "move_here" => "Move here",
        "merge_into" => "Merge into it",
        "line_reason_ph" => "Reason (changed mind, out of stock…)",
        "line_qty_ph" => "Qty to cancel (blank = all)",
        "line_target_ph" => "Move to table (e.g. D5)",
        "confirm" => "Confirm",
        "menu_search_ph" => "Search item or short code",
        "customer_paid_ph" => "Customer paid",
        "tip_ph" => "Tip",
        "table_area_ph" => "Area (Dine In, Parcel…)",
        "table_range_ph" => "Many at once: D1-D20",
        "add_these_tables" => "Add these tables",
        "remove_these_tables" => "Remove these tables",
        "table_remove_ph" => "Remove: T1-T24 or one name",
        "find_server" => "Find the shop's server",
        "bill_format" => "Bill format",
        "bill_format_help" => "Printed under your restaurant's name on every bill. Separate lines with | (e.g. MG Road, Rajkot | Ph 98250 00000).",
        "bill_header_ph" => "Address | phone",
        "fssai_ph" => "FSSAI licence no. (India)",
        "bill_footer_ph" => "Footer (e.g. Thank you)",
        "save_bill_format" => "Save bill format",
        "reset_bill_format" => "Reset to default",
        "bill_format_reset" => "Bill format reset to default",
        "reset_kot_format" => "Reset to default",
        "kot_format_reset" => "KOT format reset to default",
        "kot_format" => "KOT format",
        "kot_format_help" => "Tap a field in the list. Show, font size, and weight apply to that field on the KOT kind selected above. Each kind keeps its own sizes and weights.",
        "kot_format_fields" => "Fields",
        "kot_field_next" => "Next field",
        "kot_field_show" => "Show",
        "kot_field_hide" => "Hide",
        "kot_field_on" => "show",
        "kot_field_off" => "hide",
        "kot_field_size" => "Font size",
        "kot_field_size_ph" => "e.g. 16, 20, 24, 28, 32…",
        "kot_weight_light" => "Light",
        "bill_kot_panel_help" => "Bill and kitchen slip layout, each with its own content width. Bill Cashier is who issued or settled the bill; KOT Waiter is who sent that kitchen ticket — the full order story stays in the books.",
        "std_size_ph" => "e.g. 18, 20, 26…",
        "kot_show_shop" => "Shop name",
        "kot_hide_shop" => "Hide shop",
        "kot_show_time" => "Time",
        "kot_hide_time" => "Hide time",
        "kot_show_waiter" => "Waiter",
        "kot_hide_waiter" => "Hide waiter",
        "kot_show_station" => "Station",
        "kot_hide_station" => "Hide station",
        "kot_show_table" => "Table",
        "kot_hide_table" => "Hide table",
        "kot_show_kot_no" => "KOT No",
        "kot_hide_kot_no" => "Hide KOT No",
        "kot_width" => "Preview wrap (set content width on the KOT tab)",
        "kot_width_58" => "58 mm",
        "kot_width_80" => "80 mm",
        "kot_kind_heading" => "KOT kind",
        "kot_kind_original" => "Original",
        "kot_kind_duplicate" => "Duplicate",
        "kot_kind_modified" => "Modified",
        "kot_kind_cancelled" => "Cancelled",
        "kot_style_heading" => "Field type size & weight",
        "kot_style_title" => "Title",
        "kot_style_item_qty" => "Item · Qty",
        "kot_style_field_pick" => "Select field",
        "kot_weight_regular" => "Regular",
        "kot_weight_bold" => "Bold",
        "kot_align" => "Align",
        "kot_align_start" => "Start",
        "kot_align_center" => "Center",
        "kot_align_end" => "End",
        "content_width" => "Content width (mm)",
        "content_width_bill" => "Bill content width (mm)",
        "content_width_kot" => "KOT content width (mm)",
        "content_width_ph" => "Content width 40–210 mm",
        "content_width_help" => "How wide this slip prints. Bill and KOT each keep their own width.",
        "content_width_bill_help" => "How wide the guest bill prints. Default 210 mm (page). 58 mm narrow slip, 80 mm wide slip, 112 mm 4-inch, 210 mm A4-wide. Type any value from 40 to 210. Kitchen tickets have their own width on the KOT tab.",
        "content_width_kot_help" => "How wide the kitchen ticket prints. Default 80 mm. 58 mm narrow slip, 80 mm wide slip, 112 mm 4-inch, 210 mm page. Type any value from 40 to 210. Guest bills have their own width on the Bill tab.",
        "vp_paper_width" => "Content width (mm)",
        "vp_paper_width_ph" => "Content width 40–210 mm",
        "vp_paper_width_help" => "Content width is set under Bill & KOT format — separately for the bill (default 210 mm) and the kitchen ticket (default 80 mm).",
        "kot_size" => "Font size",
        "kot_size_s" => "Small",
        "kot_size_m" => "Normal",
        "kot_size_l" => "Large",
        "std_size" => "Font size",
        "kot_copies" => "Copies",
        "kot_copies_1" => "1",
        "kot_copies_2" => "2",
        "kot_copies_3" => "3",
        "kot_title_ph" => "Title (blank = kitchen default)",
        "kot_footer_ph" => "Kitchen note (optional)",
        "kot_preview" => "Preview",
        "save_kot_format" => "Save KOT format",
        "test_kot_format" => "Test KOT",
        "test_kot_sent" => "Kitchen ticket preview sent",
        "test_bill_format" => "Test bill",
        "test_bill_sent" => "Bill preview sent",
        "test_print_no_printer" => "No printer for that job — choose one on this screen",
        "kot_format_saved" => "KOT format saved",
        "std_print" => "Bill print size",
        "std_print_help" => "Font size for Standard Portrait and Landscape bill jobs on OS / document printers (enter points — not Small/Normal/Large). Same size on both orientations. Kitchen slip sizes are on the KOT tab.",
        "std_size_s" => "Small",
        "std_size_m" => "Normal",
        "std_size_l" => "Large",
        "save_std_print" => "Save bill print size",
        "std_print_saved" => "Bill print size saved",
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
        "team_pick_ph" => "Staff to edit (name or username)",
        "select_staff" => "Select this staff",
        "rights_help" => "Money & floor actions for this person — tap to cycle: Allowed → Needs boss PIN → Not allowed. Print bill again is bills only; kitchen stations are the checklist below.",
        "dish_picker_help" => "Search or browse our dishes and tap one to fill its name and photo — or keep your own name. Set your price (0 is fine), then Add item.",
        "close_dish_list" => "Close dish list",
        "own_item_name" => "Use this as your own item name",
        "on_your_menu" => "On your menu",
        "edit" => "Edit",
        "schemes" => "Schemes / offers",
        "schemes_help" => "BOGO, percent-off, and fixed combos. Combos expand to kitchen lines on Send KOT; components stay held until the combo fires. Example: Buy 2 Thali get 1 Chaas",
        "scheme_bogo_ph" => "Buy 2 Thali get 1 Chaas",
        "add_scheme" => "Add BOGO",
        "schedule_ph" => "Hours (e.g. Mon-Fri 07:00-11:00)",
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
        "price_ph_gst_inside" => "Price ({tax} inclusive)",
        "cancel_needs_table" => "Open this order from Floor (a table) to cancel after the kitchen has it — counter KOTs stay on the ticket until you take payment",
        "tax_rates_missing_hint" => "No {tax} rate on these dishes — on Menu set Tax rate (prices already include {tax} under {tax} inclusive)",
        "taxable_value" => "Taxable value",
        "gst_inside_price_note" => "({tax} inclusive)",
        "start_with_template" => "Start with our menu template",
        "template_heading" => "Our menu template",
        "template_info" => "{c} categories and {d} dishes with photos, ready to add. Prices start at ₹0 — set yours after.",
        "template_import" => "Import complete template",
        "template_confirm" => "Add {d} dishes with their categories? Dishes you already have stay as they are.",
        "template_yes" => "Yes, import",
        "template_importing" => "Importing menu…",
        "template_import_progress" => "Importing… {d}/{t}",
        "template_done" => "Every dish from our template is on your menu.",
        "template_imported" => "Added {d} dishes and {c} categories — set their prices on this screen",
        "client_template_heading" => "Client menu",
        "client_template_info" => "{title}: {c} categories, {d} dishes with prices, photos, and aliases. Missing dishes only — names you already have stay.",
        "client_template_import" => "Import this menu",
        "client_template_confirm" => "Add {d} dishes from {title}? Dishes you already have stay as they are.",
        "client_template_importing" => "Importing {title}…",
        "client_template_done" => "Every dish from {title} is on your menu.",
        "client_template_imported" => "Added {d} dishes and {c} categories from {title}",
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
        "delivery_by_ph" => "Delivery by (rider / staff)",
        "kitchen_all_day" => "To cook now: {items}",
        "kitchen_all_day_head" => "To cook now",
        "kitchen_tab_live" => "Live",
        "kitchen_tab_recall" => "Recall",
        "kitchen_tab_stations" => "Stations",
        "split_bill" => "Split bill",
        "split_go" => "Make separate bill",
        "cancel" => "Cancel",
        "opening_client" => "Opening client…",
        "preparing_desk" => "Preparing desk…",
        "opening_server" => "Opening server…",
        "creating_server" => "Creating server…",
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
        // Open dropdown face: Done · <closed label> (no second Done button).
        "pick_done_dd" => "Done · {label}",
        "printer_route_mode" => "If several printers on this place",
        "printer_mode_first" => "First that works",
        "printer_mode_all" => "Print on all",
        "kot_show_area" => "Area",
        "kot_hide_area" => "Hide area",
        "kot_show_item_hdr" => "Item · Qty",
        "kot_hide_item_hdr" => "Hide columns",
        "kot_show_title" => "Title line",
        "kot_hide_title" => "Hide title",
        "kot_field_order" => "Print order",
        "kot_field_up" => "Move up",
        "kot_field_down" => "Move down",
        "kot_field_cycle" => "Select next",
        "kot_field_select" => "Select next",
        "kot_field_time" => "Time",
        "kot_field_kot_no" => "KOT No",
        "kot_field_area" => "Area",
        "kot_field_table" => "Table",
        "kot_field_shop" => "Shop name",
        "kot_field_waiter" => "Staff",
        "kot_field_station" => "Station",
        "app_template_blurb_extra" => "Also: Owner Zahir, Admin Asif, Waiter Nawaz; FOH sidebar; history after 4 hours; Today strip off; compact KOT.",
        "dishes_n" => "{n} dishes",
        "recipe_ph" => "Recipe (Rice=0.2, Paneer=0.15)",
        "reorder_ph" => "Buy more at",
        "set_reorder" => "Set reorder level",
        "low_stock_alert" => "Running low — buy soon: {items}",
        "unpriced_note" => "{n} dishes on your menu have no price yet — they bill at 0 until you set one.",
        "show_unpriced" => "Show only dishes without a price",
        "show_all_dishes" => "Show all dishes",
        "role_waiter" => "Waiter",
        "role_kitchen" => "Kitchen",
        "role_title_rest_ph" => "Role title (e.g. Captain)",
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
        // Money-lane chips on the order desk / Settings (template words).
        "lane_declared_label" => "Tax invoice",
        "lane_owner_label" => "Cash memo",
        "money_lane_bill" => "Bill: {lane}",
        _ => key,
    }
    .into()
}

/// Locale string with optional template overlays (concrete feel without forks).
pub fn t_overlay(lang: &str, key: &str, overlays: &[(&str, &str)]) -> String {
    if let Some((_, value)) = overlays.iter().find(|(k, _)| *k == key) {
        return (*value).into();
    }
    t(lang, key)
}

/// Catalogue languages that read another language's table: regional
/// variants share their parent's words, Cantonese reads Traditional
/// Chinese and Goan Konkani reads Konkani (both Devanagari).
pub fn table_language(lang: &str) -> &str {
    match lang {
        "pt-BR" => "pt",
        "es-MX" => "es",
        "yue" => "zh-Hant",
        "gom" => "kok",
        other => other,
    }
}

/// UI string for `key` in `lang`. English (`en` above) is the source; every
/// other catalogue language has its own table, `translations/<code>.tsv`
/// in this crate — one `key<TAB>text` line per `en` key, `{placeholders}`
/// kept exactly. `every_catalogue_language_translates_every_key` fails on
/// any gap, so a new `en` key lands with all of its translations.
pub fn t(lang: &str, key: &str) -> String {
    if let Some(hit) = translated(lang, key) {
        return hit.to_string();
    }
    en(key)
}

fn translated(lang: &str, key: &str) -> Option<&'static str> {
    if lang == "en" {
        return None;
    }
    loaded_table(table_language(lang))?.1.get(key).copied()
}

// build.rs (`bake_translations`) deflates each translations/<code>.tsv into
// OUT_DIR and lists them here as `PACKS`; a table is inflated the first
// time its language is shown, then kept for the life of the process.
mod packs {
    include!(concat!(env!("OUT_DIR"), "/translations.rs"));
}

type Table = std::collections::HashMap<&'static str, &'static str>;

static TABLES: [std::sync::OnceLock<(&'static str, Table)>; packs::PACKS.len()] =
    [const { std::sync::OnceLock::new() }; packs::PACKS.len()];

fn loaded_table(code: &str) -> Option<&'static (&'static str, Table)> {
    let i = packs::PACKS.iter().position(|(c, _)| *c == code)?;
    Some(TABLES[i].get_or_init(|| {
        let text = inflate(packs::PACKS[i].1);
        let table = text
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .filter_map(|line| line.split_once('\t'))
            .collect();
        (text, table)
    }))
}

fn inflate(bytes: &[u8]) -> &'static str {
    use std::io::Read;
    let mut text = String::new();
    // build.rs wrote these bytes; a failure would only mean a corrupt
    // build, and English is a safe answer then (never panic in t()).
    if flate2::read::DeflateDecoder::new(bytes)
        .read_to_string(&mut text)
        .is_err()
    {
        text.clear();
    }
    Box::leak(text.into_boxed_str())
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
        assert_eq!(match_language_tag("nb_NO.UTF-8"), "no");
        assert_eq!(match_language_tag("nn-NO"), "no");
        assert_eq!(match_language_tag("iw_IL"), "he");
        assert_eq!(match_language_tag("in-ID"), "id");
        assert_eq!(match_language_tag("mni-Mtei-IN"), "mni");
        assert_eq!(match_language_tag("sat_IN"), "sat");
        assert_eq!(match_language_tag("yue-HK"), "yue");
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

    /// `en`'s keys, read from this file: the list every table must match.
    fn english_keys() -> Vec<&'static str> {
        let src = include_str!("locale.rs");
        let start = src.find("fn en(key: &str) -> String {").expect("en() in locale.rs");
        let end = start + src[start..].find("        _ => key,").expect("en() fallback arm");
        src[start..end]
            .lines()
            .filter_map(|line| {
                let rest = line.trim_start().strip_prefix('"')?;
                let (key, after) = rest.split_once('"')?;
                after.trim_start().starts_with("=>").then_some(key)
            })
            .collect()
    }

    /// `{n}`, `{name}`, `{}` … in `text`, sorted (order may change in a
    /// translation, the set may not).
    fn placeholders(text: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            let Some(len) = rest[open..].find('}') else { break };
            let inner = &rest[open + 1..open + len];
            if inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                out.push(&rest[open..=open + len]);
            }
            rest = &rest[open + len + 1..];
        }
        out.sort_unstable();
        out
    }

    /// Distinct table codes the catalogue needs (aliases folded).
    fn needed_tables() -> std::collections::BTreeSet<&'static str> {
        LANGUAGES
            .iter()
            .filter(|l| l.code != "en")
            .map(|l| table_language(l.code))
            .collect()
    }

    #[test]
    fn every_catalogue_language_translates_every_key() {
        let keys = english_keys();
        assert!(keys.len() > 500, "en() scan found only {} keys", keys.len());
        let key_set: std::collections::HashSet<&str> = keys.iter().copied().collect();
        // Every gap at once, so one run says everything still to translate.
        let mut gaps = Vec::new();
        for (code, _) in packs::PACKS {
            if !needed_tables().contains(code) {
                gaps.push(format!("translations/{code}.tsv is not a catalogue language"));
            }
        }
        for code in needed_tables() {
            let Some((text, table)) = loaded_table(code) else {
                gaps.push(format!("{code}: no translations/{code}.tsv"));
                continue;
            };
            let mut seen = std::collections::HashSet::new();
            for line in text.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
                match line.split_once('\t') {
                    None => gaps.push(format!("{code}: line without a tab: {line}")),
                    Some((key, _)) if !seen.insert(key) => gaps.push(format!("{code}: {key} twice")),
                    Some((key, _)) if !key_set.contains(key) => {
                        gaps.push(format!("{code}: {key} is not an en() key (stale)"))
                    }
                    _ => {}
                }
            }
            for key in &keys {
                let english = en(key);
                match table.get(key) {
                    None => gaps.push(format!("{code} left {key} untranslated")),
                    Some(value) if value.trim().is_empty() && !english.trim().is_empty() => {
                        gaps.push(format!("{code} {key} is empty"))
                    }
                    Some(value) if value == key => gaps.push(format!("{code} {key} shows the raw key")),
                    Some(value) if placeholders(value) != placeholders(&english) => gaps.push(format!(
                        "{code} {key} placeholders {:?} ≠ English {:?}",
                        placeholders(value),
                        placeholders(&english)
                    )),
                    _ => {}
                }
            }
        }
        assert!(gaps.is_empty(), "{} gaps:\n{}", gaps.len(), gaps.join("\n"));
        // Aliases read their parent's table.
        assert_eq!(t("pt-BR", "cash"), t("pt", "cash"));
        assert_eq!(t("yue", "cash"), t("zh-Hant", "cash"));
        assert_eq!(t("gom", "cash"), t("kok", "cash"));
        assert_ne!(t("hi", "cash"), t("en", "cash"));
    }

    /// Every character of every translation has a glyph in the WorldFonts
    /// chain (ui_script.rs) — a missing script would draw as empty boxes.
    #[test]
    fn every_translation_draws_with_the_shipped_fonts() {
        let script = include_str!("ui_script.rs");
        let start = script.find("let WorldFonts = FontFamily{").expect("WorldFonts in ui_script.rs");
        let end = start + script[start..].find("\n    }").expect("end of WorldFonts");
        let files: Vec<&str> = script[start..end]
            .lines()
            .filter_map(|line| line.split_once("crate_resource(\"self:")?.1.split_once('"').map(|(f, _)| f))
            .collect();
        assert!(files.len() > 20, "WorldFonts scan found only {files:?}");
        let data: Vec<Vec<u8>> = files
            .iter()
            .map(|f| std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(f)).unwrap())
            .collect();
        let faces: Vec<_> = data
            .iter()
            .map(|d| rustybuzz::ttf_parser::Face::parse(d, 0).unwrap())
            .collect();
        let mut missing = std::collections::BTreeMap::<&str, std::collections::BTreeSet<char>>::new();
        for code in needed_tables() {
            let Some((_, table)) = loaded_table(code) else { continue };
            for value in table.values() {
                for c in value.chars() {
                    // Joiners / variation selectors steer shaping; they need no glyph.
                    if c.is_whitespace()
                        || c.is_control()
                        || matches!(c, '\u{200b}'..='\u{200f}' | '\u{2060}' | '\u{fe00}'..='\u{fe0f}')
                    {
                        continue;
                    }
                    if !faces.iter().any(|f| f.glyph_index(c).is_some()) {
                        missing.entry(code).or_default().insert(c);
                    }
                }
            }
        }
        assert!(missing.is_empty(), "characters no shipped font draws: {missing:?}");
    }

    /// An entity-template overlay would show its English in every language;
    /// an app's vertical words belong in its own `en` (translated above).
    #[test]
    fn template_overrides_stay_empty() {
        for tmpl in crate::entity_template::TEMPLATES {
            assert!(
                tmpl.label_overrides.is_empty(),
                "{}: move label_overrides into locale::en so every language translates them",
                tmpl.id
            );
        }
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
