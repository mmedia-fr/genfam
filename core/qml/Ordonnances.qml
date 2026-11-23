// SPDX-License-Identifier: GPL-3.0-or-later
// Ordonnances d'une personne (show_ordinances) — données locales, hors GEDCOM.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: w

    property var noyau
    property string personne: ""
    readonly property alias liste: liste

    title: "Ordonnances — " + (noyau ? noyau.nomDe(personne) : "")
    width: 860
    height: 540
    color: "#EAF1F7"

    function montrer(id) {
        personne = id
        liste.remplir()
        show()
        raise()
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 6
        Label { text: "ORDONNANCES — " + (w.noyau ? w.noyau.nomDe(w.personne) : ""); font.bold: true; font.pixelSize: 14; color: "#234E70" }
        Label { text: "Identifiant : " + w.personne + "    |    Données locales, enregistrées à côté du fichier GEDCOM."; color: "#425466" }
        ListeOrdonnances {
            id: liste
            Layout.fillWidth: true
            Layout.fillHeight: true
            noyau: w.noyau
            personne: w.personne
        }
        RowLayout {
            Item { Layout.fillWidth: true }
            Bouton { text: "Fermer"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; aide: "Fermer cette fenêtre"; onClicked: w.close() }
        }
    }
}
