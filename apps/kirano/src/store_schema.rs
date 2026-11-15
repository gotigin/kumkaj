//! The books schema: every table and index, made in one go on a new file
//! and a no-op on books already open.
//!
//! Kirano has not shipped, so there is no upgrade ladder: a new column is a
//! change to this text, and books made by an earlier test build are made
//! again. Triggers are added by `install_event_times_trigger`.

pub(super) const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    price_paise INTEGER NOT NULL CHECK(price_paise >= 0),
    available INTEGER NOT NULL DEFAULT 1,
    archived INTEGER NOT NULL DEFAULT 0,
    image_key TEXT,
    stock_qty INTEGER NOT NULL DEFAULT -1,
    category_id TEXT NOT NULL DEFAULT '',
    tax_bps INTEGER NOT NULL DEFAULT 0,
    modifiers_json TEXT NOT NULL DEFAULT '[]',
    aliases_json TEXT NOT NULL DEFAULT '[]',
    branch_id TEXT NOT NULL DEFAULT '',
    tax_rate_id TEXT NOT NULL DEFAULT '',
    unit_label TEXT NOT NULL DEFAULT '',
    group_name TEXT NOT NULL DEFAULT '',
    kind TEXT NOT NULL DEFAULT 'dish',
    schedule_json TEXT NOT NULL DEFAULT '[]',
    barcode TEXT NOT NULL DEFAULT '',
    composition TEXT NOT NULL DEFAULT '',
    mrp_paise INTEGER NOT NULL DEFAULT 0,
    hsn_code TEXT NOT NULL DEFAULT '',
    stock_frac_milli INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS sales (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    total_paise INTEGER NOT NULL,
    pay_method TEXT NOT NULL,
    status TEXT NOT NULL,
    lines_total_paise INTEGER,
    rounding_paise INTEGER NOT NULL DEFAULT 0,
    customer_id TEXT,
    created_by_id TEXT NOT NULL DEFAULT '',
    created_by_name TEXT NOT NULL DEFAULT '',
    branch_id TEXT NOT NULL DEFAULT '',
    tax_paise INTEGER NOT NULL DEFAULT 0,
    tax_pricing TEXT NOT NULL DEFAULT 'inclusive',
    tax_label TEXT NOT NULL DEFAULT 'Tax',
    net_paise INTEGER NOT NULL DEFAULT 0,
    service_paise INTEGER NOT NULL DEFAULT 0,
    invoice_no TEXT NOT NULL DEFAULT '',
    money_lane TEXT NOT NULL DEFAULT 'declared',
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    server_at TEXT NOT NULL DEFAULT '',
    extras_json TEXT NOT NULL DEFAULT '[]',
    tendered_paise INTEGER NOT NULL DEFAULT 0,
    change_paise INTEGER NOT NULL DEFAULT 0,
    tip_paise INTEGER NOT NULL DEFAULT 0,
    tip_method TEXT NOT NULL DEFAULT '',
    discount_paise INTEGER NOT NULL DEFAULT 0,
    discount_reason TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS sale_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sale_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    name TEXT NOT NULL,
    unit_price_paise INTEGER NOT NULL,
    qty INTEGER NOT NULL,
    line_total_paise INTEGER NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    qty_milli INTEGER NOT NULL DEFAULT 0,
    unit_label TEXT NOT NULL DEFAULT '',
    modifiers_json TEXT NOT NULL DEFAULT '[]'
);
CREATE TABLE IF NOT EXISTS sale_payments (
    id TEXT PRIMARY KEY,
    sale_id TEXT NOT NULL,
    pay_method TEXT NOT NULL,
    amount_paise INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS expense_payments (
    id TEXT PRIMARY KEY,
    expense_id TEXT NOT NULL,
    pay_method TEXT NOT NULL,
    amount_paise INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS sale_revisions (
    id TEXT PRIMARY KEY,
    sale_id TEXT NOT NULL,
    revised_at TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    reason TEXT NOT NULL,
    previous_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS staff (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    pin_hash TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 1,
    role_title TEXT NOT NULL DEFAULT '',
    features_json TEXT NOT NULL DEFAULT '',
    login_id TEXT NOT NULL DEFAULT '',
    password_hash TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    action TEXT NOT NULL,
    detail TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    phone TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    branch_id TEXT NOT NULL DEFAULT '',
    kind_label TEXT NOT NULL DEFAULT '',
    credit_limit_paise INTEGER NOT NULL DEFAULT 0,
    loyalty_points INTEGER NOT NULL DEFAULT 0,
    wallet_paise INTEGER NOT NULL DEFAULT 0,
    birthday TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS credit_ledger (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    kind TEXT NOT NULL,
    amount_paise INTEGER NOT NULL,
    sale_id TEXT,
    pay_method TEXT,
    actor_id TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT '',
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    server_at TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS expenses (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    amount_paise INTEGER NOT NULL,
    category TEXT NOT NULL DEFAULT 'misc',
    note TEXT NOT NULL DEFAULT '',
    actor_id TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    branch_id TEXT NOT NULL DEFAULT '',
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    server_at TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS branches (
    id TEXT PRIMARY KEY,
    org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1,
    is_this_device INTEGER NOT NULL DEFAULT 0,
    last_snapshot_json TEXT NOT NULL DEFAULT '',
    last_sync_at TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL DEFAULT '',
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    image_key TEXT NOT NULL DEFAULT '',
    aliases_json TEXT NOT NULL DEFAULT '[]'
);
CREATE TABLE IF NOT EXISTS stock_movements (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL DEFAULT '',
    product_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    delta_qty INTEGER NOT NULL,
    reason TEXT NOT NULL DEFAULT 'adjust',
    note TEXT NOT NULL DEFAULT '',
    actor_id TEXT NOT NULL DEFAULT '',
    actor_name TEXT NOT NULL DEFAULT '',
    ref_id TEXT NOT NULL DEFAULT '',
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    server_at TEXT NOT NULL DEFAULT '',
    delta_milli INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS purchases (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    supplier TEXT NOT NULL DEFAULT '',
    total_paise INTEGER NOT NULL DEFAULT 0,
    note TEXT NOT NULL DEFAULT '',
    actor_id TEXT NOT NULL DEFAULT '',
    actor_name TEXT NOT NULL DEFAULT '',
    party_id TEXT NOT NULL DEFAULT '',
    on_account INTEGER NOT NULL DEFAULT 0,
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    server_at TEXT NOT NULL DEFAULT '',
    batch_code TEXT NOT NULL DEFAULT '',
    expires_on TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS purchase_lines (
    id TEXT PRIMARY KEY,
    purchase_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    name TEXT NOT NULL,
    qty INTEGER NOT NULL,
    unit_cost_paise INTEGER NOT NULL DEFAULT 0,
    qty_milli INTEGER NOT NULL DEFAULT 0,
    unit_label TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS sync_outbox (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '',
    synced_at TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS tax_rates (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL DEFAULT '',
    name TEXT NOT NULL,
    code TEXT NOT NULL DEFAULT '',
    bps INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    picker_enabled INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS money_movements (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    kind TEXT NOT NULL,
    amount_paise INTEGER NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    actor_id TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    branch_id TEXT NOT NULL DEFAULT '',
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    server_at TEXT NOT NULL DEFAULT '',
    cleared_at TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS payable_ledger (
    id TEXT PRIMARY KEY,
    party_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    kind TEXT NOT NULL,
    amount_paise INTEGER NOT NULL,
    purchase_id TEXT,
    pay_method TEXT,
    actor_id TEXT NOT NULL,
    actor_name TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT '',
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    server_at TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS units (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    builtin INTEGER NOT NULL DEFAULT 0,
    picker_enabled INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS unit_conversions (
    id TEXT PRIMARY KEY,
    from_code TEXT NOT NULL,
    to_code TEXT NOT NULL,
    factor_milli INTEGER NOT NULL,
    UNIQUE(from_code, to_code)
);
CREATE TABLE IF NOT EXISTS lan_devices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    token_hash TEXT NOT NULL UNIQUE,
    paired_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL DEFAULT '',
    revoked_at TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS lan_sessions (
    token_hash TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    staff_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS lan_replies (
    request_id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    status INTEGER NOT NULL,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS lan_outbox (
    id TEXT PRIMARY KEY,
    seq INTEGER NOT NULL,
    path TEXT NOT NULL,
    body TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    staff_id TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS lan_refused (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    body TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    error TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    refused_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS schemes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    trigger_json TEXT NOT NULL DEFAULT '{}',
    reward_json TEXT NOT NULL DEFAULT '{}',
    active INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS shift_closes (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    ended_at TEXT NOT NULL,
    closed_by TEXT NOT NULL,
    bills INTEGER NOT NULL,
    cash_paise INTEGER NOT NULL,
    upi_paise INTEGER NOT NULL,
    card_paise INTEGER NOT NULL,
    cash_tips_paise INTEGER NOT NULL,
    cash_out_paise INTEGER NOT NULL,
    float_paise INTEGER NOT NULL,
    expected_paise INTEGER NOT NULL,
    counted_paise INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS notify_outbox (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    channel TEXT NOT NULL DEFAULT '',
    to_addr TEXT NOT NULL DEFAULT '',
    template_key TEXT NOT NULL DEFAULT '',
    payload_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'queued',
    last_error TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS entry_attachments (
    id TEXT PRIMARY KEY,
    entity_kind TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    kind TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    body TEXT NOT NULL DEFAULT '',
    stored_name TEXT NOT NULL DEFAULT '',
    mime TEXT NOT NULL DEFAULT '',
    bytes INTEGER NOT NULL DEFAULT 0,
    actor_id TEXT NOT NULL DEFAULT '',
    actor_name TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS stock_levels (
    product_id TEXT PRIMARY KEY,
    reorder_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS stock_counts (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    closed_at TEXT NOT NULL DEFAULT '',
    closed_by TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS stock_count_lines (
    id TEXT PRIMARY KEY,
    count_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    name TEXT NOT NULL,
    expected_qty INTEGER NOT NULL DEFAULT 0,
    counted_qty INTEGER NOT NULL,
    reason TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS gift_vouchers (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    amount_paise INTEGER NOT NULL,
    balance_paise INTEGER NOT NULL,
    issued_at TEXT NOT NULL,
    redeemed_at TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS time_clock (
    id TEXT PRIMARY KEY,
    staff_id TEXT NOT NULL,
    staff_name TEXT NOT NULL DEFAULT '',
    clock_in TEXT NOT NULL,
    clock_out TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS purchase_orders (
    id TEXT PRIMARY KEY,
    supplier_id TEXT NOT NULL DEFAULT '',
    supplier_name TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT NOT NULL,
    received_at TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS purchase_order_lines (
    id TEXT PRIMARY KEY,
    po_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    name TEXT NOT NULL,
    qty INTEGER NOT NULL,
    unit_cost_paise INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS feedback (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    score INTEGER NOT NULL,
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS connector_outbox (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'queued',
    note TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_stock_movements_created ON stock_movements(created_at);
CREATE INDEX IF NOT EXISTS idx_sync_outbox_pending ON sync_outbox(synced_at);
CREATE INDEX IF NOT EXISTS idx_tax_rates_branch ON tax_rates(branch_id);
CREATE INDEX IF NOT EXISTS idx_money_movements_day ON money_movements(created_at);
CREATE INDEX IF NOT EXISTS idx_payable_party ON payable_ledger(party_id);
CREATE INDEX IF NOT EXISTS idx_attach_entity ON entry_attachments(entity_kind, entity_id);
CREATE INDEX IF NOT EXISTS idx_sale_lines_sale ON sale_lines(sale_id);
CREATE INDEX IF NOT EXISTS idx_sale_payments_sale ON sale_payments(sale_id);
CREATE INDEX IF NOT EXISTS idx_sales_created ON sales(created_at);
CREATE INDEX IF NOT EXISTS idx_sale_revisions_sale ON sale_revisions(sale_id);
CREATE INDEX IF NOT EXISTS idx_credit_ledger_customer ON credit_ledger(customer_id);
CREATE INDEX IF NOT EXISTS idx_credit_ledger_created ON credit_ledger(created_at);
CREATE INDEX IF NOT EXISTS idx_credit_ledger_sale ON credit_ledger(sale_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_events(created_at);
CREATE INDEX IF NOT EXISTS idx_expense_payments_expense ON expense_payments(expense_id);
CREATE INDEX IF NOT EXISTS idx_purchase_lines_purchase ON purchase_lines(purchase_id);
CREATE INDEX IF NOT EXISTS idx_sales_day ON sales(created_at COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_expenses_day ON expenses(created_at COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_money_movements_like ON money_movements(created_at COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_credit_ledger_day ON credit_ledger(created_at COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_payable_ledger_day ON payable_ledger(created_at COLLATE NOCASE);
"#;
