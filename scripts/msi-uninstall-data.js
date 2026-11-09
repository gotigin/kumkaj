// Runs during MSI uninstall only (see build-msi.sh): one dialog with three
// checkboxes for which local data to delete, then deletes them. Same path as
// the old three shell.Popup Yes/No prompts (InstallExecuteSequence Before
// RemoveFiles) — only the UI is one checkbox dialog via mshta (WScript.Popup
// cannot host checkboxes; PowerShell WinForms does not show from this CA).
// Trading / UI wipes call the still-installed exe (--uninstall-wipe);
// schema wipe removes the books folders from JScript.
// Placeholders __APP_ID__, __APP_LABEL__, __BIN_NAME__ are replaced by build-msi.sh.
// The whole body is guarded: any failure here must never block uninstall.
function PromptDeleteBooksData() {
    try {
        var shell = new ActiveXObject("WScript.Shell");
        var fso = new ActiveXObject("Scripting.FileSystemObject");
        // Drop Desktop / taskbar-pin copies if a prior Setup.bat or pin fallback left them
        // (WiX also removes its own Desktop shortcut with the product).
        try {
            var desk = shell.SpecialFolders("Desktop") + "\\__APP_LABEL__.lnk";
            if (fso.FileExists(desk)) {
                fso.DeleteFile(desk, true);
            }
            var pin =
                shell.ExpandEnvironmentStrings("%APPDATA%") +
                "\\Microsoft\\Internet Explorer\\Quick Launch\\User Pinned\\TaskBar\\__APP_LABEL__.lnk";
            if (fso.FileExists(pin)) {
                fso.DeleteFile(pin, true);
            }
        } catch (ePin) {}
        var dir = shell.ExpandEnvironmentStrings("%APPDATA%") + "\\__APP_ID__";
        if (!fso.FolderExists(dir)) {
            return;
        }

        var choice = promptWipeCheckboxes(shell, fso, dir);
        var delRecords = choice.delRecords;
        var delSetup = choice.delSetup;
        var delUi = choice.delUi;

        if (!delRecords && !delSetup && !delUi) {
            return;
        }

        var installDir = "";
        try {
            installDir = Session.Property("INSTALLFOLDER");
        } catch (e2) {
            installDir = "";
        }
        if (!installDir) {
            try {
                installDir =
                    shell.ExpandEnvironmentStrings("%LOCALAPPDATA%") +
                    "\\Programs\\__APP_LABEL__\\";
            } catch (eDir) {
                installDir = "";
            }
        }
        if (installDir && installDir.charAt(installDir.length - 1) !== "\\") {
            installDir += "\\";
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

// One checkbox dialog (records / shop setup / screen prefs). Written as a
// temp .hta and shown with mshta — same process model as shell.Popup from
// this Immediate CA; returns {delRecords, delSetup, delUi}.
function promptWipeCheckboxes(shell, fso, dir) {
    var empty = { delRecords: false, delSetup: false, delUi: false };
    try {
        var tmp = shell.ExpandEnvironmentStrings("%TEMP%");
        var hta = tmp + "\\kumkaj_wipe___APP_ID__.hta";
        var out = tmp + "\\kumkaj_wipe___APP_ID__.txt";
        try {
            if (fso.FileExists(out)) {
                fso.DeleteFile(out, true);
            }
        } catch (e0) {}
        var outJs = out.replace(/\\/g, "\\\\");
        var dirHtml = String(dir)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
        var html =
            "<html><head><meta http-equiv='X-UA-Compatible' content='IE=edge'/>" +
            "<title>Uninstall __APP_LABEL__ — local data</title>" +
            "<HTA:APPLICATION ID='KumkajWipe' APPLICATIONNAME='__APP_LABEL__' " +
            "BORDER='dialog' SCROLL='no' SINGLEINSTANCE='yes' SYSMENU='yes'/>" +
            "<style>body{font:13px Segoe UI,sans-serif;margin:16px;}" +
            "label{display:block;margin:10px 0;}" +
            "button{margin-right:8px;min-width:110px;padding:4px 10px;}</style>" +
            "<script language='JScript'>function finish(ok){" +
            "var f=new ActiveXObject('Scripting.FileSystemObject');" +
            "var t=f.CreateTextFile('" +
            outJs +
            "',true);" +
            "if(ok){t.Write((document.getElementById('r').checked?'1':'0')+" +
            "(document.getElementById('s').checked?'1':'0')+" +
            "(document.getElementById('u').checked?'1':'0'));}" +
            "else{t.Write('000');}t.Close();window.close();}</script></head><body>" +
            "<p><b>Choose what to remove from this computer.</b><br/>" +
            dirHtml +
            "<br/>Nothing is deleted until you confirm. Leave all unchecked to keep everything.</p>" +
            "<label><input type='checkbox' id='r'/> Business records — sales, payments, day books (shop setup stays)</label>" +
            "<label><input type='checkbox' id='s'/> Shop setup — products, rates, staff, brand, empty books (also clears records)</label>" +
            "<label><input type='checkbox' id='u'/> Screen preferences — sidebar, amounts, window size, language</label>" +
            "<p style='margin-top:18px'>" +
            "<button onclick='finish(true)'>Continue</button>" +
            "<button onclick='finish(false)'>Keep all data</button></p>" +
            "</body></html>";
        var ts = fso.CreateTextFile(hta, true);
        ts.Write(html);
        ts.Close();
        // Prefer System32 mshta (Win10/11 PATH can resolve SysWOW64 first).
        // Window style 1 = normal; true = wait (same as the old Popup wait).
        var mshta =
            shell.ExpandEnvironmentStrings("%SystemRoot%") + "\\System32\\mshta.exe";
        if (!fso.FileExists(mshta)) {
            mshta = "mshta.exe";
        }
        shell.Run('"' + mshta + '" "' + hta + '"', 1, true);
        var delRecords = false;
        var delSetup = false;
        var delUi = false;
        if (fso.FileExists(out)) {
            var rf = fso.OpenTextFile(out, 1);
            var s = rf.AtEndOfStream ? "000" : rf.ReadAll();
            rf.Close();
            delRecords = s.charAt(0) === "1";
            delSetup = s.charAt(1) === "1";
            delUi = s.charAt(2) === "1";
        }
        try {
            if (fso.FileExists(hta)) {
                fso.DeleteFile(hta, true);
            }
            if (fso.FileExists(out)) {
                fso.DeleteFile(out, true);
            }
        } catch (e1) {}
        return { delRecords: delRecords, delSetup: delSetup, delUi: delUi };
    } catch (e2) {
        return empty;
    }
}
