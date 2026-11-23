// SPDX-License-Identifier: GPL-3.0-or-later
// Choix d'une personne par saisie : quelques lettres du nom, du prénom ou une année
// suffisent ; les personnes qui conviennent s'affichent avec leurs dates, pour ne pas
// confondre deux homonymes. Remplace la liste déroulante de toutes les personnes.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: c

    property var noyau
    /// Identifiant choisi (vide : aucun) et son nom.
    property string choisi: ""
    property string choisiNom: ""
    /// Identifiants à ne pas proposer (la personne elle-même, par exemple).
    property var exclus: []
    property string invite: "Tapez un nom, un prénom ou une année…"
    property var resultats: []
    property int total: 0

    signal choix(string id)

    function definir(id, nom) {
        choisi = id
        choisiNom = nom
        saisie.text = ""
        resultats = []
        total = 0
    }

    function effacer() {
        definir("", "")
        saisie.forceActiveFocus()
    }

    function chercher() {
        if (!noyau || saisie.text.trim().length < 2) {
            resultats = []
            total = 0
            return
        }
        const r = JSON.parse(noyau.chercherPersonnes(saisie.text, 60))
        resultats = r.liste.filter(p => c.exclus.indexOf(p.id) < 0)
        total = r.total
        liste.currentIndex = resultats.length > 0 ? 0 : -1
    }

    function prendre(i) {
        if (i < 0 || i >= resultats.length)
            return
        const p = resultats[i]
        definir(p.id, p.nom)
        c.choix(p.id)
    }

    spacing: 2

    RowLayout {
        visible: c.choisi !== ""
        Layout.fillWidth: true
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 30
            color: "white"
            border.color: "#9AAEBB"
            radius: 3
            Text {
                anchors.fill: parent
                anchors.leftMargin: 8
                verticalAlignment: Text.AlignVCenter
                text: c.choisiNom + "   [" + c.choisi + "]"
                font.bold: true
                color: "#173A54"
                elide: Text.ElideRight
            }
        }
        Bouton {
            text: "✖ Changer"
            couleur: "#D9E4ED"
            couleurTexte: "#234E70"
            aide: "Retirer ce choix pour en saisir un autre (ou laisser la place vide)"
            onClicked: c.effacer()
        }
    }
    TextField {
        id: saisie
        visible: c.choisi === ""
        Layout.fillWidth: true
        placeholderText: c.invite
        onTextChanged: minuterie.restart()
        onAccepted: c.prendre(liste.currentIndex)
        Keys.onDownPressed: liste.incrementCurrentIndex()
        Keys.onUpPressed: liste.decrementCurrentIndex()
    }
    Timer { id: minuterie; interval: 250; onTriggered: c.chercher() }
    Frame {
        visible: c.choisi === "" && c.resultats.length > 0
        Layout.fillWidth: true
        Layout.preferredHeight: Math.min(6, c.resultats.length) * 24 + 4
        padding: 1
        ListView {
            id: liste
            anchors.fill: parent
            clip: true
            model: c.resultats
            ScrollBar.vertical: ScrollBar {}
            delegate: Rectangle {
                required property int index
                required property var modelData
                width: ListView.view.width
                height: 24
                color: index === liste.currentIndex ? "#B9D7EE" : (index % 2 === 0 ? "white" : "#F5F7F9")
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    Text { text: modelData.nomListe; font.bold: true; Layout.fillWidth: true; elide: Text.ElideRight; color: "#253746" }
                    Text { text: modelData.dates; color: "#425466" }
                    Text { text: modelData.lieu; color: "#5A6872"; Layout.maximumWidth: 140; elide: Text.ElideRight }
                    Text { text: modelData.id; color: "#5A6872"; Layout.preferredWidth: 64 }
                }
                MouseArea {
                    anchors.fill: parent
                    onClicked: c.prendre(index)
                }
            }
        }
    }
    Label {
        visible: c.choisi === "" && saisie.text.trim().length >= 2
        text: c.total === 0 ? "Aucune personne ne correspond."
              : (c.total > c.resultats.length ? c.total + " personnes correspondent : précisez la saisie (prénom, année…)." : "")
        color: "#5A6872"
        font.pixelSize: 11
    }
}
