// Post-install helpers for the MSI (see build-msi.sh).
// Placeholders __APP_LABEL__ / __BIN_NAME__ are substituted at build time.
// Must never fail the install (Return=ignore).
//
// Pin runs after InstallFinalize (shortcuts already exist).
// Launch runs only when the user ticks "Launch …" on the Finish page —
// never from InstallFinalize (that started the app before Finish).

function PostInstallPin() {
    try {
        var shell = new ActiveXObject("WScript.Shell");
        var fso = new ActiveXObject("Scripting.FileSystemObject");
        var startLnk = resolveStartShortcut(shell, fso);
        if (startLnk) {
            pinToTaskbar(shell, fso, startLnk);
        }
    } catch (e) {
        // ignore — install already succeeded
    }
}

function PostInstallLaunch() {
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
        // Finish-page checkbox / silent installs: only launch when UI asked for it.
        var ui = parseInt(Session.Property("UILevel"), 10);
        if (!isNaN(ui) && ui <= 4) {
            return;
        }
        shell.Run('"' + exe + '"', 1, false);
    } catch (e) {
        // ignore
    }
}

function resolveStartShortcut(shell, fso) {
    var startLnk =
        shell.SpecialFolders("Programs") +
        "\\__APP_LABEL__\\__APP_LABEL__.lnk";
    if (fso.FileExists(startLnk)) {
        return startLnk;
    }
    var install = Session.Property("INSTALLFOLDER");
    if (!install) {
        return null;
    }
    if (install.charAt(install.length - 1) !== "\\") {
        install += "\\";
    }
    var exe = install + "__BIN_NAME__.exe";
    if (fso.FileExists(exe)) {
        return exe;
    }
    return null;
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
