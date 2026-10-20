// Kumkaj Courier MSI data prompt (JScript custom actions).
// com.kumkaj.courier is replaced by build-msi.sh (com.kumkaj.courier[.debug]).
// Keep is the safe default: only an explicit "Yes" deletes anything.
// All errors are swallowed so install/uninstall can never be blocked.

function kumkajDataDir() {
    var shell = new ActiveXObject("WScript.Shell");
    return shell.ExpandEnvironmentStrings("%APPDATA%") + "\\com.kumkaj.courier";
}

function kumkajAskDelete(when) {
    try {
        var fso = new ActiveXObject("Scripting.FileSystemObject");
        var dir = kumkajDataDir();
        if (!fso.FolderExists(dir)) {
            return;
        }
        var shell = new ActiveXObject("WScript.Shell");
        var text;
        if (when === "uninstall") {
            text = "Also delete your Kumkaj Courier books data?\n\n" + dir +
                "\n\nYes = delete books forever\nNo = keep books (recommended)";
        } else {
            text = "Books data from a previous install was found:\n\n" + dir +
                "\n\nDelete it before installing?\n\nYes = delete books forever\nNo = keep books and reuse them (recommended)";
        }
        var answer = shell.Popup(text, 0, "Kumkaj Courier", 4 + 48);
        if (answer === 6) { // Yes
            fso.DeleteFolder(dir, true);
        }
    } catch (e) {
        // Never block install/uninstall on a prompt failure.
    }
}

function AskDeleteOnInstall() {
    kumkajAskDelete("install");
}

function AskDeleteOnUninstall() {
    kumkajAskDelete("uninstall");
}
