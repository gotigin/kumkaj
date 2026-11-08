// Runs during MSI uninstall only (see build-msi.sh): one checkbox dialog for
// which local data to delete (via delete-books-data.ps1 -FromUninstall), then
// cleans Desktop / taskbar leftovers. Trading / UI / schema wipes call the
// still-installed exe (--uninstall-wipe) from that script.
// Placeholders __APP_ID__, __APP_LABEL__, __BIN_NAME__ are replaced by build-msi.sh.
// The whole body is guarded: any failure here must never block uninstall.
//
// Scheduled on InstallUISequence (not InstallExecuteSequence): Settings → Apps
// often runs a basic UI uninstall where execute-sequence UI is suppressed, and
// INSTALLFOLDER is sometimes empty in that path — both made the three-checkbox
// dialog vanish silently after the “one dialog” fix shipped.
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
        // Still offer the dialog when the books folder exists; nothing to ask
        // about if there is no local data.
        if (!fso.FolderExists(dir)) {
            return;
        }

        var installDir = resolveInstallDir(shell, fso);
        if (!installDir) {
            return;
        }
        var wipePs1 = installDir + "delete-books-data.ps1";
        if (!fso.FileExists(wipePs1)) {
            return;
        }

        // One dialog with three checkboxes (same as Start Menu "Delete books data").
        // Window style 1 = normal; true = wait. Exe is still on disk
        // (UISequence runs before RemoveFiles).
        try {
            shell.Run(
                'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "' +
                    wipePs1 +
                    '" -FromUninstall',
                1,
                true
            );
        } catch (e3) {
            // Keep the data and let the uninstall continue.
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

function resolveInstallDir(shell, fso) {
    var candidates = [];
    try {
        var fromSession = Session.Property("INSTALLFOLDER");
        if (fromSession) {
            candidates.push(fromSession);
        }
    } catch (e2) {}
    // Per-user MSI default path — used when Session.Property("INSTALLFOLDER")
    // is empty during Settings → Apps uninstall (common with wixl MSIs).
    try {
        candidates.push(
            shell.ExpandEnvironmentStrings("%LOCALAPPDATA%") +
                "\\Programs\\__APP_LABEL__\\"
        );
    } catch (e3) {}

    for (var i = 0; i < candidates.length; i++) {
        var dir = candidates[i];
        if (!dir) {
            continue;
        }
        if (dir.charAt(dir.length - 1) !== "\\") {
            dir += "\\";
        }
        if (fso.FileExists(dir + "delete-books-data.ps1")) {
            return dir;
        }
    }
    return "";
}
