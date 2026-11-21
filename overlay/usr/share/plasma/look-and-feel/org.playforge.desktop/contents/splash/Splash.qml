import QtQuick

Rectangle {
    id: root
    color: "#070B14"

    property int stage

    Image {
        anchors.fill: parent
        source: "file:///usr/share/wallpapers/Playforge/contents/images/2560x1440.png"
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
    }

    Rectangle {
        anchors.fill: parent
        color: "#B8070B14"
    }

    Column {
        anchors.centerIn: parent
        spacing: root.height * 0.022

        Image {
            anchors.horizontalCenter: parent.horizontalCenter
            source: "images/playforge-logo.svg"
            sourceSize.width: Math.min(root.width, root.height) * 0.22
            sourceSize.height: Math.min(root.width, root.height) * 0.22
            fillMode: Image.PreserveAspectFit
            asynchronous: true
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "PLAYFORGE"
            color: "#F8FAFC"
            font.pixelSize: Math.max(18, root.height * 0.042)
            font.bold: true
            font.letterSpacing: font.pixelSize * 0.32
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "GAME ON. OUT OF THE BOX."
            color: "#22D3EE"
            font.pixelSize: Math.max(9, root.height * 0.013)
            font.letterSpacing: font.pixelSize * 0.42
        }
    }

    Rectangle {
        id: track

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.height * 0.12
        width: Math.min(root.width * 0.32, root.height * 0.55)
        height: Math.max(3, root.height * 0.0035)
        radius: height / 2
        color: "#1E293B"

        Rectangle {
            height: parent.height
            radius: parent.radius
            width: parent.width * Math.min(1, Math.max(0.04, root.stage / 5))
            color: "#22D3EE"

            Behavior on width {
                NumberAnimation {
                    duration: 300
                    easing.type: Easing.OutCubic
                }
            }
        }
    }
}
