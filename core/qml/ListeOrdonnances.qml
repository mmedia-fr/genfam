// SPDX-License-Identifier: GPL-3.0-or-later
// Ordonnances d'une personne (données locales, hors GEDCOM), avec l'import des relevés
// FamilySearch : fichier CSV pour tout le fichier, ou relevé copié depuis une page.
// Sert à la fenêtre « Ordonnances » et à l'onglet de la fiche.
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts

ColumnLayout {
    id: o

    property var noyau
    property string personne: ""
    property var lignes: []
    property int choix: -1
    readonly property var listes: noyau ? JSON.parse(noyau.listesOrdonnances()) : ({ types: [], statuts: [] })

    function remplir() {
        lignes = noyau && personne !== "" ? JSON.parse(noyau.ordonnances(personne)) : []
        choix = -1
    }

    function informer(titre, texte) {
        boite.title = titre
        boite.text = texte
        boite.open()
    }

    function importer() { dialogueCsv.open() }
    function coller() { collage.open() }

    function compteRendu(titre, r) {
        remplir()
        informer(r.ok ? titre : "Import impossible", r.message)
    }

    spacing: 6

    Connections {
        target: o.noyau
        function onRevisionChanged() { o.remplir() }
    }

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
                model: o.lignes
                ScrollBar.vertical: ScrollBar {}
                delegate: Rectangle {
                    required property int index
                    required property var modelData
                    width: ListView.view.width
                    height: 26
                    color: index === o.choix ? "#B9D7EE" : (index % 2 === 0 ? "white" : "#F5F7F9")
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        Text { text: modelData.type; Layout.preferredWidth: 200; elide: Text.ElideRight }
                        Text { text: modelData.statut; Layout.preferredWidth: 120 }
                        Text { text: modelData.date; Layout.preferredWidth: 120 }
                        Text { text: modelData.temple; Layout.fillWidth: true; elide: Text.ElideRight }
                    }
                    MouseArea { anchors.fill: parent; onClicked: o.choix = index }
                }
            }
        }
    }
    Flow {
        Layout.fillWidth: true
        spacing: 6
        Bouton { text: "➕ Ajouter"; couleur: "#4F8A5B"; aide: "Noter une ordonnance à la main pour cette personne"; onClicked: saisie.open() }
        Bouton {
            text: "🗑 Supprimer"; couleur: "#B83A3A"; enabled: o.choix >= 0
            aide: "Supprimer l'ordonnance sélectionnée dans la liste"
            onClicked: { o.noyau.supprimerOrdonnance(o.personne, o.choix); o.remplir() }
        }
        Bouton {
            text: "📋 Coller un relevé…"; couleur: "#7256A3"
            aide: "Coller le texte des ordonnances copié sur FamilySearch : pour cette personne, ou un tableau pour tout le fichier"
            onClicked: o.coller()
        }
        Bouton {
            text: "📥 Importer un CSV FamilySearch…"; couleur: "#7256A3"
            aide: "Mettre à jour en une fois les ordonnances accomplies de tout le fichier, d'après l'export CSV de FamilySearch"
            onClicked: o.importer()
        }
    }

    MessageDialog { id: boite; buttons: MessageDialog.Ok }

    FileDialog {
        id: dialogueCsv
        title: "Importer les ordonnances accomplies (export CSV de FamilySearch)"
        fileMode: FileDialog.OpenFile
        nameFilters: ["Relevés CSV (*.csv *.txt *.tsv)", "Tous les fichiers (*)"]
        onAccepted: o.compteRendu("Import des ordonnances", JSON.parse(o.noyau.importerOrdonnances(selectedFile)))
    }

    Dialog {
        id: saisie
        title: "Nouvelle ordonnance"
        modal: true
        anchors.centerIn: Overlay.overlay
        width: 460
        standardButtons: Dialog.Save | Dialog.Cancel
        onAboutToShow: { genre.currentIndex = 0; statut.currentIndex = 0; date.text = ""; temple.text = "" }
        onAccepted: {
            o.noyau.ajouterOrdonnance(o.personne, JSON.stringify({ type: genre.currentText, statut: statut.currentText, date: date.text, temple: temple.text }))
            o.remplir()
        }
        ColumnLayout {
            anchors.fill: parent
            Label { text: "Type" }
            ComboBox { id: genre; Layout.fillWidth: true; model: o.listes.types }
            Label { text: "Statut" }
            ComboBox { id: statut; Layout.fillWidth: true; model: o.listes.statuts }
            Label { text: "Date" }
            TextField { id: date; Layout.fillWidth: true; placeholderText: "ex. 12 MAR 1990" }
            Label { text: "Temple" }
            TextField { id: temple; Layout.fillWidth: true }
        }
    }

    Dialog {
        id: collage
        title: "Coller un relevé d'ordonnances"
        modal: true
        anchors.centerIn: Overlay.overlay
        width: Math.min(680, Overlay.overlay ? Overlay.overlay.width - 40 : 680)
        height: Math.min(520, Overlay.overlay ? Overlay.overlay.height - 40 : 520)
        standardButtons: Dialog.Ok | Dialog.Cancel
        onAboutToShow: texte.text = ""
        onAccepted: o.compteRendu("Relevé collé", JSON.parse(o.noyau.collerReleve(o.personne, texte.text)))
        ColumnLayout {
            anchors.fill: parent
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: "#425466"
                text: "Collez ici les ordonnances copiées sur la page de la personne dans FamilySearch "
                      + "(intitulé, statut, date, temple), ou un tableau copié avec sa ligne d'en-tête "
                      + "(identifiant FamilySearch, ordonnance…) : un tableau met à jour tout le fichier."
            }
            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                background: Rectangle { color: "white"; border.color: "#C9D3DB" }
                TextArea { id: texte; wrapMode: TextEdit.NoWrap; placeholderText: "Baptême\nAccomplie\n12 mars 1990\nTemple de Paris…" }
            }
        }
    }
}
