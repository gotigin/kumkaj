//! Combos / BOGO / percent-off schemes (Local).
//!
//! Applied on the cart at settle / bill draft. Fixed combo: parent charged at
//! combo price; components complimentary. Kitchen expand happens on Send KOT.

use crate::domain::{Cart, CartLine, Money};
use serde::{Deserialize, Serialize};

/// `bogo` | `combo` | `percent_items`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemeKind {
    Bogo,
    Combo,
    PercentItems,
}

impl SchemeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bogo => "bogo",
            Self::Combo => "combo",
            Self::PercentItems => "percent_items",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "combo" => Self::Combo,
            "percent_items" => Self::PercentItems,
            _ => Self::Bogo,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scheme {
    pub id: String,
    pub name: String,
    pub kind: SchemeKind,
    pub trigger_json: String,
    pub reward_json: String,
    pub active: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BogoTrigger {
    #[serde(default)]
    pub buy_product_id: String,
    #[serde(default)]
    pub buy_group: String,
    #[serde(default = "one")]
    pub buy_qty: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BogoReward {
    #[serde(default)]
    pub get_product_id: String,
    #[serde(default)]
    pub get_group: String,
    #[serde(default = "one")]
    pub get_qty: i64,
    /// Free (true) or percent off the get items (0–100).
    #[serde(default = "default_free")]
    pub free: bool,
    #[serde(default)]
    pub percent: i64,
}

fn one() -> i64 {
    1
}
fn default_free() -> bool {
    true
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PercentTrigger {
    #[serde(default)]
    pub product_ids: Vec<String>,
    #[serde(default)]
    pub group: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PercentReward {
    /// 0–100.
    #[serde(default)]
    pub percent: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComboComponent {
    #[serde(default)]
    pub product_id: String,
    #[serde(default = "one")]
    pub qty: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComboTrigger {
    #[serde(default)]
    pub parent_product_id: String,
    #[serde(default)]
    pub components: Vec<ComboComponent>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComboReward {
    /// Fixed combo price in paise (0 = keep parent unit price).
    #[serde(default)]
    pub price_paise: i64,
}

impl Scheme {
    pub fn bogo(
        id: impl Into<String>,
        name: impl Into<String>,
        buy_product_id: &str,
        buy_qty: i64,
        get_product_id: &str,
        get_qty: i64,
    ) -> Self {
        let trigger = BogoTrigger {
            buy_product_id: buy_product_id.into(),
            buy_group: String::new(),
            buy_qty: buy_qty.max(1),
        };
        let reward = BogoReward {
            get_product_id: get_product_id.into(),
            get_group: String::new(),
            get_qty: get_qty.max(1),
            free: true,
            percent: 0,
        };
        Self {
            id: id.into(),
            name: name.into(),
            kind: SchemeKind::Bogo,
            trigger_json: serde_json::to_string(&trigger).unwrap_or_else(|_| "{}".into()),
            reward_json: serde_json::to_string(&reward).unwrap_or_else(|_| "{}".into()),
            active: true,
        }
    }

    /// Loose parse: `Buy 2 Thali get 1 Chaas` → name kept, needs product ids
    /// resolved by the caller from the typed names. Returns (buy_name, buy_qty,
    /// get_name, get_qty) when the shape matches.
    pub fn parse_bogo_human(text: &str) -> Option<(String, i64, String, i64)> {
        let t = text.trim();
        let lower = t.to_lowercase();
        let buy_idx = lower.find("buy")?;
        let get_idx = lower.find("get")?;
        if get_idx <= buy_idx {
            return None;
        }
        let buy_part = t[buy_idx + 3..get_idx].trim();
        let get_part = t[get_idx + 3..].trim();
        let (buy_qty, buy_name) = split_qty_name(buy_part)?;
        let (get_qty, get_name) = split_qty_name(get_part)?;
        if buy_name.is_empty() || get_name.is_empty() {
            return None;
        }
        Some((buy_name, buy_qty, get_name, get_qty))
    }

    pub fn fixed_combo(
        id: impl Into<String>,
        name: impl Into<String>,
        parent_product_id: &str,
        components: &[(&str, i64)],
        price_paise: i64,
    ) -> Self {
        let trigger = ComboTrigger {
            parent_product_id: parent_product_id.into(),
            components: components
                .iter()
                .map(|(pid, qty)| ComboComponent {
                    product_id: (*pid).into(),
                    qty: (*qty).max(1),
                })
                .collect(),
        };
        let reward = ComboReward { price_paise };
        Self {
            id: id.into(),
            name: name.into(),
            kind: SchemeKind::Combo,
            trigger_json: serde_json::to_string(&trigger).unwrap_or_else(|_| "{}".into()),
            reward_json: serde_json::to_string(&reward).unwrap_or_else(|_| "{}".into()),
            active: true,
        }
    }
}

fn split_qty_name(part: &str) -> Option<(i64, String)> {
    let part = part.trim();
    let mut bits = part.splitn(2, char::is_whitespace);
    let first = bits.next()?.trim();
    let rest = bits.next().unwrap_or("").trim();
    if let Ok(n) = first.parse::<i64>() {
        if n > 0 && !rest.is_empty() {
            return Some((n, rest.to_string()));
        }
    }
    // "Thali" alone → qty 1
    if !part.is_empty() {
        return Some((1, part.to_string()));
    }
    None
}

fn line_matches(line: &CartLine, product_id: &str, group: &str, products_group: &dyn Fn(&str) -> String) -> bool {
    if !product_id.is_empty() && line.product_id == product_id {
        return true;
    }
    if !group.is_empty() {
        let g = products_group(&line.product_id);
        if !g.is_empty() && g.eq_ignore_ascii_case(group.trim()) {
            return true;
        }
        if line.name.eq_ignore_ascii_case(group.trim()) {
            return true;
        }
    }
    false
}

/// Mark cheapest matching reward lines complimentary / percent-off when the
/// trigger qty is met. `group_of` resolves a product id → dish group label
/// (empty when unknown). Returns scheme names that fired.
pub fn apply_schemes(
    cart: &mut Cart,
    schemes: &[Scheme],
    group_of: &dyn Fn(&str) -> String,
) -> Vec<String> {
    let mut fired = Vec::new();
    // Clear previous auto-scheme marks so re-apply is idempotent: only lines
    // that were complimentary *because* of a scheme note get reset. Manual NC
    // (running_line complimentary from store) keeps its flag via note empty /
    // non-scheme. We key off a sentinel prefix in the note.
    const MARK: &str = "Scheme: ";
    for line in cart.lines.iter_mut() {
        if line.note.starts_with(MARK) {
            line.complimentary = false;
            line.line_total_override = None;
            line.note.clear();
        }
    }
    for scheme in schemes.iter().filter(|s| s.active) {
        match scheme.kind {
            SchemeKind::Bogo => {
                if apply_bogo(cart, scheme, group_of) {
                    fired.push(scheme.name.clone());
                }
            }
            SchemeKind::PercentItems => {
                if apply_percent(cart, scheme, group_of) {
                    fired.push(scheme.name.clone());
                }
            }
            SchemeKind::Combo => {
                if apply_combo(cart, scheme) {
                    fired.push(scheme.name.clone());
                }
            }
        }
    }
    cart.scheme_notes = fired.clone();
    if !fired.is_empty() {
        cart.charge_total = None;
    }
    fired
}

fn apply_combo(cart: &mut Cart, scheme: &Scheme) -> bool {
    let trigger: ComboTrigger = serde_json::from_str(&scheme.trigger_json).unwrap_or_default();
    let reward: ComboReward = serde_json::from_str(&scheme.reward_json).unwrap_or_default();
    if trigger.parent_product_id.is_empty() {
        return false;
    }
    let parents: Vec<usize> = cart
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.product_id == trigger.parent_product_id && !l.complimentary)
        .map(|(i, _)| i)
        .collect();
    if parents.is_empty() {
        return false;
    }
    let mark = format!("Scheme: {}", scheme.name);
    let mut any = false;
    for i in parents {
        if reward.price_paise > 0 {
            let q = cart.lines[i].qty.max(1);
            cart.lines[i].unit_price = Money(reward.price_paise);
            cart.lines[i].line_total_override = Some(Money(reward.price_paise.saturating_mul(q)));
            cart.lines[i].note = mark.clone();
            any = true;
        }
    }
    for comp in &trigger.components {
        if comp.product_id.is_empty() {
            continue;
        }
        for line in cart.lines.iter_mut() {
            if line.product_id == comp.product_id && !line.complimentary {
                line.complimentary = true;
                line.line_total_override = None;
                line.note = mark.clone();
                any = true;
            }
        }
    }
    any
}

fn apply_bogo(cart: &mut Cart, scheme: &Scheme, group_of: &dyn Fn(&str) -> String) -> bool {
    let trigger: BogoTrigger = serde_json::from_str(&scheme.trigger_json).unwrap_or_default();
    let reward: BogoReward = serde_json::from_str(&scheme.reward_json).unwrap_or_default();
    let buy_qty = trigger.buy_qty.max(1);
    let get_qty = reward.get_qty.max(1);
    let buy_units: i64 = cart
        .lines
        .iter()
        .filter(|l| {
            !l.complimentary
                && line_matches(l, &trigger.buy_product_id, &trigger.buy_group, group_of)
        })
        .map(|l| l.qty.max(1))
        .sum();
    let sets = buy_units / buy_qty;
    if sets <= 0 {
        return false;
    }
    let mut free_left = sets * get_qty;
    // Cheapest matching get lines first.
    let mut idxs: Vec<usize> = cart
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            !l.complimentary
                && line_matches(l, &reward.get_product_id, &reward.get_group, group_of)
        })
        .map(|(i, _)| i)
        .collect();
    idxs.sort_by_key(|&i| cart.lines[i].unit_price.paise());
    let mark = format!("Scheme: {}", scheme.name);
    let mut any = false;
    for i in idxs {
        if free_left <= 0 {
            break;
        }
        let line = &mut cart.lines[i];
        let q = line.qty.max(1);
        if reward.free {
            if q <= free_left {
                line.complimentary = true;
                line.line_total_override = None;
                line.note = mark.clone();
                free_left -= q;
                any = true;
            } else {
                // Partial: charge for the remainder.
                let charged = q - free_left;
                let total = Money(line.unit_price.paise().saturating_mul(charged));
                line.line_total_override = Some(total);
                line.note = mark.clone();
                free_left = 0;
                any = true;
            }
        } else if reward.percent > 0 {
            let pct = reward.percent.min(100).max(0);
            let take = q.min(free_left);
            let full = line.actual_total().paise();
            let per = if q > 0 { full / q } else { 0 };
            let discount = per.saturating_mul(take).saturating_mul(pct) / 100;
            line.line_total_override = Some(Money((full - discount).max(0)));
            line.note = mark.clone();
            free_left -= take;
            any = true;
        }
    }
    any
}

fn apply_percent(cart: &mut Cart, scheme: &Scheme, group_of: &dyn Fn(&str) -> String) -> bool {
    let trigger: PercentTrigger = serde_json::from_str(&scheme.trigger_json).unwrap_or_default();
    let reward: PercentReward = serde_json::from_str(&scheme.reward_json).unwrap_or_default();
    let pct = reward.percent.min(100).max(0);
    if pct <= 0 {
        return false;
    }
    let mark = format!("Scheme: {}", scheme.name);
    let mut any = false;
    for line in cart.lines.iter_mut() {
        if line.complimentary {
            continue;
        }
        let hit = trigger.product_ids.iter().any(|id| id == &line.product_id)
            || (!trigger.group.is_empty()
                && line_matches(line, "", &trigger.group, group_of));
        if !hit {
            continue;
        }
        let full = line.actual_total().paise();
        let discount = full.saturating_mul(pct) / 100;
        line.line_total_override = Some(Money((full - discount).max(0)));
        line.note = mark.clone();
        any = true;
    }
    any
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Product;

    #[test]
    fn bogo_makes_one_line_free_when_qty_matches() {
        let thali = Product::new_basic("thali", "Thali", Money::from_rupees(200));
        let chaas = Product::new_basic("chaas", "Chaas", Money::from_rupees(30));
        let mut cart = Cart::default();
        cart.add_or_bump(&thali);
        cart.add_or_bump(&thali);
        cart.add_or_bump(&chaas);
        let scheme = Scheme::bogo("s1", "Buy 2 Thali get 1 Chaas", "thali", 2, "chaas", 1);
        let fired = apply_schemes(&mut cart, &[scheme], &|_| String::new());
        assert_eq!(fired, vec!["Buy 2 Thali get 1 Chaas"]);
        let chaas_line = cart.lines.iter().find(|l| l.product_id == "chaas").unwrap();
        assert!(chaas_line.complimentary);
        assert_eq!(chaas_line.applied_total().paise(), 0);
        assert_eq!(cart.applied_total().paise(), 40_000); // 2 × 200
    }

    #[test]
    fn parse_bogo_human_buy_get() {
        let (b, bq, g, gq) = Scheme::parse_bogo_human("Buy 2 Thali get 1 Chaas").unwrap();
        assert_eq!((b.as_str(), bq, g.as_str(), gq), ("Thali", 2, "Chaas", 1));
    }
}
