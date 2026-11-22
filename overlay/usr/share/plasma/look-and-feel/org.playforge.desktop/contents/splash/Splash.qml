import QtQuick

// Playforge – Splash «ASCII Brutal».
// Reines Schwarz, Monospace, harte Kanten; der Ladebalken besteht aus
// einzelnen Blöcken wie eine Terminal-Fortschrittsanzeige.
Rectangle {
    id: root
    color: "#000000"

    property int stage
    property int segments: 32
    readonly property int lit: Math.max(3, Math.round(Math.min(1, stage / 5) * segments))

    // Eckmarken statt runder Rahmen.
    Repeater {
        model: 4
        Rectangle {
            width: Math.max(28, root.width * 0.026)
            height: 2
            color: "#1793D1"
            x: (index % 2 === 0) ? root.width * 0.06 : root.width * 0.94 - width
            y: (index < 2) ? root.height * 0.16 : root.height * 0.84
        }
    }
    Repeater {
        model: 4
        Rectangle {
            width: 2
            height: Math.max(28, root.width * 0.026)
            color: "#1793D1"
            x: (index % 2 === 0) ? root.width * 0.06 : root.width * 0.94 - width
            y: (index < 2) ? root.height * 0.16 : root.height * 0.84 - height
        }
    }

    Column {
        anchors.centerIn: parent
        spacing: root.height * 0.026

        Image {
            anchors.horizontalCenter: parent.horizontalCenter
            source: "images/playforge-logo.svg"
            sourceSize.width: Math.min(root.width, root.height) * 0.19
            sourceSize.height: Math.min(root.width, root.height) * 0.19
            fillMode: Image.PreserveAspectFit
            asynchronous: true
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "PLAYFORGE"
            color: "#F2F2EC"
            font.family: "monospace"
            font.pixelSize: Math.max(22, root.height * 0.052)
            font.bold: true
            font.letterSpacing: font.pixelSize * 0.08
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "ARCH LINUX / GAMING / ASCII BRUTAL"
            color: "#1793D1"
            font.family: "monospace"
            font.pixelSize: Math.max(9, root.height * 0.0125)
            font.letterSpacing: font.pixelSize * 0.16
        }
    }

    Row {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.height * 0.12
        spacing: Math.max(2, root.height * 0.0025)

        Repeater {
            model: root.segments
            Rectangle {
                width: Math.max(3, Math.min(root.width * 0.008, 14))
                height: Math.max(4, root.height * 0.006)
                color: index < root.lit ? "#1793D1" : "#1C1C1C"
            }
        }
    }

    Text {
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.leftMargin: root.width * 0.06
        anchors.bottomMargin: root.height * 0.048
        text: "> boot"
        color: "#565650"
        font.family: "monospace"
        font.pixelSize: Math.max(9, root.height * 0.012)
    }

    Text {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.rightMargin: root.width * 0.06
        anchors.bottomMargin: root.height * 0.048
        text: "tty1 / kde plasma"
        color: "#565650"
        font.family: "monospace"
        font.pixelSize: Math.max(9, root.height * 0.012)
    }

    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.height * 0.06
        width: Math.max(10, root.height * 0.016)
        height: width
        color: "#F2F2EC"
        opacity: (Math.floor(stage / 2) % 2) === 0 ? 0.0 : 1.0
    }
}
