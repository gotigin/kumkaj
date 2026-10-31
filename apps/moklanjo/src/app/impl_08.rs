impl App {
fn apply_cart_note(&mut self, cx: &mut Cx, index: usize, text: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.cart.set_note(index, text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn clear_menu_form(&mut self, cx: &mut Cx) {
        for path in [
            ids!(menu_name_input),
            ids!(menu_aliases_input),
            ids!(menu_price_input),
            ids!(menu_unit_input),
            ids!(menu_tax_input),
            ids!(menu_category_input),
            ids!(menu_modifiers_input),
            ids!(menu_first_kg_input),
            ids!(menu_first_amt_input),
            ids!(menu_extra_kg_input),
            ids!(menu_zone_input),
            ids!(menu_carrier_input),
            ids!(menu_party_input),
        ] {
            self.ui.text_input(cx, path).set_text(cx, "");
        }
        let mut st = app_state().lock().unwrap();
        st.editing_product_id = None;
        st.draft_image_key = None;
    }

    /// Leave the item currently in the form and start a blank one.
    fn start_new_menu_item(&mut self, cx: &mut Cx) {
        self.clear_menu_form(cx);
        self.ui
            .view(cx, ids!(menu_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.sync_chrome(cx);
    }

    /// Tap loads that item into the form. A different item replaces it.
    /// The same item stays as it is, so a second tap does not wipe unsaved edits.
    /// New item is the way back to adding.
    fn open_menu_item(&mut self, cx: &mut Cx, index: usize) {
        let loaded = {
            let st = app_state().lock().unwrap();
            let Some(product) = st.products.get(index).cloned() else {
                return;
            };
            if st.editing_product_id.as_deref() == Some(product.id.as_str()) {
                return;
            } else {
                let currency = st.locale.currency.clone();
                let store = st.store.as_ref();
                let tax = store
                    .map(|s| menu_tax_text(s, &product))
                    .unwrap_or_default();
                let category = store
                    .and_then(|s| s.list_categories().ok())
                    .map(|list| {
                        name_for_id(
                            list.iter().map(|row| (row.id.as_str(), row.name.as_str())),
                            &product.category_id,
                        )
                    })
                    .unwrap_or_default();
                let aliases = product.aliases().join(", ");
                let modifiers = modifiers_as_typed(&product.modifiers(), &currency);
                let price = locale::format_major_input(product.price.paise(), &currency);
                Some((product, tax, category, aliases, modifiers, price))
            }
        };
        let Some((product, tax, category, aliases, modifiers, price)) = loaded else {
            return;
        };
        self.ui
            .text_input(cx, ids!(menu_name_input))
            .set_text(cx, &product.name);
        self.ui
            .text_input(cx, ids!(menu_aliases_input))
            .set_text(cx, &aliases);
        self.ui
            .text_input(cx, ids!(menu_price_input))
            .set_text(cx, &price);
        self.ui
            .text_input(cx, ids!(menu_unit_input))
            .set_text(cx, &product.unit_label);
        self.ui
            .text_input(cx, ids!(menu_tax_input))
            .set_text(cx, &tax);
        self.ui
            .text_input(cx, ids!(menu_category_input))
            .set_text(cx, &category);
        self.ui
            .text_input(cx, ids!(menu_modifiers_input))
            .set_text(cx, &modifiers);
        let currency = {
            let st = app_state().lock().unwrap();
            st.locale.currency.clone()
        };
        let first_kg = if product.first_kg > precise::ZERO {
            precise::format_kg(product.first_kg)
        } else {
            String::new()
        };
        let first_amt = if product.first_paise > 0 {
            locale::format_major_input(product.first_paise, &currency)
        } else {
            String::new()
        };
        let extra_kg = if product.extra_per_kg_paise > 0 {
            locale::format_major_input(product.extra_per_kg_paise, &currency)
        } else {
            String::new()
        };
        let (zone_name, carrier_name, party_name) = {
            let st = app_state().lock().unwrap();
            let zones = st
                .store
                .as_ref()
                .and_then(|s| s.list_zones().ok())
                .unwrap_or_default();
            let parties = st
                .store
                .as_ref()
                .and_then(|s| s.list_customers().ok())
                .unwrap_or_default();
            let zone_name = zones
                .into_iter()
                .find(|z| z.id == product.zone_id)
                .map(|z| z.name)
                .unwrap_or_default();
            let carrier_name = parties
                .iter()
                .find(|p| p.id == product.carrier_party_id)
                .map(|p| p.name.clone())
                .unwrap_or_default();
            let party_name = parties
                .iter()
                .find(|p| p.id == product.party_id)
                .map(|p| p.name.clone())
                .unwrap_or_default();
            (zone_name, carrier_name, party_name)
        };
        self.ui
            .text_input(cx, ids!(menu_first_kg_input))
            .set_text(cx, &first_kg);
        self.ui
            .text_input(cx, ids!(menu_first_amt_input))
            .set_text(cx, &first_amt);
        self.ui
            .text_input(cx, ids!(menu_extra_kg_input))
            .set_text(cx, &extra_kg);
        self.ui
            .text_input(cx, ids!(menu_zone_input))
            .set_text(cx, &zone_name);
        self.ui
            .text_input(cx, ids!(menu_carrier_input))
            .set_text(cx, &carrier_name);
        self.ui
            .text_input(cx, ids!(menu_party_input))
            .set_text(cx, &party_name);
        {
            let mut st = app_state().lock().unwrap();
            st.editing_product_id = Some(product.id.clone());
            st.draft_image_key = product.image_key.clone();
        }
        self.ui
            .view(cx, ids!(menu_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.sync_chrome(cx);
    }

    fn add_menu_item(&mut self, cx: &mut Cx) {
        let name = self.ui.text_input(cx, ids!(menu_name_input)).text();
        let aliases_text = self.ui.text_input(cx, ids!(menu_aliases_input)).text();
        let price_text = self.ui.text_input(cx, ids!(menu_price_input)).text();
        let unit_text = self.ui.text_input(cx, ids!(menu_unit_input)).text();
        let tax_text = self.ui.text_input(cx, ids!(menu_tax_input)).text();
        let category_text = self.ui.text_input(cx, ids!(menu_category_input)).text();
        let modifiers_text = self.ui.text_input(cx, ids!(menu_modifiers_input)).text();
        let first_kg_text = self.ui.text_input(cx, ids!(menu_first_kg_input)).text();
        let first_amt_text = self.ui.text_input(cx, ids!(menu_first_amt_input)).text();
        let extra_kg_text = self.ui.text_input(cx, ids!(menu_extra_kg_input)).text();
        let zone_text = self.ui.text_input(cx, ids!(menu_zone_input)).text();
        let carrier_text = self.ui.text_input(cx, ids!(menu_carrier_input)).text();
        let party_text = self.ui.text_input(cx, ids!(menu_party_input)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let updating = st.editing_product_id.is_some();
            st.require_server_role(if updating {
                "Change items"
            } else {
                "Add items"
            })?;
            let actor = st.actor()?;
            if !actor.role.can_manage_items() {
                return Err("Not allowed to manage items".into());
            }
            let name = name.trim();
            if name.is_empty() {
                return Err("Name required".into());
            }
            let currency = st.locale.currency.clone();
            let price_text = if price_text.trim().is_empty() {
                "0"
            } else {
                price_text.trim()
            };
            let price_minor = locale::parse_major_input(price_text, &currency)?;
            if price_minor < 0 {
                return Err("Price cannot be negative".into());
            }
            let mut image_key = st.draft_image_key.clone();
            let editing_id = st.editing_product_id.clone();
            // Auto: leave None so display always re-detects from name.
            // Built-in / custom: persist chosen key.
            let product_id = editing_id.clone().unwrap_or_else(new_id);
            if let Some(key) = image_key.clone()
                && key.starts_with("pending_custom:")
            {
                let src = std::path::PathBuf::from(&key["pending_custom:".len()..]);
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                image_key = Some(store.import_item_image(&product_id, &src)?);
            }
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            // Sellers type either a rate from their country pack ("Standard",
            // "GST 18%") or a bare percent, which becomes a named rate.
            let tax_rate = store.resolve_tax_rate(&tax_text)?;
            let category_id = match category_text.trim() {
                "" => String::new(),
                wanted => match store
                    .list_categories()?
                    .into_iter()
                    .find(|c| c.name.eq_ignore_ascii_case(wanted))
                {
                    Some(c) => c.id,
                    None => store.add_category(wanted)?.id,
                },
            };
            let modifiers = domain::ModifierOption::parse_human_list(&modifiers_text, &currency)?;
            let mut product = if let Some(id) = editing_id {
                store
                    .get_product(&id)?
                    .ok_or_else(|| "That item is no longer in Items".to_string())?
            } else {
                Product::new_basic(product_id, name, Money(price_minor))
            };
            product.name = name.to_string();
            product.price = Money(price_minor);
            product.unit_label = unit_text.trim().to_string();
            product.image_key = image_key;
            product.set_aliases(&typed_aliases(&aliases_text));
            if let Some(rate) = &tax_rate {
                product.tax_rate_id = rate.id.clone();
                product.tax_bps = rate.bps;
            } else {
                product.tax_rate_id.clear();
                product.tax_bps = 0;
            }
            product.category_id = category_id;
            product.set_modifiers(&modifiers);
            product.first_kg = booking::parse_positive(&first_kg_text).unwrap_or(precise::ZERO);
            product.first_paise =
                locale::parse_major_input(&first_amt_text, &currency).unwrap_or(0);
            product.extra_per_kg_paise =
                locale::parse_major_input(&extra_kg_text, &currency).unwrap_or(0);
            let zone_name = zone_text.trim();
            if !zone_name.is_empty() {
                let zones = store.list_zones()?;
                let zone = if let Some(z) = zones
                    .iter()
                    .find(|z| z.name.eq_ignore_ascii_case(zone_name))
                {
                    z.clone()
                } else {
                    store.add_zone(zone_name)?
                };
                product.zone_id = zone.id;
            } else {
                product.zone_id.clear();
            }
            let parties = store.list_customers()?;
            let resolve_party = |typed: &str| -> Result<String, String> {
                let typed = typed.trim();
                if typed.is_empty() {
                    return Ok(String::new());
                }
                if let Some(p) = parties
                    .iter()
                    .find(|p| p.name.eq_ignore_ascii_case(typed) || p.id == typed)
                {
                    return Ok(p.id.clone());
                }
                Err(format!("No party named “{typed}” — add them on Parties first"))
            };
            product.carrier_party_id = resolve_party(&carrier_text)?;
            product.party_id = resolve_party(&party_text)?;
            store.upsert_product(&product)?;
            let verb = if updating { "menu.update" } else { "menu.add" };
            // A price change says from what to what: who changed a price is
            // the question the owner asks of the audit (vision §18).
            let was = st.products.iter().find(|p| p.id == product.id).map(|p| p.price);
            let detail = match was {
                Some(old) if old != product.price => format!(
                    "{} · {} → {}",
                    product.name,
                    old.format_localized(&st.locale.currency, &st.fiscal),
                    product.price.format_localized(&st.locale.currency, &st.fiscal)
                ),
                _ => product.name.clone(),
            };
            store.audit(&actor, verb, &detail)?;
            st.draft_image_key = None;
            st.editing_product_id = None;
            st.reload_all()?;
            st.status = if updating {
                format!("Updated {}", product.name)
            } else {
                format!("Added {}", product.name)
            };
            Ok(())
        })();
        match result {
            Ok(()) => self.clear_menu_form(cx),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    fn remove_menu_item(&mut self, cx: &mut Cx, index: usize) {
        let was_editing = {
            let st = app_state().lock().unwrap();
            st.products.get(index).is_some_and(|product| {
                st.editing_product_id.as_deref() == Some(product.id.as_str())
            })
        };
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.require_server_role("Remove items")?;
            let actor = st.actor()?;
            if !actor.role.can_manage_items() {
                return Err("Not allowed to manage items".into());
            }
            let product = st
                .products
                .get(index)
                .cloned()
                .ok_or_else(|| "Missing product".to_string())?;
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.archive_product(&product.id)?;
            store.audit(&actor, "menu.remove", &product.name)?;
            if st.editing_product_id.as_deref() == Some(product.id.as_str()) {
                st.editing_product_id = None;
                st.draft_image_key = None;
            }
            st.cart.lines.retain(|l| l.product_id != product.id);
            st.reload_all()?;
            st.status = format!("Removed {}", product.name);
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        } else if was_editing {
            self.clear_menu_form(cx);
        }
        self.sync_chrome(cx);
    }

    fn set_language(&mut self, cx: &mut Cx, code: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.language_follow_platform = false;
            st.locale.language = code.into();
            // On DeviceSetup, preview immediately; persist when Server/Client is chosen.
            // After unlock (Settings), persist as the fixed user language.
            if !st.must_confirm_device_role {
                if let Some(store) = st.store.as_ref() {
                    st.locale = store.set_user_language(code)?;
                }
                st.save_remembered_language();
            }
            st.status = format!("{}: {code}", st.t("language"));
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn set_currency(&mut self, cx: &mut Cx, code: &str) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.locale.currency = code.into();
            if let Some(store) = st.store.as_ref() {
                store.set_locale_settings(&st.locale)?;
            }
            st.status = format!("Currency: {code}");
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
        self.sync_chrome(cx);
    }

    fn image_clear_photo(&mut self, cx: &mut Cx) {
        app_state().lock().unwrap().draft_image_key = None;
        self.sync_chrome(cx);
    }

    fn image_pick_file(&mut self, cx: &mut Cx) {
        // Native picker on every OS (Android SAF included). Do not use rfd —
        // it is desktop-only and left Android with a stub message.
        let dialog = FileDialog::new()
            .set_title("Choose rate photo".into())
            .add_filter(
                "Images".into(),
                vec!["webp".into(), "png".into(), "jpg".into(), "jpeg".into()],
            )
            .set_id(live_id!(item_photo_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening photo picker…");
    }

    fn attach_pick_image(&mut self, cx: &mut Cx, entity: AttachmentEntity) {
        app_state().lock().unwrap().attach_pick = Some(AttachPickTarget::Image(entity));
        let dialog = FileDialog::new()
            .set_title("Attach photo".into())
            .add_filter(
                "Images".into(),
                vec!["webp".into(), "png".into(), "jpg".into(), "jpeg".into()],
            )
            .set_id(live_id!(entry_attach_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening photo picker…");
    }

    fn attach_pick_pdf(&mut self, cx: &mut Cx, entity: AttachmentEntity) {
        app_state().lock().unwrap().attach_pick = Some(AttachPickTarget::File(entity));
        let dialog = FileDialog::new()
            .set_title("Attach PDF".into())
            .add_filter("Documents".into(), vec!["pdf".into(), "txt".into()])
            .set_id(live_id!(entry_attach_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening file picker…");
    }

    /// Resolve typed/scanned AWB into the ops desk card (+ reports pane for copy).
    fn resolve_ops_awb_card(&mut self, cx: &mut Cx, awb: &str) -> Result<String, String> {
        let card = {
            let st = app_state().lock().unwrap();
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            courier_ops_view::awb_desk_card(store, awb)?
        };
        self.ui
            .widget(cx, ids!(ops_awb_card))
            .set_text(cx, &plain_display_text(&card));
        app_state().lock().unwrap().day_close_text = card.clone();
        Ok(card)
    }

    fn attach_pick_ops_pod(&mut self, cx: &mut Cx) {
        app_state().lock().unwrap().attach_pick = Some(AttachPickTarget::OpsPod);
        let dialog = FileDialog::new()
            .set_title("POD photo".into())
            .add_filter(
                "Images".into(),
                vec!["webp".into(), "png".into(), "jpg".into(), "jpeg".into()],
            )
            .set_id(live_id!(entry_attach_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening POD picker…");
    }

    fn entity_id_for_attach(&self, entity: AttachmentEntity) -> Result<String, String> {
        let st = app_state().lock().unwrap();
        let id = match entity {
            AttachmentEntity::Sale => st.selected_sale_id.clone(),
            AttachmentEntity::Party => st.selected_customer_id.clone(),
            AttachmentEntity::Expense => st.selected_expense_id.clone(),
            AttachmentEntity::Purchase | AttachmentEntity::TrackingEvent => None,
        };
        id.filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "Select an entry first".into())
    }

    fn on_entry_attach_picked(&mut self, cx: &mut Cx, path: PathBuf) {
        let target = app_state().lock().unwrap().attach_pick.take();
        let Some(target) = target else {
            self.set_status(cx, "Nothing to attach");
            return;
        };
        match self.materialize_picked_file(cx, &path) {
            Ok(file) => match target {
                AttachPickTarget::OpsPod => {
                    app_state().lock().unwrap().pending_ops_pod = Some(file);
                    self.set_status(cx, "POD photo ready — advance tracking to save");
                    self.sync_chrome(cx);
                }
                AttachPickTarget::Image(entity) => {
                    let result = (|| -> Result<(), String> {
                        let entity_id = self.entity_id_for_attach(entity)?;
                        let mut st = app_state().lock().unwrap();
                        let actor = st.actor()?;
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.add_attachment_image(entity, &entity_id, &file, "", &actor)?;
                        st.status = "Photo attached (stored as WebP)".into();
                        Ok(())
                    })();
                    match result {
                        Ok(()) => self.set_status(cx, "Photo attached (stored as WebP)"),
                        Err(e) => self.set_status(cx, e),
                    }
                    self.sync_chrome(cx);
                }
                AttachPickTarget::File(entity) => {
                    let result = (|| -> Result<(), String> {
                        let entity_id = self.entity_id_for_attach(entity)?;
                        let mut st = app_state().lock().unwrap();
                        let actor = st.actor()?;
                        let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                        store.add_attachment_file(entity, &entity_id, &file, "", &actor)?;
                        st.status = "File attached".into();
                        Ok(())
                    })();
                    match result {
                        Ok(()) => self.set_status(cx, "File attached"),
                        Err(e) => self.set_status(cx, e),
                    }
                    self.sync_chrome(cx);
                }
            },
            Err(e) => self.set_status(cx, e),
        }
    }

    fn add_entry_note(&mut self, cx: &mut Cx, entity: AttachmentEntity, note_id: &[LiveId]) {
        let body = self.ui.text_input(cx, note_id).text();
        let result = (|| -> Result<(), String> {
            let entity_id = self.entity_id_for_attach(entity)?;
            let mut st = app_state().lock().unwrap();
            let actor = st.actor()?;
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.add_attachment_note(entity, &entity_id, "", &body, &actor)?;
            st.status = "Note added".into();
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.ui.text_input(cx, note_id).set_text(cx, "");
                self.set_status(cx, "Note added");
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
    }

    /// Copy a Storage Access Framework URI into app storage. Real file paths
    /// are returned unchanged.
    fn materialize_picked_file(
        &mut self,
        cx: &mut Cx,
        path: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let path_str = path.display().to_string();
        if !path_str.starts_with("content://") {
            return Ok(path.to_path_buf());
        }
        let bytes = cx
            .read_content_uri(&path_str)
            .ok_or_else(|| "Could not read that photo".to_string())?;
        let ext = image_extension(&bytes);
        let dir = cx
            .get_data_dir()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("picked_images");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let dest = dir.join(format!("photo-{}.{}", new_id(), ext));
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
        Ok(dest)
    }

    /// Open the books as they were in `backup`. The books as they are now are
    /// saved first (`before-restore-…`), so a wrong pick can be undone; a
    /// file that is not these books is refused before anything changes.
    /// A backup picked to restore, as a file: Android's picker returns a
    /// `content://` URI, copied beside the books; a path is used as it is.
    fn materialize_picked_backup(&mut self, cx: &mut Cx, path: &std::path::Path) -> Result<PathBuf, String> {
        let text = path.display().to_string();
        if !text.starts_with("content://") {
            return Ok(path.to_path_buf());
        }
        let bytes = cx.read_content_uri(&text).ok_or_else(|| "Could not read that backup".to_string())?;
        let dir = app_state()
            .lock()
            .unwrap()
            .store
            .as_ref()
            .map(|s| s.auto_backup_dir())
            .ok_or_else(|| "No store".to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = dir.join(format!("picked-{}.sqlite3", chrono::Local::now().format("%Y%m%d-%H%M%S")));
        std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
        Ok(file)
    }

    /// Android: make a fresh backup and ask where to save a copy of it (a USB
    /// stick, an SD card, Downloads — anywhere off the books' own storage).
    /// After a diagnostics export: keep folder under app data, then ask every
    /// platform (Linux / Windows / macOS / Android) where to save the single
    /// `.kumkaj-diag.json` bundle (Downloads, USB, share sheet).
    fn offer_diag_bundle_save(
        &mut self,
        cx: &mut Cx,
        paths: diag::ExportPaths,
        for_send: bool,
    ) {
        {
            let mut st = app_state().lock().unwrap();
            if let Some(old) = st.pending_diag_export.take() {
                let _ = std::fs::remove_file(old);
            }
            st.pending_diag_export = Some(paths.bundle.clone());
        }
        let name = paths
            .bundle
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("diagnostics.kumkaj-diag.json")
            .to_string();
        let title = app_state().lock().unwrap().t("diag_save_bundle");
        cx.open_save_file_dialog(
            FileDialog::new()
                .set_title(title)
                .set_filename(name)
                .add_filter(
                    "Kumkaj diagnostics".into(),
                    vec!["json".into(), "kumkaj-diag.json".into()],
                )
                .set_id(live_id!(diag_save_pick)),
        );
        let msg = app_state()
            .lock()
            .unwrap()
            .t(if for_send {
                "diag_export_send_done"
            } else {
                "diag_export_local_done"
            })
            .replace("{path}", &paths.bundle.display().to_string());
        self.show_ok(cx, msg);
    }

    fn write_diag_export_copy(&mut self, cx: &mut Cx, target: &std::path::Path) {
        let Some(file) = app_state().lock().unwrap().pending_diag_export.take() else {
            return;
        };
        let target_text = target.display().to_string();
        let written = match std::fs::read(&file) {
            Ok(bytes) if target_text.starts_with("content://") => {
                cx.write_content_uri(&target_text, &bytes)
            }
            Ok(_) => std::fs::copy(&file, target).is_ok(),
            Err(_) => false,
        };
        let _ = file;
        let msg = if written {
            app_state()
                .lock()
                .unwrap()
                .t("diag_bundle_saved")
                .replace("{path}", &target_text)
        } else {
            app_state().lock().unwrap().t("diag_bundle_failed")
        };
        if written {
            self.show_ok(cx, msg);
        } else {
            self.show_error(cx, msg);
        }
        self.sync_diag_panel(cx);
    }

    fn save_backup_copy(&mut self, cx: &mut Cx) {
        let made = (|| -> Result<String, String> {
            let mut st = app_state().lock().unwrap();
            st.require_server_role("Save a backup copy")?;
            // A copy whose save was cancelled goes now.
            if let Some(old) = st.pending_backup_copy.take() {
                let _ = std::fs::remove_file(old);
            }
            let store = st.store.as_ref().ok_or("No store")?;
            let stamp = chrono::Local::now().format("%Y%m%d-%H%M").to_string();
            let file = store.auto_backup_dir().join(format!("copy-{stamp}.sqlite3"));
            store.backup_to(&file)?;
            st.pending_backup_copy = Some(file);
            Ok(format!("{}-backup-{stamp}.sqlite3", store::APP_LABEL.to_lowercase().replace(' ', "-")))
        })();
        match made {
            Ok(name) => {
                let title = app_state().lock().unwrap().t("save_backup_copy");
                cx.open_save_file_dialog(
                    FileDialog::new()
                        .set_title(title)
                        .set_filename(name)
                        .add_filter("Backups".into(), vec!["sqlite3".into()])
                        .set_id(live_id!(backup_save_pick)),
                );
            }
            Err(e) => self.set_status(cx, e),
        }
    }

    /// The owner picked where the backup copy goes: write it there.
    fn write_backup_copy(&mut self, cx: &mut Cx, target: &std::path::Path) {
        let Some(file) = app_state().lock().unwrap().pending_backup_copy.take() else {
            return;
        };
        let target_text = target.display().to_string();
        let written = match std::fs::read(&file) {
            Ok(bytes) if target_text.starts_with("content://") => cx.write_content_uri(&target_text, &bytes),
            Ok(_) => std::fs::copy(&file, target).is_ok(),
            Err(_) => false,
        };
        let _ = std::fs::remove_file(&file);
        let msg = {
            let st = app_state().lock().unwrap();
            if written {
                if let Some(store) = st.store.as_ref() {
                    let _ = store.note_backup_copy_saved();
                }
                st.t("backup_copy_saved")
            } else {
                st.t("backup_copy_failed")
            }
        };
        self.set_status(cx, msg);
        self.sync_chrome(cx);
    }

    fn restore_backup_from(&mut self, cx: &mut Cx, backup: PathBuf) {
        let result = (|| -> Result<String, String> {
            store::check_backup_file(&backup)?;
            let saved = {
                let mut st = app_state().lock().unwrap();
                // Restoring a Client would overwrite its pairing with the
                // Server's own books.
                st.require_server_role("Restore a backup")?;
                let store = st.store.as_ref().ok_or("No store")?;
                let saved = store.auto_backup_dir().join(format!(
                    "before-restore-{}.sqlite3",
                    chrono::Local::now().format("%Y%m%d-%H%M%S")
                ));
                store.backup_to(&saved)?;
                st.store = None;
                saved
            };
            let new_store = Store::restore_from(&backup)?;
            let mut st = app_state().lock().unwrap();
            st.store = Some(new_store);
            st.reload_all()?;
            Ok(format!(
                "Restored from {} — the books before it are saved as {}",
                backup.file_name().unwrap_or_default().to_string_lossy(),
                saved.file_name().unwrap_or_default().to_string_lossy()
            ))
        })();
        // A failed restore after the store was let go: open the books again.
        if app_state().lock().unwrap().store.is_none() {
            self.ensure_store(cx);
        }
        match result {
            Ok(msg) | Err(msg) => self.set_status(cx, msg),
        }
        self.sync_chrome(cx);
    }

    fn on_item_photo_picked(&mut self, cx: &mut Cx, path: PathBuf) {
        match self.materialize_picked_file(cx, &path) {
            Ok(file) => {
                invalidate_image_loads_for_path(&file);
                app_state().lock().unwrap().draft_image_key =
                    Some(format!("pending_custom:{}", file.display()));
                self.set_status(cx, format!("Photo selected: {}", file.display()));
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
        self.ui.widget(cx, ids!(draft_preview)).redraw(cx);
    }

    fn party_image_clear_logo(&mut self, cx: &mut Cx) {
        // Empty string marks clear-on-save when editing; None for a new party.
        let mut st = app_state().lock().unwrap();
        if st.selected_customer_id.is_some() {
            st.draft_party_image_key = Some(String::new());
        } else {
            st.draft_party_image_key = None;
        }
        drop(st);
        self.sync_chrome(cx);
    }

    fn party_image_pick_file(&mut self, cx: &mut Cx) {
        let dialog = FileDialog::new()
            .set_title("Choose party logo".into())
            .add_filter(
                "Images".into(),
                vec!["webp".into(), "png".into(), "jpg".into(), "jpeg".into()],
            )
            .set_id(live_id!(party_photo_pick));
        cx.open_select_file_dialog(dialog);
        self.set_status(cx, "Opening logo picker…");
    }

    fn on_party_photo_picked(&mut self, cx: &mut Cx, path: PathBuf) {
        match self.materialize_picked_file(cx, &path) {
            Ok(file) => {
                invalidate_image_loads_for_path(&file);
                app_state().lock().unwrap().draft_party_image_key =
                    Some(format!("pending_custom:{}", file.display()));
                self.set_status(cx, format!("Logo selected: {}", file.display()));
            }
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
        self.ui.widget(cx, ids!(party_draft_preview)).redraw(cx);
    }
}
