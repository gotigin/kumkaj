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

    /// Pull Unit @ / Applied / note from a ticket row into the cart before any
    /// action that sync-paints (±, modifiers, pay). Quiet — caller syncs when needed.
    fn commit_cart_row_fields(&mut self, cx: &mut Cx, index: usize, item: &WidgetRef) {
        let unit_text = item.text_input(cx, ids!(unit_price)).text();
        let total_text = item.text_input(cx, ids!(line_total)).text();
        let note_text = item.text_input(cx, ids!(note)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if index >= st.cart.lines.len() {
                return Ok(());
            }
            let currency = st.locale.currency.clone();
            let unit_minor = locale::parse_major_input(&unit_text, &currency)?;
            st.cart.set_unit_price(index, Money(unit_minor))?;
            let total_minor = locale::parse_major_input(&total_text, &currency)?;
            st.cart.set_line_total(index, Money(total_minor))?;
            st.cart.set_note(index, note_text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
    }

    fn commit_amend_row_fields(&mut self, cx: &mut Cx, index: usize, item: &WidgetRef) {
        let unit_text = item.text_input(cx, ids!(unit_price)).text();
        let total_text = item.text_input(cx, ids!(line_total)).text();
        let note_text = item.text_input(cx, ids!(note)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            if index >= st.amend_cart.lines.len() {
                return Ok(());
            }
            let currency = st.locale.currency.clone();
            let unit_minor = locale::parse_major_input(&unit_text, &currency)?;
            st.amend_cart.set_unit_price(index, Money(unit_minor))?;
            let total_minor = locale::parse_major_input(&total_text, &currency)?;
            st.amend_cart.set_line_total(index, Money(total_minor))?;
            st.amend_cart.set_note(index, note_text)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
        }
    }

    /// Pay runs before the cart portal loop — commit row widgets into the cart.
    fn commit_cart_widget_fields_before_pay(&mut self, cx: &mut Cx) {
        let len = app_state().lock().unwrap().cart.lines.len();
        if len == 0 {
            return;
        }
        let host = self.ui.widget(cx, ids!(cart_list));
        let list = host.portal_list(cx, ids!(list));
        for index in 0..len {
            let item = list.item(cx, index, live_id!(Item));
            self.commit_cart_row_fields(cx, index, &item);
        }
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
            let actor = st.actor()?;
            if !actor.is_owner() && !actor.can_access_items() {
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
            let modifiers = domain::ModifierOption::parse_human_list(&modifiers_text, &currency)?;
            let product_id = st.editing_product_id.clone().unwrap_or_default();
            let (image_bytes, image_filename, existing_image_key) = match st.draft_image_key.clone()
            {
                Some(key) if key.starts_with("pending_custom:") => {
                    let src = std::path::PathBuf::from(&key["pending_custom:".len()..]);
                    let bytes = std::fs::read(&src)
                        .map_err(|e| format!("Could not read the photo: {e}"))?;
                    let filename = src
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("item.webp")
                        .to_string();
                    (bytes, filename, None)
                }
                other => (Vec::new(), String::new(), other),
            };
            let msg = st.books_op(BooksOp::SaveItem {
                updating,
                product_id,
                name: name.to_string(),
                price_paise: price_minor,
                unit_label: unit_text.trim().to_string(),
                aliases: typed_aliases(&aliases_text),
                tax_text: tax_text.trim().to_string(),
                category_name: category_text.trim().to_string(),
                modifiers,
                first_kg: first_kg_text,
                first_paise: locale::parse_major_input(&first_amt_text, &currency).unwrap_or(0),
                extra_per_kg_paise: locale::parse_major_input(&extra_kg_text, &currency)
                    .unwrap_or(0),
                zone_name: zone_text.trim().to_string(),
                carrier_name: carrier_text.trim().to_string(),
                party_name: party_text.trim().to_string(),
                image_bytes,
                image_filename,
                existing_image_key,
            })?;
            st.draft_image_key = None;
            st.editing_product_id = None;
            st.status = msg;
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
            let actor = st.actor()?;
            if !actor.is_owner() && !actor.can_access_items() {
                return Err("Not allowed to manage items".into());
            }
            let product = st
                .products
                .get(index)
                .cloned()
                .ok_or_else(|| "Missing product".to_string())?;
            let msg = st.books_op(BooksOp::ArchiveProduct {
                id: product.id.clone(),
                name: product.name.clone(),
            })?;
            if st.editing_product_id.as_deref() == Some(product.id.as_str()) {
                st.editing_product_id = None;
                st.draft_image_key = None;
            }
            st.cart.lines.retain(|l| l.product_id != product.id);
            st.status = msg;
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
                st.books_op(BooksOp::SetUserLanguage {
                    code: code.to_string(),
                })?;
                if st.device_role == DeviceRole::Client {
                    st.locale.language = code.into();
                    st.language_follow_platform = false;
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
            if st.store.is_some() {
                let msg = st.books_op(BooksOp::SetCurrency {
                    code: code.to_string(),
                })?;
                if st.device_role == DeviceRole::Client {
                    st.locale.currency = code.into();
                }
                st.status = msg;
            } else {
                st.status = format!("Currency: {code}");
            }
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
                    let result = (|| -> Result<String, String> {
                        let entity_id = self.entity_id_for_attach(entity)?;
                        let bytes = std::fs::read(&file)
                            .map_err(|e| format!("Could not read the photo: {e}"))?;
                        let filename = file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("photo.webp")
                            .to_string();
                        let mut st = app_state().lock().unwrap();
                        st.books_op(BooksOp::AddAttachmentImage {
                            entity,
                            entity_id,
                            label: String::new(),
                            filename,
                            bytes,
                        })
                    })();
                    match result {
                        Ok(msg) => self.set_status(cx, msg),
                        Err(e) => self.set_status(cx, e),
                    }
                    self.sync_chrome(cx);
                }
                AttachPickTarget::File(entity) => {
                    let result = (|| -> Result<String, String> {
                        let entity_id = self.entity_id_for_attach(entity)?;
                        let bytes = std::fs::read(&file)
                            .map_err(|e| format!("Could not read the file: {e}"))?;
                        let filename = file
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file.pdf")
                            .to_string();
                        let mut st = app_state().lock().unwrap();
                        st.books_op(BooksOp::AddAttachmentFile {
                            entity,
                            entity_id,
                            label: String::new(),
                            filename,
                            bytes,
                        })
                    })();
                    match result {
                        Ok(msg) => self.set_status(cx, msg),
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
        let result = (|| -> Result<String, String> {
            let entity_id = self.entity_id_for_attach(entity)?;
            let mut st = app_state().lock().unwrap();
            st.books_op(BooksOp::AddAttachmentNote {
                entity,
                entity_id,
                label: String::new(),
                body,
            })
        })();
        match result {
            Ok(msg) => {
                self.ui.text_input(cx, note_id).set_text(cx, "");
                self.set_status(cx, msg);
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
        if app_state().lock().unwrap().device_role == DeviceRole::Client {
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                st.books_op(BooksOp::SaveBackupCopy)
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            return;
        }
        // VACUUM runs on a worker; the save dialog opens when Signal fires.
        let job = (|| -> Result<(PathBuf, PathBuf, String), String> {
            let mut st = app_state().lock().unwrap();
            if let Some(old) = st.pending_backup_copy.take() {
                let _ = std::fs::remove_file(old);
            }
            let store = st.store.as_ref().ok_or("No store")?;
            let stamp = chrono::Local::now().format("%Y%m%d-%H%M").to_string();
            let file = store.auto_backup_dir().join(format!("copy-{stamp}.sqlite3"));
            let db = store.db_path().to_path_buf();
            let name = format!(
                "{}-backup-{stamp}.sqlite3",
                store::APP_LABEL.to_lowercase().replace(' ', "-")
            );
            Ok((db, file, name))
        })();
        match job {
            Ok((db, file, name)) => {
                self.set_status(cx, "Making backup copy…");
                self.paint_submit_busy_chrome(cx);
                backup_copy_job(db, file, name);
            }
            Err(e) => self.set_status(cx, e),
        }
    }

    /// The owner picked where the backup copy goes: write it there — on a
    /// worker (`backup_copy_write_job`) with the same Progress + Cancel as
    /// the VACUUM half. Writing a whole books file here froze the till for
    /// seconds right after the save dialog (DEV.md 7b).
    fn write_backup_copy(&mut self, cx: &mut Cx, target: &std::path::Path) {
        let Some(file) = app_state().lock().unwrap().pending_backup_copy.take() else {
            return;
        };
        backup_copy_write_job(file, target.display().to_string());
        self.paint_submit_busy_chrome(cx);
    }

    /// The backup copy write finished (Signal): say how it went.
    fn finish_backup_copy_write(&mut self, cx: &mut Cx, result: Result<(), String>) {
        let msg = {
            let st = app_state().lock().unwrap();
            match &result {
                Ok(()) => {
                    if let Some(store) = st.store.as_ref() {
                        let _ = store.note_backup_copy_saved();
                    }
                    st.t("backup_copy_saved")
                }
                Err(e) if e == "Cancelled" => st.t("cancelled"),
                Err(_) => st.t("backup_copy_failed"),
            }
        };
        self.set_status(cx, msg);
        self.paint_submit_busy_chrome(cx);
        self.sync_chrome(cx);
    }

    fn restore_backup_from(&mut self, cx: &mut Cx, backup: PathBuf) {
        // VACUUM + full SQLite copy must never run on the UI thread.
        let job = (|| -> Result<(PathBuf, PathBuf, PathBuf), String> {
            store::check_backup_file(&backup)?;
            let mut st = app_state().lock().unwrap();
            st.require_server_host("Restore a backup")?;
            // Backup Settings access is not enough — restore replaces the books.
            if !st.session.as_ref().is_some_and(|a| a.is_owner()) {
                return Err(st.t("restore_owner_only"));
            }
            let store = st.store.as_ref().ok_or("No store")?;
            let db = store.db_path().to_path_buf();
            let before = store.auto_backup_dir().join(format!(
                "before-restore-{}.sqlite3",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            ));
            st.store = None;
            Ok((db, backup, before))
        })();
        match job {
            Ok((db, backup, before)) => {
                self.set_status(cx, "Restoring backup…");
                self.paint_submit_busy_chrome(cx);
                restore_job(db, backup, before);
            }
            Err(e) => {
                if app_state().lock().unwrap().store.is_none() {
                    self.ensure_store(cx);
                }
                self.set_status(cx, e);
                self.sync_chrome(cx);
            }
        }
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
