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
            let actor = st.actor()?;
            if !actor.role.can_manage_menu() {
                return Err("Not allowed to manage items".into());
            }
            let name = name.trim();
            if name.is_empty() {
                return Err("Name required".into());
            }
            let currency = st.locale.currency.clone();
            let price_text = price_text.trim();
            let price_minor = if price_text.is_empty() {
                0
            } else {
                locale::parse_major_input(price_text, &currency)?
            };
            if let Some(msg) = domain::menu_price_error(Money(price_minor)) {
                return Err(msg);
            }
            let product_id = st.editing_product_id.clone().unwrap_or_default();
            let (image_bytes, image_filename, existing_image_key) = match st.draft_image_key.as_deref()
            {
                Some(k) if k.starts_with("pending_custom:") => {
                    let src = PathBuf::from(&k["pending_custom:".len()..]);
                    let bytes = std::fs::read(&src)
                        .map_err(|e| format!("Could not read the photo: {e}"))?;
                    let filename = src
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("photo.webp")
                        .to_string();
                    (bytes, filename, None)
                }
                other => (Vec::new(), String::new(), other.map(|s| s.to_string())),
            };
            let typed = category_text.trim();
            let category_id = match (&st.form_category_id, &st.form_category_label) {
                (Some(id), label) if typed.eq_ignore_ascii_case(label) => id.clone(),
                _ => String::new(),
            };
            let (category_photo_bytes, category_photo_filename) = match &st.form_category_photo {
                Some((label, photo))
                    if !typed.is_empty() && typed.eq_ignore_ascii_case(label.trim()) =>
                {
                    let bytes = std::fs::read(photo)
                        .map_err(|e| format!("Could not read the category photo: {e}"))?;
                    let filename = photo
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("category.webp")
                        .to_string();
                    (bytes, filename)
                }
                _ => (Vec::new(), String::new()),
            };
            let modifier_groups =
                domain::ModifierGroup::parse_human_list(&modifiers_text, &currency)?;
            let variations = domain::ModifierOption::parse_human_list(&variations_text, &currency)
                .map_err(|e| e.replace("Modifier", "Size"))?;
            let area_prices = domain::ModifierOption::parse_human_list(&area_prices_text, &currency)
                .map_err(|e| e.replace("Modifier", "Area"))?;
            let schedule = domain::ScheduleWindow::parse_human_list(&schedule_text, &currency)?;
            let resale = st.form_resale;
            let msg = st.books_op(BooksOp::SaveMenuItem {
                updating,
                product_id,
                name: name.to_string(),
                price_paise: price_minor,
                unit_label: unit_text.trim().to_string(),
                resale,
                aliases: typed_aliases(&aliases_text),
                tax_text: tax_text.trim().to_string(),
                category_id,
                category_name: typed.to_string(),
                category_photo_bytes,
                category_photo_filename,
                station_name: station_text.trim().to_string(),
                modifier_groups,
                variations,
                area_prices,
                schedule,
                group_name: group_text.trim().to_string(),
                recipe_text,
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
            if !actor.role.can_manage_menu() {
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

    /// Put a removed item back on the menu, exactly as it was. Works the same
    /// for dishes copied from the app's shelf and for the owner's own items —
    /// Remove only ever archived the row.
    fn restore_menu_item(&mut self, cx: &mut Cx, index: usize) {
        let result = (|| -> Result<(), String> {
            let mut st = app_state().lock().unwrap();
            let actor = st.actor()?;
            if !actor.role.can_manage_menu() {
                return Err("Not allowed to manage items".into());
            }
            let product = st
                .removed_products
                .get(index)
                .cloned()
                .ok_or_else(|| "Missing item".to_string())?;
            let msg = st.books_op(BooksOp::RestoreProduct {
                id: product.id,
                name: product.name,
            })?;
            st.status = msg;
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

    fn add_entry_note(
        &mut self,
        cx: &mut Cx,
        entity: AttachmentEntity,
        note_id: &[LiveId],
    ) {
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
    /// VACUUM runs on a worker; the save dialog opens when Signal fires.
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
                backup_copy_job(db, file, name);
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
        // VACUUM + full SQLite copy must never run on the UI thread.
        let job = (|| -> Result<(PathBuf, PathBuf, PathBuf), String> {
            store::check_backup_file(&backup)?;
            let mut st = app_state().lock().unwrap();
            st.require_server_host("Restore a backup")?;
            let store = st.store.as_ref().ok_or("No store")?;
            let db = store.db_path().to_path_buf();
            let before = store.auto_backup_dir().join(format!(
                "before-restore-{}.sqlite3",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            ));
            // Release the UI connection so the worker can VACUUM / restore.
            st.store = None;
            Ok((db, backup, before))
        })();
        match job {
            Ok((db, backup, before)) => {
                self.set_status(cx, "Restoring backup…");
                restore_job(db, backup, before);
                self.sync_chrome(cx);
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

    fn on_category_photo_picked(&mut self, cx: &mut Cx, path: PathBuf) {
        let category_id = app_state().lock().unwrap().category_photo_id.take();
        let Some(category_id) = category_id else {
            self.set_status(cx, "Pick a category first");
            return;
        };
        let outcome = (|| -> Result<String, String> {
            let file = self.materialize_picked_file(cx, &path)?;
            let bytes = std::fs::read(&file)
                .map_err(|e| format!("Could not read the category photo: {e}"))?;
            let filename = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("category.webp")
                .to_string();
            invalidate_image_loads_for_path(&file);
            let mut st = app_state().lock().unwrap();
            let msg = st.books_op(BooksOp::ImportCategoryImage {
                category_id: category_id.clone(),
                filename,
                bytes,
            })?;
            if let Some(store) = st.store.as_ref() {
                if let Ok(dir) = store.item_images_dir() {
                    invalidate_image_loads_for_path(&dir.join(format!("cat-{category_id}.webp")));
                }
                if st.device_role != DeviceRole::Client {
                    st.categories = store.list_categories().unwrap_or_default();
                }
            }
            Ok(msg)
        })();
        match outcome {
            Ok(msg) => self.set_status(cx, msg),
            Err(e) => self.set_status(cx, e),
        }
        self.sync_chrome(cx);
        self.ui.redraw(cx);
    }
}
