// Runs during MSI uninstall only (see build-msi.sh): asks whether to also
// delete the books data folder, and deletes it on Yes.
// __APP_ID__ is replaced by build-msi.sh (prod/dev data folders differ).
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
        var answer = shell.Popup(
            "Also delete your __APP_LABEL__ books data?\n\n" +
                dir +
                "\n\nYes - delete books, backups and settings forever." +
                "\nNo - keep them; a reinstall will find them again.",
            60,
            "Uninstall __APP_LABEL__",
            4 + 32 + 4096
        );
        if (answer == 6) {
            fso.DeleteFolder(dir, true);
        }
    } catch (e) {
        // Keep the data and let the uninstall continue.
    }
}
