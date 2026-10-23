import QtQuick
import Quickshell
import Quickshell.Io

// Watching Driftline's state file for the layout's short code beats polling
// `driftline msg layout`.
Tile {
    id: root

    label: "kbd"
    value: "—"
    clickable: true
    onClicked: Quickshell.execDetached(["driftline", "msg", "action", "switch-layout next"])

    FileView {
        path: `${Quickshell.env("XDG_RUNTIME_DIR")}/driftwm/state`
        watchChanges: true
        onFileChanged: reload()
        onLoaded: {
            const match = text().match(/^layout_short=(.*)$/m)
            root.value = match ? match[1].trim().toLowerCase() : "—"
        }
    }
}
