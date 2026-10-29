// Runs during MSI uninstall only (see build-msi.sh): asks which local data
// categories to delete, then deletes them. Trading / UI wipes call the still-
// installed exe (--uninstall-wipe) so SQLite rows are cleared correctly;
// schema wipe removes the books folders from JScript.
// Placeholders __APP_ID__, __APP_LABEL__, __BIN_NAME__ are replaced by build-msi.sh.
// The whole body is guarded: any failure here must never block uninstall.
function PromptDeleteBooksData() {
    try {
        var shell = new ActiveXObject("WScript.Shell");
        var fso = new ActiveXObject("Scripting.FileSystemObject");
        var dir = shell.ExpandEnvironmentStrings("%APPDATA%") + "\\__APP_ID__";
        if (!fso.FolderExists(dir)) {
            return;
        }
        // Button flags 4 = Yes/No, icon 32 = question, 4096 = system-modal
        // (without it the prompt opens behind the uninstall progress window).
        // 60s timeout returns -1 (treated as No) so a silent uninstall
        // never hangs on this.
        var flags = 4 + 32 + 4096;

        var delRecords = shell.Popup(
            "Delete __APP_LABEL__ business records?\n\n" +
                "Sales, payments, day books and history.\n" +
                "Shop setup (products, rates, staff, brand) stays.\n\n" +
                dir +
                "\n\nYes - delete records forever.\nNo - keep records.",
            60,
            "Uninstall __APP_LABEL__",
            flags
        ) == 6;

        var delSetup = shell.Popup(
            "Also delete __APP_LABEL__ shop setup?\n\n" +
                "Products, rates, staff, brand and empty books databases.\n" +
                "Choosing Yes also removes any business records still there.\n\n" +
                dir +
                "\n\nYes - delete setup forever.\nNo - keep setup.",
            60,
            "Uninstall __APP_LABEL__",
            flags
        ) == 6;

        var delUi = shell.Popup(
            "Delete saved __APP_LABEL__ screen preferences?\n\n" +
                "Sidebar order, hidden amounts, window size and language.\n\n" +
                dir +
                "\n\nYes - reset preferences.\nNo - keep preferences.",
            60,
            "Uninstall __APP_LABEL__",
            flags
        ) == 6;

        if (!delRecords && !delSetup && !delUi) {
            return;
        }

        var installDir = "";
        try {
            installDir = Session.Property("INSTALLFOLDER");
        } catch (e2) {
            installDir = "";
        }
        var exe = "";
        if (installDir && fso.FileExists(installDir + "__BIN_NAME__.exe")) {
            exe = installDir + "__BIN_NAME__.exe";
        }

        function runWipe(cats) {
            if (!exe || !cats) {
                return;
            }
            // Window style 0 = hidden; true = wait. Exe is still on disk
            // (this action runs Before="RemoveFiles").
            try {
                shell.Run('"' + exe + '" --uninstall-wipe ' + cats, 0, true);
            } catch (e3) {
                // Fall through; folder deletes below still help.
            }
        }

        if (delSetup) {
            // Full shop databases + media. Prefer the exe so WAL sidecars
            // and nested folders are handled the same as a reinstall wipe.
            runWipe(delUi ? "schema ui" : "schema");
            // Belt-and-braces if the exe could not run.
            try {
                if (fso.FolderExists(dir + "\\servers")) {
                    fso.DeleteFolder(dir + "\\servers", true);
                }
                if (fso.FolderExists(dir + "\\client")) {
                    fso.DeleteFolder(dir + "\\client", true);
                }
                if (fso.FolderExists(dir + "\\backups")) {
                    fso.DeleteFolder(dir + "\\backups", true);
                }
                if (fso.FileExists(dir + "\\servers.json")) {
                    fso.DeleteFile(dir + "\\servers.json", true);
                }
            } catch (e4) {}
            if (delUi) {
                try {
                    if (fso.FileExists(dir + "\\window_geom.json")) {
                        fso.DeleteFile(dir + "\\window_geom.json", true);
                    }
                    if (fso.FileExists(dir + "\\app_language.json")) {
                        fso.DeleteFile(dir + "\\app_language.json", true);
                    }
                } catch (e5) {}
            }
        } else {
            var cats = "";
            if (delRecords) {
                cats += "trading ";
            }
            if (delUi) {
                cats += "ui ";
            }
            runWipe(cats.replace(/\s+$/, ""));
            if (delUi && !exe) {
                try {
                    if (fso.FileExists(dir + "\\window_geom.json")) {
                        fso.DeleteFile(dir + "\\window_geom.json", true);
                    }
                    if (fso.FileExists(dir + "\\app_language.json")) {
                        fso.DeleteFile(dir + "\\app_language.json", true);
                    }
                } catch (e6) {}
            }
        }

        // Remove the data folder if it is now empty.
        try {
            if (fso.FolderExists(dir)) {
                var folder = fso.GetFolder(dir);
                if (folder.Files.Count == 0 && folder.SubFolders.Count == 0) {
                    fso.DeleteFolder(dir, true);
                }
            }
        } catch (e7) {}
    } catch (e) {
        // Keep the data and let the uninstall continue.
    }
}
