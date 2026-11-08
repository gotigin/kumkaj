// Runs during MSI uninstall only (see build-msi.sh): one checkbox dialog for
// which local data to delete (via delete-books-data.ps1 -FromUninstall), then
// cleans Desktop / taskbar leftovers. Trading / UI / schema wipes call the
// still-installed exe (--uninstall-wipe) from that script.
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

        var installDir = "";
        try {
            installDir = Session.Property("INSTALLFOLDER");
        } catch (e2) {
            installDir = "";
        }
        if (!installDir) {
            return;
        }
        if (installDir.charAt(installDir.length - 1) !== "\\") {
            installDir += "\\";
        }
        var wipePs1 = installDir + "delete-books-data.ps1";
        if (!fso.FileExists(wipePs1)) {
            return;
        }

        // One dialog with three checkboxes (same as Start Menu "Delete books data").
        // Window style 1 = normal; true = wait. Exe is still on disk
        // (this action runs Before="RemoveFiles").
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
