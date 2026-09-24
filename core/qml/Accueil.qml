// Écran de démarrage : choix de la généalogie à charger (startup_gedcom_choice).
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Popup {
    id: accueil

    property var noyau

    signal ouvrir()
    signal creer()
    signal quitter()

    modal: true
    closePolicy: Popup.NoAutoClose
    anchors.centerIn: Overlay.overlay
    width: 620
    height: 400
    padding: 0

    background: Rectangle { color: "#EAF1F7"; border.color: "#B7C7D5" }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 78
            color: "#234E70"
            Column {
                anchors.centerIn: parent
                Text { text: "🌳  GÉNÉALOGIE"; color: "white"; font.pixelSize: 24; font.bold: true; anchors.horizontalCenter: parent.horizontalCenter }
                Text { text: "Version " + (noyau ? noyau.version : ""); color: "#DCEAF5"; font.pixelSize: 12; anchors.horizontalCenter: parent.horizontalCenter }
            }
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 25
            text: "Choisissez la généalogie à charger"
            color: "#234E70"; font.pixelSize: 17; font.bold: true
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 8
            text: "Sélectionnez un fichier GEDCOM existant, ou créez une nouvelle généalogie."
            color: "#425466"; font.pixelSize: 13
        }
        ColumnLayout {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 20
            Layout.preferredWidth: 360
            spacing: 10
            Bouton { Layout.fillWidth: true; text: "📂  Ouvrir un fichier GEDCOM"; onClicked: accueil.ouvrir() }
            Bouton { Layout.fillWidth: true; text: "📄  Créer un nouveau fichier GEDCOM"; couleur: "#4F8A5B"; onClicked: accueil.creer() }
            Bouton { Layout.fillWidth: true; Layout.topMargin: 12; text: "⏻  Quitter"; couleur: "#B83A3A"; onClicked: accueil.quitter() }
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 12
            text: "Vous pourrez ensuite ouvrir une autre généalogie depuis le menu Fichier."
            color: "#536878"; font.pixelSize: 11
        }
        Item { Layout.fillHeight: true }
    }
}
