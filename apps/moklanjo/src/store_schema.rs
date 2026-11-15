//! The books schema: every table and index, made in one go on a new file
//! and a no-op on books already open.
//!
//! Moklanjo has not shipped, so there is no upgrade ladder: a new column is a
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
    zone_id TEXT NOT NULL DEFAULT '',
    first_kg TEXT NOT NULL DEFAULT '0',
    first_paise INTEGER NOT NULL DEFAULT 0,
    extra_per_kg_paise INTEGER NOT NULL DEFAULT 0,
    carrier_party_id TEXT NOT NULL DEFAULT '',
    party_id TEXT NOT NULL DEFAULT ''
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
    awb_no TEXT NOT NULL DEFAULT '',
    destination TEXT NOT NULL DEFAULT '',
    consignee_name TEXT NOT NULL DEFAULT '',
    actual_weight_kg TEXT NOT NULL DEFAULT '0',
    volumetric_weight_kg TEXT NOT NULL DEFAULT '0',
    chargeable_weight_kg TEXT NOT NULL DEFAULT '0',
    tracking_status TEXT NOT NULL DEFAULT 'booked',
    zone_id TEXT NOT NULL DEFAULT '',
    cod_paise INTEGER NOT NULL DEFAULT 0,
    carrier_cost_paise INTEGER NOT NULL DEFAULT 0,
    period_bill_id TEXT NOT NULL DEFAULT '',
    handover_kind TEXT NOT NULL DEFAULT '',
    carrier_party_id TEXT NOT NULL DEFAULT '',
    rider_id TEXT NOT NULL DEFAULT '',
    pod_note TEXT NOT NULL DEFAULT '',
    extras_json TEXT NOT NULL DEFAULT '[]',
    declared_value_paise INTEGER NOT NULL DEFAULT 0,
    insurance INTEGER NOT NULL DEFAULT 0,
    contents TEXT NOT NULL DEFAULT '',
    eway_bill_no TEXT NOT NULL DEFAULT '',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    ndr_reason TEXT NOT NULL DEFAULT '',
    next_attempt_at TEXT NOT NULL DEFAULT '',
    origin_name TEXT NOT NULL DEFAULT '',
    origin_phone TEXT NOT NULL DEFAULT '',
    origin_address TEXT NOT NULL DEFAULT '',
    consignee_phone TEXT NOT NULL DEFAULT '',
    consignee_address TEXT NOT NULL DEFAULT '',
    feedback_score INTEGER NOT NULL DEFAULT 0,
    area TEXT NOT NULL DEFAULT '',
    dest_pincode TEXT NOT NULL DEFAULT '',
    piece_count INTEGER NOT NULL DEFAULT 1
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
    unit_label TEXT NOT NULL DEFAULT ''
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
    address TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    branch_id TEXT NOT NULL DEFAULT '',
    kind_label TEXT NOT NULL DEFAULT '',
    custom_fields_json TEXT NOT NULL DEFAULT '[]',
    image_key TEXT NOT NULL DEFAULT '',
    credit_limit_paise INTEGER NOT NULL DEFAULT 0
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
    sort_order INTEGER NOT NULL DEFAULT 0
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
    server_at TEXT NOT NULL DEFAULT ''
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
    server_at TEXT NOT NULL DEFAULT ''
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
    active INTEGER NOT NULL DEFAULT 1
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
    builtin INTEGER NOT NULL DEFAULT 0
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
    table_id TEXT NOT NULL DEFAULT '',
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
CREATE TABLE IF NOT EXISTS zones (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS pincode_service (
    pincode TEXT NOT NULL,
    zone_id TEXT NOT NULL,
    carrier_party_id TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    PRIMARY KEY(pincode, zone_id, carrier_party_id)
);
CREATE TABLE IF NOT EXISTS extra_charge_types (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL DEFAULT 'other',
    name TEXT NOT NULL,
    default_paise INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS riders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    phone TEXT NOT NULL DEFAULT '',
    vehicle_label TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1,
    note TEXT NOT NULL DEFAULT '',
    pay_per_delivery_paise INTEGER NOT NULL DEFAULT 0,
    pay_per_kg_paise INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS manifests (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    kind TEXT NOT NULL DEFAULT 'carrier',
    carrier_party_id TEXT NOT NULL DEFAULT '',
    carrier_name TEXT NOT NULL DEFAULT '',
    rider_id TEXT NOT NULL DEFAULT '',
    rider_name TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open'
);
CREATE TABLE IF NOT EXISTS manifest_lines (
    id TEXT PRIMARY KEY,
    manifest_id TEXT NOT NULL,
    sale_id TEXT NOT NULL,
    awb_no TEXT NOT NULL DEFAULT '',
    destination TEXT NOT NULL DEFAULT '',
    consignee_name TEXT NOT NULL DEFAULT '',
    stop_order INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS tracking_events (
    id TEXT PRIMARY KEY,
    sale_id TEXT NOT NULL,
    awb_no TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    pod_ref TEXT NOT NULL DEFAULT '',
    actor_name TEXT NOT NULL DEFAULT '',
    lat REAL NOT NULL DEFAULT 0,
    lon REAL NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS period_bills (
    id TEXT PRIMARY KEY,
    party_id TEXT NOT NULL,
    party_name TEXT NOT NULL DEFAULT '',
    invoice_no TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    from_date TEXT NOT NULL DEFAULT '',
    to_date TEXT NOT NULL DEFAULT '',
    total_paise INTEGER NOT NULL DEFAULT 0,
    sale_ids_json TEXT NOT NULL DEFAULT '[]',
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS cod_entries (
    id TEXT PRIMARY KEY,
    sale_id TEXT NOT NULL,
    awb_no TEXT NOT NULL DEFAULT '',
    party_id TEXT NOT NULL DEFAULT '',
    party_name TEXT NOT NULL DEFAULT '',
    cod_paise INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    collected_at TEXT NOT NULL DEFAULT '',
    remittance_id TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS cod_remittances (
    id TEXT PRIMARY KEY,
    party_id TEXT NOT NULL,
    party_name TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    user_at TEXT NOT NULL DEFAULT '',
    device_at TEXT NOT NULL DEFAULT '',
    total_paise INTEGER NOT NULL DEFAULT 0,
    pay_method TEXT NOT NULL DEFAULT 'cash',
    note TEXT NOT NULL DEFAULT '',
    entry_ids_json TEXT NOT NULL DEFAULT '[]'
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
CREATE TABLE IF NOT EXISTS incoming_manifests (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    hub_name TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS incoming_manifest_lines (
    id TEXT PRIMARY KEY,
    manifest_id TEXT NOT NULL,
    awb_no TEXT NOT NULL,
    sale_id TEXT NOT NULL DEFAULT '',
    destination TEXT NOT NULL DEFAULT '',
    consignee_name TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS pickup_requests (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    party_id TEXT NOT NULL DEFAULT '',
    party_name TEXT NOT NULL DEFAULT '',
    address TEXT NOT NULL DEFAULT '',
    pincode TEXT NOT NULL DEFAULT '',
    area TEXT NOT NULL DEFAULT '',
    pieces INTEGER NOT NULL DEFAULT 1,
    note TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open',
    run_id TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS pickup_runs (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    rider_id TEXT NOT NULL DEFAULT '',
    rider_name TEXT NOT NULL DEFAULT '',
    drop_note TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'open'
);
CREATE TABLE IF NOT EXISTS assign_rules (
    id TEXT PRIMARY KEY,
    area TEXT NOT NULL DEFAULT '',
    pincode_prefix TEXT NOT NULL DEFAULT '',
    rider_id TEXT NOT NULL,
    rider_name TEXT NOT NULL DEFAULT '',
    round_robin_slot INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS rider_pay_rates (
    rider_id TEXT PRIMARY KEY,
    per_delivery_paise INTEGER NOT NULL DEFAULT 0,
    per_kg_paise INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS vehicle_inspections (
    id TEXT PRIMARY KEY,
    manifest_id TEXT NOT NULL DEFAULT '',
    rider_id TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    lights_ok INTEGER NOT NULL DEFAULT 1,
    tyres_ok INTEGER NOT NULL DEFAULT 1,
    brakes_ok INTEGER NOT NULL DEFAULT 1,
    docs_ok INTEGER NOT NULL DEFAULT 1,
    fuel_note TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS rider_run_cache (
    rider_id TEXT PRIMARY KEY,
    rider_name TEXT NOT NULL DEFAULT '',
    manifest_id TEXT NOT NULL DEFAULT '',
    downloaded_at TEXT NOT NULL,
    stops_json TEXT NOT NULL DEFAULT '[]'
);
CREATE TABLE IF NOT EXISTS franchise_rates (
    carrier_party_id TEXT PRIMARY KEY,
    rate_pct INTEGER NOT NULL DEFAULT 0,
    on_margin INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE IF NOT EXISTS awb_pools (
    id TEXT PRIMARY KEY,
    carrier_party_id TEXT NOT NULL DEFAULT '',
    prefix TEXT NOT NULL DEFAULT '',
    start_n INTEGER NOT NULL,
    end_n INTEGER NOT NULL,
    next_n INTEGER NOT NULL,
    label TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_stock_movements_created ON stock_movements(created_at);
CREATE INDEX IF NOT EXISTS idx_sync_outbox_pending ON sync_outbox(synced_at);
CREATE INDEX IF NOT EXISTS idx_tax_rates_branch ON tax_rates(branch_id);
CREATE INDEX IF NOT EXISTS idx_money_movements_day ON money_movements(created_at);
CREATE INDEX IF NOT EXISTS idx_payable_party ON payable_ledger(party_id);
CREATE INDEX IF NOT EXISTS idx_attach_entity ON entry_attachments(entity_kind, entity_id);
CREATE INDEX IF NOT EXISTS idx_sales_awb ON sales(awb_no);
CREATE INDEX IF NOT EXISTS idx_tracking_sale ON tracking_events(sale_id);
CREATE INDEX IF NOT EXISTS idx_cod_party ON cod_entries(party_id);
CREATE INDEX IF NOT EXISTS idx_manifest_lines ON manifest_lines(manifest_id);
CREATE INDEX IF NOT EXISTS idx_incoming_lines ON incoming_manifest_lines(manifest_id);
CREATE INDEX IF NOT EXISTS idx_pickup_run ON pickup_requests(run_id);
CREATE INDEX IF NOT EXISTS idx_sales_tracking ON sales(tracking_status);
CREATE INDEX IF NOT EXISTS idx_awb_pools_carrier ON awb_pools(carrier_party_id);
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
