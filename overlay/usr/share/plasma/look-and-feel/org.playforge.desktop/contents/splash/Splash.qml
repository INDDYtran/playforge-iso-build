import QtQuick

Rectangle {
    id: root
    color: "#0C0A08"

    property int stage

    // Warm ember glow behind the mark
    Rectangle {
        anchors.centerIn: parent
        width: Math.min(root.width, root.height) * 0.55
        height: width
        radius: width / 2
        color: "#1A0F06"
        opacity: 0.55
    }

    Image {
        anchors.fill: parent
        source: "file:///usr/share/wallpapers/Playforge/contents/images/2560x1440.png"
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        opacity: 0.35
    }

    Rectangle {
        anchors.fill: parent
        color: "#CC0C0A08"
    }

    Column {
        anchors.centerIn: parent
        spacing: root.height * 0.018

        Image {
            anchors.horizontalCenter: parent.horizontalCenter
            source: "images/playforge-logo.svg"
            sourceSize.width: Math.min(root.width, root.height) * 0.18
            sourceSize.height: Math.min(root.width, root.height) * 0.18
            fillMode: Image.PreserveAspectFit
            asynchronous: true
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "PLAYFORGE"
            color: "#F5EFE4"
            font.pixelSize: Math.max(20, root.height * 0.046)
            font.bold: true
            font.letterSpacing: font.pixelSize * 0.26
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "GAMING. FORGED."
            color: "#F59E0B"
            font.pixelSize: Math.max(10, root.height * 0.014)
            font.letterSpacing: font.pixelSize * 0.34
        }
    }

    Rectangle {
        id: track

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.height * 0.11
        width: Math.min(root.width * 0.26, root.height * 0.45)
        height: Math.max(3, root.height * 0.003)
        radius: height / 2
        color: "#2A2218"

        Rectangle {
            height: parent.height
            radius: parent.radius
            width: parent.width * Math.min(1, Math.max(0.04, root.stage / 5))
            color: "#F59E0B"

            Behavior on width {
                NumberAnimation {
                    duration: 260
                    easing.type: Easing.OutCubic
                }
            }
        }
    }
}