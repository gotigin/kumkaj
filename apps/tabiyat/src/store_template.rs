//! App template packs: menu + optional `shop_defaults.json` (Kirano copy).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::Store;
use crate::domain::{StaffFeatures, StaffMember, StaffRole};
use crate::media;
use crate::menu_templates;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShopDefaultsFile {
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub format: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub app: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shop_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_login_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub staff: Vec<ShopDefaultsStaff>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deactivate_tax_rates: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub clear_product_tax: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_charge_bps: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_day_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_retention: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_retention_hours: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_today_bar: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_tax_chrome: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_tax_invoice: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pay_methods: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav_hidden_csv: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub prefs: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_rates: Option<Vec<TaxRateSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<Vec<UnitSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_conversions: Option<Vec<ConversionSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_qr_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stamp_base64: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShopDefaultsStaff {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub role_title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub login_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub features: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access: Option<StaffFeatures>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxRateSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub code: String,
    pub bps: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitSpec {
    pub code: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionSpec {
    pub from: String,
    pub to: String,
    pub factor_milli: i64,
}

impl ShopDefaultsFile {
    pub fn read(path: &Path) -> Result<Self, String> {
        let raw =
            fs::read_to_string(path).map_err(|e| format!("Read {}: {e}", path.display()))?;
        let d: Self =
            serde_json::from_str(&raw).map_err(|e| format!("shop_defaults.json: {e}"))?;
        menu_templates::check_pack_header(d.format, &d.app)?;
        Ok(d)
    }
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}
fn is_false(v: &bool) -> bool {
    !*v
}

impl Store {
    pub fn app_template_offer_done(&self) -> Result<bool, String> {
        Ok(self
            .meta_get("app_template_offer_done")?
            .as_deref()
            == Some("1"))
    }

    pub fn set_app_template_offer_done(&self) -> Result<(), String> {
        self.meta_set("app_template_offer_done", "1")
    }

  /// Apply menu + optional shop settings from a pack folder.
    pub fn apply_app_template_pack(
        &self,
        pack_dir: &Path,
        apply_shop_defaults: bool,
    ) -> Result<super::CatalogueImport, String> {
        self.assert_server_writer("Apply app template")?;
        let tmpl = menu_templates::read_from_dir(pack_dir)
            .map_err(|e| format!("No usable template.json in {}: {e}", pack_dir.display()))?;
        if apply_shop_defaults {
            let path = pack_dir.join("shop_defaults.json");
            if path.is_file() {
                let d = ShopDefaultsFile::read(&path)?;
                self.apply_pack_catalogue_setup(&d)?;
                let summary = self.import_menu_template_from_dir(&tmpl, pack_dir)?;
                self.apply_pack_shop_defaults(&d)?;
                return Ok(summary);
            }
        }
        self.import_menu_template_from_dir(&tmpl, pack_dir)
    }

    fn template_actor(&self) -> Result<StaffMember, String> {
        self.list_staff()?
            .into_iter()
            .find(|s| s.is_owner())
            .ok_or_else(|| "Owner account missing".to_string())
    }

    fn apply_pack_catalogue_setup(&self, d: &ShopDefaultsFile) -> Result<(), String> {
        if d.deactivate_tax_rates {
            self.conn
                .execute("UPDATE tax_rates SET active = 0, picker_enabled = 0", [])
                .map_err(|e| e.to_string())?;
        }
        if let Some(specs) = &d.tax_rates {
            for spec in specs {
                let name = spec.name.trim();
                if name.is_empty() {
                    continue;
                }
                let existing = self.list_tax_rates()?.into_iter().find(|r| r.name == name);
                if existing.is_none() {
                    let _ = self.add_tax_rate(name, spec.code.trim(), spec.bps)?;
                }
            }
        }
        if let Some(specs) = &d.units {
            for spec in specs {
                let code = spec.code.trim();
                if code.is_empty() {
                    continue;
                }
                let _ = self.add_unit(code, spec.name.trim())?;
            }
        }
        if let Some(specs) = &d.unit_conversions {
            for spec in specs {
                let _ = self.add_unit_conversion(
                    spec.from.trim(),
                    spec.to.trim(),
                    spec.factor_milli,
                )?;
            }
        }
        Ok(())
    }

    fn apply_pack_shop_defaults(&self, d: &ShopDefaultsFile) -> Result<(), String> {
        if let Some(name) = d.shop_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            self.set_venue_title(name)?;
        }
        if let Some(logo) = d.logo_base64.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let bytes = media::decode_base64(logo).map_err(|e| format!("Shop logo: {e}"))?;
            let tmp = std::env::temp_dir().join(format!(
                "tabiyat_pack_logo_{}.bin",
                std::process::id()
            ));
            fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
            let _ = self.import_venue_icon(&tmp);
            let _ = fs::remove_file(&tmp);
        }
        if d.clear_product_tax {
            self.conn
                .execute("UPDATE products SET tax_rate_id = '', tax_bps = 0", [])
                .map_err(|e| e.to_string())?;
        }
        if let Some(bps) = d.service_charge_bps {
            self.set_service_charge_bps(bps)?;
        }
        let owner = self.template_actor()?;
        if let (Some(name), Some(login)) = (
            d.owner_name.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            d.owner_login_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty()),
        ) {
            self.assert_login_id_free(login, Some(&owner.id))?;
            self.conn
                .execute(
                    "UPDATE staff SET name = ?1, login_id = ?2 WHERE id = ?3 AND role = 'owner' AND active = 1",
                    rusqlite::params![name, login, owner.id],
                )
                .map_err(|e| e.to_string())?;
        } else if let Some(name) = d.owner_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            self.conn
                .execute(
                    "UPDATE staff SET name = ?1 WHERE id = ?2 AND role = 'owner' AND active = 1",
                    rusqlite::params![name, owner.id],
                )
                .map_err(|e| e.to_string())?;
        }
        let owner = self.template_actor()?;
        let existing = self.list_staff()?;
        for hire in &d.staff {
            let role = StaffRole::parse(&hire.role)
                .ok_or_else(|| format!("Unknown staff role '{}'", hire.role))?;
            let features = hire
                .access
                .clone()
                .unwrap_or_else(|| match hire.features.trim() {
                    "all" => StaffFeatures::all(),
                    _ => StaffFeatures::for_role(role),
                });
            let title = if hire.role_title.trim().is_empty() {
                role.default_title()
            } else {
                hire.role_title.trim()
            };
            let same = |s: &StaffMember| {
                s.active
                    && ((!hire.login_id.trim().is_empty()
                        && s.login_id.eq_ignore_ascii_case(hire.login_id.trim()))
                        || s.name.eq_ignore_ascii_case(hire.name.trim()))
            };
            if existing.iter().any(same) {
                continue;
            }
            self.add_staff_with_features(
                &super::StaffHire {
                    name: hire.name.trim(),
                    role,
                    role_title: title,
                    pin: "",
                    login_id: hire.login_id.trim(),
                    password: "",
                    features,
                },
                &owner,
            )?;
        }
        for (key, value) in &d.prefs {
            if super::SHOP_PREF_KEYS.contains(&key.as_str())
                || menu_templates::template_pref_extra_keys().contains(&key.as_str())
            {
                self.meta_set(key, value)?;
            }
        }
        if let Some(mode) = d
            .business_day_mode
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            self.meta_set("business_day_mode", mode)?;
        }
        if let Some(v) = d.history_retention.as_deref() {
            self.meta_set("history_retention", v)?;
        }
        if let Some(v) = d.history_retention_hours.as_deref() {
            self.meta_set("history_retention_hours", v)?;
        }
        if let Some(v) = d.show_today_bar {
            self.meta_set("show_today_bar", if v { "1" } else { "0" })?;
        }
        if let Some(v) = d.show_tax_chrome {
            self.meta_set("show_tax_chrome", if v { "1" } else { "0" })?;
        }
        if let Some(v) = d.show_tax_invoice {
            self.meta_set("show_tax_invoice", if v { "1" } else { "0" })?;
        }
        if let Some(v) = d.pay_methods.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            self.meta_set("pay_methods", v)?;
        }
        if let Some(v) = d.nav_hidden_csv.as_deref() {
            self.set_nav_hidden_csv(v)?;
        }
        Ok(())
    }
}