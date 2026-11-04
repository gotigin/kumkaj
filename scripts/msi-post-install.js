// Runs after a successful MSI install (see build-msi.sh): pin the Start Menu
// shortcut to the taskbar (best-effort on Windows 10/11) and launch the app.
// Desktop + Start Menu shortcuts are created by WiX; this script does not
// create those. Placeholders __APP_LABEL__ / __BIN_NAME__ are substituted.
// Must never fail the install (Return=ignore).
function PostInstall() {
    try {
        var shell = new ActiveXObject("WScript.Shell");
        var fso = new ActiveXObject("Scripting.FileSystemObject");
        var install = Session.Property("INSTALLFOLDER");
        if (!install) {
            return;
        }
        if (install.charAt(install.length - 1) !== "\\") {
            install += "\\";
        }
        var exe = install + "__BIN_NAME__.exe";
        if (!fso.FileExists(exe)) {
            return;
        }

        // Prefer the Start Menu .lnk WiX just wrote (has icon + working dir).
        var startLnk =
            shell.SpecialFolders("Programs") +
            "\\__APP_LABEL__\\__APP_LABEL__.lnk";
        if (!fso.FileExists(startLnk)) {
            startLnk = exe;
        }

        pinToTaskbar(shell, fso, startLnk);

        // Launch only when the user saw the UI (skip msiexec /qn).
        var ui = parseInt(Session.Property("UILevel"), 10);
        if (isNaN(ui) || ui > 4) {
            shell.Run('"' + exe + '"', 1, false);
        }
    } catch (e) {
        // ignore — install already succeeded
    }
}

function pinToTaskbar(shell, fso, targetPath) {
    try {
        var folderPath = fso.GetParentFolderName(targetPath);
        var fileName = fso.GetFileName(targetPath);
        var app = new ActiveXObject("Shell.Application");
        var folder = app.NameSpace(folderPath);
        if (!folder) {
            return;
        }
        var item = folder.ParseName(fileName);
        if (!item) {
            return;
        }
        var verbs = item.Verbs();
        var i;
        for (i = 0; i < verbs.Count; i++) {
            var raw = "" + verbs.Item(i).Name;
            var name = raw.replace(/&/g, "").toLowerCase();
            // EN "Pin to taskbar"; tolerate other locales that mention taskbar.
            if (
                name.indexOf("pin to taskbar") >= 0 ||
                (name.indexOf("taskbar") >= 0 && name.indexOf("pin") >= 0) ||
                (name.indexOf("taskleiste") >= 0 && name.indexOf("anheft") >= 0)
            ) {
                verbs.Item(i).DoIt();
                return;
            }
        }
    } catch (e1) {}

    // Fallback used by older Windows: drop a .lnk into the pinned TaskBar folder.
    try {
        if (!fso.FileExists(targetPath) || !/\.lnk$/i.test(targetPath)) {
            return;
        }
        var pinDir =
            shell.ExpandEnvironmentStrings("%APPDATA%") +
            "\\Microsoft\\Internet Explorer\\Quick Launch\\User Pinned\\TaskBar";
        if (!fso.FolderExists(pinDir)) {
            fso.CreateFolder(pinDir);
        }
        var dest = pinDir + "\\" + fso.GetFileName(targetPath);
        fso.CopyFile(targetPath, dest, true);
    } catch (e2) {}
}
