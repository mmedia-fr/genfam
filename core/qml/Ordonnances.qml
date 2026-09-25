// SPDX-License-Identifier: GPL-3.0-or-later
// Ordonnances d'une personne (show_ordinances) — données locales, hors GEDCOM.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: w

    property var noyau
    property string personne: ""
    property var lignes: []
    property int choix: -1
    readonly property var listes: noyau ? JSON.parse(noyau.listesOrdonnances()) : ({ types: [], statuts: [] })

    title: "Ordonnances — " + (noyau ? noyau.nomDe(personne) : "")
    width: 820
    height: 520
    color: "#EAF1F7"

    function montrer(id) {
        personne = id
        remplir()
        show()
        raise()
    }

    function remplir() {
        lignes = JSON.parse(noyau.ordonnances(personne))
        choix = -1
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 6
        Label { text: "ORDONNANCES — " + (w.noyau ? w.noyau.nomDe(w.personne) : ""); font.bold: true; font.pixelSize: 14; color: "#234E70" }
        Label { text: "Identifiant : " + w.personne + "    |    Données locales, enregistrées à côté du fichier GEDCOM."; color: "#425466" }
        Frame {
            Layout.fillWidth: true
            Layout.fillHeight: true
            padding: 1
            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 28
                    color: "#234E70"
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        Text { text: "Ordonnance"; color: "white"; font.bold: true; Layout.preferredWidth: 200 }
                        Text { text: "Statut"; color: "white"; font.bold: true; Layout.preferredWidth: 120 }
                        Text { text: "Date"; color: "white"; font.bold: true; Layout.preferredWidth: 120 }
                        Text { text: "Temple"; color: "white"; font.bold: true; Layout.fillWidth: true }
                    }
                }
                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: w.lignes
                    delegate: Rectangle {
                        required property int index
                        required property var modelData
                        width: ListView.view.width
                        height: 26
                        color: index === w.choix ? "#B9D7EE" : (index % 2 === 0 ? "white" : "#F5F7F9")
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 6
                            Text { text: modelData.type; Layout.preferredWidth: 200; elide: Text.ElideRight }
                            Text { text: modelData.statut; Layout.preferredWidth: 120 }
                            Text { text: modelData.date; Layout.preferredWidth: 120 }
                            Text { text: modelData.temple; Layout.fillWidth: true; elide: Text.ElideRight }
                        }
                        MouseArea { anchors.fill: parent; onClicked: w.choix = index }
                    }
                }
            }
        }
        RowLayout {
            Bouton { text: "➕ Ajouter"; couleur: "#4F8A5B"; onClicked: saisie.open() }
            Bouton {
                text: "🗑 Supprimer"; couleur: "#B83A3A"; enabled: w.choix >= 0
                onClicked: { w.noyau.supprimerOrdonnance(w.personne, w.choix); w.remplir() }
            }
            Item { Layout.fillWidth: true }
            Bouton { text: "Fermer"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: w.close() }
        }
    }

    Dialog {
        id: saisie
        title: "Nouvelle ordonnance"
        modal: true
        anchors.centerIn: parent
        width: 460
        standardButtons: Dialog.Save | Dialog.Cancel
        onAboutToShow: { genre.currentIndex = 0; statut.currentIndex = 0; date.text = ""; temple.text = "" }
        onAccepted: {
            w.noyau.ajouterOrdonnance(w.personne, JSON.stringify({ type: genre.currentText, statut: statut.currentText, date: date.text, temple: temple.text }))
            w.remplir()
        }
        ColumnLayout {
            anchors.fill: parent
            Label { text: "Type" }
            ComboBox { id: genre; Layout.fillWidth: true; model: w.listes.types }
            Label { text: "Statut" }
            ComboBox { id: statut; Layout.fillWidth: true; model: w.listes.statuts }
            Label { text: "Date" }
            TextField { id: date; Layout.fillWidth: true }
            Label { text: "Temple" }
            TextField { id: temple; Layout.fillWidth: true }
        }
    }
}
