/// Settings hub: one section open at a time (`AppState::settings_panel`).
/// The single page used to stack every control — language lists, printer
/// routes, AWB pools, backup, diagnostics — and scroll for ever (Khaynjo's
/// shop trials settled on a hub, 2026-10-31). Each row: section key, hub
/// row, its panel, the locale key of the row's label.
const SETTINGS_SECTIONS: [(&str, &[LiveId], &[LiveId], &str); 11] = [
    ("language", ids!(settings_hub_language), ids!(settings_panel_language), "language_currency"),
    ("country", ids!(settings_hub_country), ids!(settings_panel_country), "country_tax"),
    ("brand", ids!(settings_hub_brand), ids!(settings_panel_brand), "your_brand"),
    ("printers", ids!(settings_hub_printers), ids!(settings_panel_printers), "printers_heading"),
    ("awb", ids!(settings_hub_awb), ids!(settings_panel_awb), "awb_pools_heading"),
    ("measures", ids!(settings_hub_measures), ids!(settings_panel_measures), "booking_measures"),
    ("units", ids!(settings_hub_units), ids!(settings_panel_units), "units_heading"),
    ("devices", ids!(settings_hub_devices), ids!(settings_panel_devices), "settings_sec_devices"),
    ("alerts", ids!(settings_hub_alerts), ids!(settings_panel_alerts), "settings_sec_alerts"),
    ("backup", ids!(settings_hub_backup), ids!(settings_panel_backup), "backup"),
    ("diag", ids!(settings_hub_diag), ids!(settings_panel_diag), "settings_sec_diag"),
];

/// Back on Settings: an open section closes to the hub first; only the hub
/// itself leaves the screen. Returns the panel to show next, or `None` when
/// Back should leave Settings.
fn settings_back_step(screen: Screen, panel: Option<&'static str>) -> Option<Option<&'static str>> {
    (screen == Screen::Settings && panel.is_some()).then_some(None)
}

impl App {
    /// Hub vs the one open section, plus the hub rows' words.
    fn sync_settings_panels(&mut self, cx: &mut Cx) {
        let panel = app_state().lock().unwrap().settings_panel;
        self.ui.view(cx, ids!(settings_hub)).set_visible(cx, panel.is_none());
        for (key, _, view, _) in SETTINGS_SECTIONS {
            self.ui.view(cx, view).set_visible(cx, panel == Some(key));
        }
        let st = app_state().lock().unwrap();
        let s = |k: &str| st.t(k);
        for (_, hub, _, label) in SETTINGS_SECTIONS {
            self.ui.button(cx, hub).set_text(cx, &s(label));
        }
        self.ui
            .button(cx, ids!(sidebar_settings_open))
            .set_text(cx, &s("sidebar_settings"));
        self.ui
            .widget(cx, ids!(h_settings_language))
            .set_text(cx, &plain_display_text(&s("language_currency")));
        if panel == Some("alerts") {
            self.ui
                .widget(cx, ids!(h_alerts))
                .set_text(cx, &plain_display_text(&s("settings_sec_alerts")));
            self.ui
                .widget(cx, ids!(h_alerts_help))
                .set_text(cx, &plain_display_text(&s("alerts_help")));
            self.ui
                .text_input(cx, ids!(snack_secs_input))
                .set_empty_text(cx, s("snack_secs_ph"));
            self.ui
                .button(cx, ids!(snack_secs_save))
                .set_text(cx, &s("snack_secs_save"));
            let field = self.ui.text_input(cx, ids!(snack_secs_input));
            if !cx.has_key_focus(field.area()) {
                field.set_text(cx, &format!("{}", st.snack_secs.round() as i64));
            }
        }
    }

    /// Show one section (or the hub with `None`) from its top.
    fn open_settings_panel(&mut self, cx: &mut Cx, panel: Option<&'static str>) {
        {
            let mut st = app_state().lock().unwrap();
            st.settings_panel = panel;
            st.suggest_open = None;
        }
        // A row hidden under the pointer never hears the mouse leave, so it
        // came back grey (hovered) with the hub — clear every row's face.
        for (_, hub, _, _) in SETTINGS_SECTIONS {
            self.clear_button_animators(cx, hub);
        }
        self.clear_button_animators(cx, ids!(sidebar_settings_open));
        self.ui
            .view(cx, ids!(settings_screen))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        diag::breadcrumb("settings_panel", panel.unwrap_or("hub"));
        self.sync_chrome(cx);
    }

    fn handle_settings_hub_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let opened = SETTINGS_SECTIONS
            .iter()
            .find(|(_, hub, _, _)| self.ui.button(cx, hub).clicked(actions))
            .map(|(key, _, _, _)| *key);
        if let Some(key) = opened {
            self.open_settings_panel(cx, Some(key));
        }
        if self.ui.button(cx, ids!(snack_secs_save)).clicked(actions) {
            let typed = self.ui.text_input(cx, ids!(snack_secs_input)).text();
            let result = (|| -> Result<String, String> {
                let mut st = app_state().lock().unwrap();
                let secs = typed
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| st.t("snack_secs_bad"))?;
                let secs = SnackKind::clamp_secs(secs);
                let user = st
                    .session
                    .as_ref()
                    .map(lan_key)
                    .ok_or_else(|| "Unlock first".to_string())?;
                let store = st.store.as_ref().ok_or_else(|| "No store".to_string())?;
                store.set_snack_secs(&user, secs)?;
                st.snack_secs = secs;
                Ok(st
                    .t("snack_secs_saved")
                    .replace("{n}", &format!("{}", secs.round() as i64)))
            })();
            match result {
                Ok(msg) => self.set_status(cx, msg),
                Err(e) => self.set_status(cx, e),
            }
            // The field shows what was kept (a typed 99 reads back 60).
            let secs = app_state().lock().unwrap().snack_secs;
            self.ui
                .text_input(cx, ids!(snack_secs_input))
                .set_text(cx, &format!("{}", secs.round() as i64));
            self.sync_chrome(cx);
        }
    }
}
