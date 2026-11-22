use uuid::Uuid;

use crate::db::{Account, PlanRow};

use super::config::BillingConfig;
use super::{BillingError, CreatedCheckout};

/// Local sandbox checkout — no money moves. Complete via
/// `POST /v1/billing/sandbox/complete` after the operator "pays".
pub async fn create_checkout(
    cfg: &BillingConfig,
    checkout_id: Uuid,
    account: &Account,
    plan: &PlanRow,
) -> Result<CreatedCheckout, BillingError> {
    let external_id = format!("mock_cs_{checkout_id}");
    let url = format!(
        "{}/account?billing=mock&checkout_id={checkout_id}&plan={}&account={}",
        cfg.success_url
            .split('?')
            .next()
            .unwrap_or("http://127.0.0.1:8011/account"),
        plan.plan_id,
        account.account_id
    );
    // Prefer a stable API-local complete hint in the URL for smoke tests.
    let _ = cfg.public_api_base;
    Ok(CreatedCheckout { external_id, url })
}
