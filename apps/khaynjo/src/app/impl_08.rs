impl App {
fn add_menu_item(&mut self, cx: &mut Cx) {
        let name = self.ui.text_input(cx, ids!(menu_name_input)).text();
        let aliases_text = self.ui.text_input(cx, ids!(menu_aliases_input)).text();
        let price_text = self.ui.text_input(cx, ids!(menu_price_input)).text();
        let unit_text = self.ui.text_input(cx, ids!(menu_unit_input)).text();
        let tax_text = self.ui.text_input(cx, ids!(menu_tax_input)).text();
        let category_text = self.ui.text_input(cx, ids!(menu_category_input)).text();
        let modifiers_text = self.ui.text_input(cx, ids!(menu_modifiers_input)).text();
        let variations_text = self.ui.text_input(cx, ids!(menu_variations_input)).text();
        let group_text = self.ui.text_input(cx, ids!(menu_group_input)).text();
        let area_prices_text = self.ui.text_input(cx, ids!(menu_area_prices_input)).text();
        let schedule_text = self.ui.text_input(cx, ids!(menu_schedule_input)).text();
        let recipe_text = self.ui.text_input(cx, ids!(menu_recipe_input)).text();
        let station_text = self.ui.text_input(cx, ids!(menu_station_input)).text();
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let updating = st.editing_product_id.is_some();
            st.require_server_role(if updating { "Change items" } else { "Add items" })?;
            let actor = st.actor()?;
            if !actor.role.can_manage_menu() {
                return Err("Not allowed to manage items".into());
            }
            let name = name.trim();
            if name.is_empty() {
                return Err("Name required".into());
            }
            let currency = st.locale.currency.clone();
            // Blank reads as ₹0, which is a real price (complimentary items,
            // dishes priced at the counter by their @). Only negative is
            // refused; the till bills whatever the form saves.
            let price_text = price_text.trim();
            let price_minor = if price_text.is_empty() {
                0
            } else {
                locale::parse_major_input(price_text, &currency)?
            };
            if let Some(msg) = domain::menu_price_error(Money(price_minor)) {
                return Err(msg);
            }
            let mut image_key = st.draft_image_key.clone();
            let editing_id = st.editing_product_id.clone();
            // Auto: leave None so display always re-detects from name.
            // Built-in / custom: persist chosen key.
            let product_id = editing_id.clone().unwrap_or_else(new_id);
            // Sizes typed on a new item make only the size items, not the
            // base — so its photo is copied to each of them, not imported here.
            let sizes_only = !variations_text.trim().is_empty() && !updating;
            let photo_source: Option<PathBuf> = match image_key.as_deref() {
                Some(k) if k.starts_with("pending_custom:") => {
                    Some(PathBuf::from(&k["pending_custom:".len()..]))
                }
                Some(k) if k.starts_with("custom:") => st
                    .store
                    .as_ref()
                    .and_then(|s| s.item_images_dir().ok())
                    .map(|d| d.join(&k["custom:".len()..])),
                _ => None,
            };
            if let Some(key) = image_key.clone()
                && key.starts_with("pending_custom:")
                && !sizes_only {
                    let src = std::path::PathBuf::from(&key["pending_custom:".len()..]);
                    let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                    image_key = Some(store.import_item_image(&product_id, &src)?);
                }
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            // Sellers type either a rate from their country pack ("Standard",
            // "GST 18%") or a bare percent, which becomes a named rate.
            let tax_rate = store.resolve_tax_rate(&tax_text)?;
            // A category picked from the Categories list travels by
            // UUID; the typed text only has to still match what was
            // picked. Retyping the field hands control back to name
            // resolution (find-or-create by name — a typing
            // convenience, never the identity).
            let typed = category_text.trim();
            let picked_id = match (&st.form_category_id, &st.form_category_label) {
                (Some(id), label) if typed.eq_ignore_ascii_case(label) => store
                    .list_categories()?
                    .into_iter()
                    .find(|c| &c.id == id)
                    .map(|c| c.id),
                _ => None,
            };
            let category_id = match picked_id {
                Some(id) => id,
                None => match typed {
                    "" => String::new(),
                    wanted => match store
                        .list_categories()?
                        .into_iter()
                        .find(|c| c.name.eq_ignore_ascii_case(wanted))
                    {
                        Some(c) => c.id,
                        None => store.add_category(wanted)?.id,
                    },
                },
            };
            // A shelf folder picked as the category brings its photo, the way
            // a shelf dish brings the item's — only onto a category that has
            // no photo yet, and only if the field still names that folder.
            if let Some((label, photo)) = st.form_category_photo.clone()
                && !category_id.is_empty()
                && typed.eq_ignore_ascii_case(label.trim())
            {
                store.set_category_photo_if_missing(&category_id, &photo)?;
            }
            // Kitchen station by name (created if new), same pattern as category.
            let station_id = match station_text.trim() {
                "" => String::new(),
                wanted => match store
                    .list_stations()?
                    .into_iter()
                    .find(|s| s.name.eq_ignore_ascii_case(wanted))
                {
                    Some(s) => s.id,
                    None => store.add_station(wanted)?.id,
                },
            };
            // Groups: `Size*: Small=199, Large=399 | Toppings+: Cheese=40`
            // or a flat add-on list `Extra cheese=40`. Separate-item sizes
            // still use the Sizes field below.
            let modifier_groups =
                domain::ModifierGroup::parse_human_list(&modifiers_text, &currency)?;
            let variations = domain::ModifierOption::parse_human_list(&variations_text, &currency)
                .map_err(|e| e.replace("Modifier", "Size"))?;
            let area_prices = domain::ModifierOption::parse_human_list(&area_prices_text, &currency)
                .map_err(|e| e.replace("Modifier", "Area"))?;
            let mut product = if let Some(id) = editing_id {
                store.get_product(&id)?.ok_or_else(|| {
                    "That item is no longer in Items".to_string()
                })?
            } else {
                Product::new_basic(product_id, name, Money(price_minor))
            };
            product.name = name.to_string();
            product.price = Money(price_minor);
            product.unit_label = unit_text.trim().to_string();
            product.kind = if st.form_resale { domain::ITEM_KIND_RESALE } else { domain::ITEM_KIND_DISH }.into();
            // A resold good is counted from the start.
            if st.form_resale && !product.tracks_stock() {
                product.stock_qty = 0;
            }
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
            product.station_id = station_id;
            product.set_modifier_groups(&modifier_groups);
            product.set_area_prices(&area_prices);
            let schedule = domain::ScheduleWindow::parse_human_list(&schedule_text, &currency)?;
            product.set_schedule(&schedule);
            product.group_name = group_text.trim().to_string();
            if !sizes_only {
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
                // Only touch the recipe when there is one, or one to clear.
                if !recipe_text.trim().is_empty() || !store.recipe(&product.id)?.is_empty() {
                    store.set_recipe_from_text(&product.id, &recipe_text, &actor)?;
                }
            }
            // Each size is an item of its own (own UUID, price, stock,
            // station), grouped under this dish's name.
            let sizes = if variations.is_empty() {
                Vec::new()
            } else {
                store.add_variant_items(&product, &variations, photo_source.as_deref(), &actor)?
            };
            st.draft_image_key = None;
            st.editing_product_id = None;
            st.reload_all()?;
            let size_names = sizes.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ");
            st.status = match (updating, sizes.is_empty()) {
                (true, true) => format!("Updated {}", product.name),
                (false, true) => format!("Added {}", product.name),
                (true, false) => format!("Updated {} · added {size_names}", product.name),
                (false, false) => format!("Added {size_names}"),
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
            if !actor.role.can_manage_menu() {
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

    /// Put a removed item back on the menu, exactly as it was. Works the same
    /// for dishes copied from the app's shelf and for the owner's own items —
    /// Remove only ever archived the row.
    fn restore_menu_item(&mut self, cx: &mut Cx, index: usize) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            st.require_server_role("Restore items")?;
            let actor = st.actor()?;
            if !actor.role.can_manage_menu() {
                return Err("Not allowed to manage items".into());
            }
            let product = st
                .removed_products
                .get(index)
                .cloned()
                .ok_or_else(|| "Missing item".to_string())?;
            let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
            store.restore_product(&product.id)?;
            store.audit(&actor, "menu.restore", &product.name)?;
            st.reload_all()?;
            st.status = format!("{} back on the menu", product.name);
            Ok(())
        })();
        if let Err(e) = result {
            self.set_status(cx, e);
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
            .set_title("Choose item photo".into())
            .add_filter(
                "Images".into(),
                vec![
                    "webp".into(),
                    "png".into(),
                    "jpg".into(),
                    "jpeg".into(),
                ],
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
                vec![
                    "webp".into(),
                    "png".into(),
                    "jpg".into(),
                    "jpeg".into(),
                ],
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

    /// Which row an attachment belongs to (sale / party / expense selection).
    fn entity_id_for_attach(&self, entity: AttachmentEntity) -> Result<String, String> {
        let st = app_state().lock().unwrap();
        let id = match entity {
            AttachmentEntity::Sale => st.selected_sale_id.clone(),
            AttachmentEntity::Party => st.selected_customer_id.clone(),
            AttachmentEntity::Expense => st.selected_expense_id.clone(),
            AttachmentEntity::Purchase => None,
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

    fn add_entry_note(
        &mut self,
        cx: &mut Cx,
        entity: AttachmentEntity,
        note_id: &[LiveId],
    ) {
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
    fn materialize_picked_file(&mut self, cx: &mut Cx, path: &std::path::Path) -> Result<PathBuf, String> {
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
                .add_filter("Kumkaj diagnostics".into(), vec!["json".into(), "kumkaj-diag.json".into()])
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
        // Keep the in-app copy; only clear the pending pointer.
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

    /// Android: make a fresh backup and ask where to save a copy of it (a USB
    /// stick, an SD card, Downloads — anywhere off the books' own storage).
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

    fn on_category_photo_picked(&mut self, cx: &mut Cx, path: PathBuf) {
        let category_id = app_state().lock().unwrap().category_photo_id.take();
        let Some(category_id) = category_id else {
            self.set_status(cx, "Pick a category first");
            return;
        };
        let outcome = (|| -> Result<String, String> {
            let file = self.materialize_picked_file(cx, &path)?;
            let mut st = app_state().lock().unwrap();
            let store = st
                .store
                .as_ref()
                .ok_or_else(|| "Storage not ready".to_string())?;
            let key = store.import_category_image(&category_id, &file)?;
            invalidate_image_loads_for_path(&file);
            // Stable on-disk name may be overwritten — force a fresh decode.
            if let Ok(dir) = store.item_images_dir() {
                invalidate_image_loads_for_path(&dir.join(format!("cat-{category_id}.webp")));
            }
            store.set_category_image(&category_id, &key)?;
            st.categories = store.list_categories().unwrap_or_default();
            Ok("Category photo saved".to_string())
        })();
        match outcome {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
        self.ui.redraw(cx);
    }
}
