// Choix de la personne N° 1 de la numérotation Ahnentafel (choose_ahnentafel_root).
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: w

    property var noyau
    property var lignes: []
    property string choix: ""

    signal choisie(string id)

    title: "Numérotation Ahnentafel — Choisir la personne N° 1"
    width: 760
    height: 600
    minimumWidth: 650
    minimumHeight: 500
    modality: Qt.ApplicationModal
    color: "#EAF1F7"

    function montrer() {
        filtre.text = ""
        choix = noyau.racine
        remplir()
        show()
        raise()
        filtre.forceActiveFocus()
    }

    function remplir() {
        lignes = JSON.parse(noyau.personnes(filtre.text, "Alpha"))
    }

    function valider() {
        if (choix === "")
            return
        noyau.definirRacine(choix)
        w.choisie(choix)
        close()
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 15
        spacing: 8
        Label { text: "CHOIX DE LA PERSONNE DE RÉFÉRENCE"; font.bold: true; font.pixelSize: 14; color: "#234E70" }
        Label {
            text: "Sélectionnez la personne qui recevra le N° 1.\nLa numérotation Ahnentafel sera ensuite calculée à partir de cette personne."
            color: "#425466"
        }
        RowLayout {
            Label { text: "Rechercher :" }
            TextField { id: filtre; Layout.fillWidth: true; onTextChanged: w.remplir(); onAccepted: w.valider() }
        }
        Frame {
            Layout.fillWidth: true
            Layout.fillHeight: true
            padding: 1
            ListView {
                anchors.fill: parent
                clip: true
                model: w.lignes
                ScrollBar.vertical: ScrollBar {}
                delegate: Rectangle {
                    required property int index
                    required property var modelData
                    width: ListView.view.width
                    height: 26
                    color: modelData.id === w.choix ? "#B9D7EE" : (index % 2 === 0 ? "white" : "#F5F7F9")
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        Text { text: modelData.id; Layout.preferredWidth: 110 }
                        Text { text: modelData.nom; Layout.fillWidth: true; elide: Text.ElideRight }
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: w.choix = modelData.id
                        onDoubleClicked: { w.choix = modelData.id; w.valider() }
                    }
                }
            }
        }
        RowLayout {
            Bouton { text: "✓ Définir comme N° 1"; couleur: "#4F8A5B"; enabled: w.choix !== ""; onClicked: w.valider() }
            Bouton { text: "↶ Réinitialiser"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: { w.noyau.reinitialiserRacine(); w.close() } }
            Item { Layout.fillWidth: true }
            Bouton { text: "Annuler"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: w.close() }
        }
    }
}
