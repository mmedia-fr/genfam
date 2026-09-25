// SPDX-License-Identifier: GPL-3.0-or-later
// Bouton coloré, repris de l'habillage de la version Python (styles ttk « Primary »,
// « Green », « Orange », « Purple », « Red », « Light »).
import QtQuick
import QtQuick.Controls

Button {
    id: b

    property color couleur: "#2E75B6"
    property color couleurTexte: "white"

    font.bold: true
    font.pixelSize: 12
    leftPadding: 11
    rightPadding: 11
    topPadding: 7
    bottomPadding: 7

    contentItem: Text {
        text: b.text
        font: b.font
        color: b.enabled ? b.couleurTexte : "#A0A0A0"
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
    background: Rectangle {
        implicitHeight: 32
        radius: 3
        color: !b.enabled ? Qt.lighter(b.couleur, 1.3)
             : b.down ? Qt.darker(b.couleur, 1.3)
             : b.hovered ? Qt.darker(b.couleur, 1.12) : b.couleur
    }
}
