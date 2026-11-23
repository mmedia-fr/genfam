// SPDX-License-Identifier: GPL-3.0-or-later
// Ajout et suppression de personnes (manage_persons), avec annulation de la
// dernière modification par la sauvegarde .bak.
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts

Window {
    id: w

    property var noyau
    property var lignes: []
    property string choix: ""

    title: "Ajouter / supprimer une personne"
    width: 760
    height: 540
    color: "#EAF1F7"

    function montrer() {
        remplir()
        show()
        raise()
    }

    function remplir() {
        lignes = JSON.parse(noyau.personnes(filtre.text, "Ahnentafel"))
        choix = ""
    }

    function informer(titre, texte) {
        info.title = titre
        info.text = texte
        info.open()
    }

    Connections {
        target: w.noyau
        function onRevisionChanged() { if (w.visible) w.remplir() }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 6
        Label { text: "GESTION DES PERSONNES"; font.bold: true; font.pixelSize: 14; color: "#234E70" }
        RowLayout {
            Layout.fillWidth: true
            Label { text: "Rechercher :" }
            TextField {
                id: filtre
                Layout.fillWidth: true
                placeholderText: "Nom, prénom, identifiant, lieu ou année"
                onTextChanged: minuterie.restart()
            }
            Timer { id: minuterie; interval: 250; onTriggered: w.remplir() }
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
                        Text { text: modelData.numero; Layout.preferredWidth: 60; horizontalAlignment: Text.AlignHCenter; elide: Text.ElideRight }
                        Text { text: modelData.id; Layout.preferredWidth: 100 }
                        Text { text: modelData.nom; Layout.fillWidth: true; elide: Text.ElideRight }
                    }
                    MouseArea { anchors.fill: parent; onClicked: w.choix = modelData.id }
                }
            }
        }
        RowLayout {
            Bouton { text: "➕ Ajouter un nom"; couleur: "#4F8A5B"; aide: "Créer une personne, reliée si besoin à son père et à sa mère"; onClicked: saisie.open() }
            Bouton {
                text: "🗑 Supprimer le nom sélectionné"; couleur: "#B83A3A"
                aide: "Supprimer la personne choisie dans la liste, avec ses liens (une copie .bak est faite avant)"
                onClicked: {
                    if (w.choix === "") { w.informer("Supprimer", "Sélectionnez une personne."); return }
                    confirmation.text = "Supprimer définitivement cette personne du GEDCOM ?\n\n" + w.noyau.nomDe(w.choix) + "\n" + w.choix
                    confirmation.open()
                }
            }
            Bouton {
                text: "↶ Annuler la dernière modification"; couleur: "#D8872F"
                aide: "Rétablir le fichier tel qu'il était avant la dernière écriture (copie .bak)"
                onClicked: {
                    const e = w.noyau.annuler()
                    w.informer("Annulation", e === "" ? "La dernière modification a été annulée." : e)
                }
            }
            Item { Layout.fillWidth: true }
            Bouton { text: "Fermer"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; aide: "Fermer cette fenêtre"; onClicked: w.close() }
        }
    }

    MessageDialog { id: info; buttons: MessageDialog.Ok }
    MessageDialog {
        id: confirmation
        title: "Confirmation"
        buttons: MessageDialog.Yes | MessageDialog.No
        onButtonClicked: (bouton) => {
            if (bouton !== MessageDialog.Yes)
                return
            const e = w.noyau.supprimerPersonne(w.choix)
            if (e !== "")
                w.informer("Erreur", e)
        }
    }

    Dialog {
        id: saisie
        title: "Nouvelle personne"
        modal: true
        anchors.centerIn: parent
        width: Math.min(w.width - 20, 640)
        height: Math.min(w.height - 20, 640)
        standardButtons: Dialog.Cancel
        onAboutToShow: {
            formulaire.vider()
            choixPere.effacer()
            choixMere.effacer()
            formulaire.focaliser()
        }
        ScrollView {
            id: defil
            anchors.fill: parent
            contentWidth: availableWidth
            ColumnLayout {
                width: defil.availableWidth
                spacing: 4
                FormulaireIdentite { id: formulaire; Layout.fillWidth: true }
                Label { text: "Père (facultatif)"; font.bold: true; Layout.topMargin: 6 }
                ChoixPersonne { id: choixPere; Layout.fillWidth: true; noyau: w.noyau }
                Label { text: "Mère (facultatif)"; font.bold: true }
                ChoixPersonne { id: choixMere; Layout.fillWidth: true; noyau: w.noyau }
                Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    color: "#425466"
                    text: "Tapez quelques lettres du nom du père ou de la mère, puis choisissez dans la liste : "
                          + "la nouvelle personne leur est reliée aussitôt. Pour relier ensuite conjoints et enfants, "
                          + "ouvrez sa fiche (bouton « Fiche et liens » de la fenêtre principale)."
                }
                Bouton {
                    Layout.fillWidth: true
                    Layout.topMargin: 8
                    text: "✓ VALIDER L'AJOUT"
                    couleur: "#4F8A5B"
                    aide: "Écrire la nouvelle personne dans le fichier GEDCOM (une copie .bak est faite avant)"
                    onClicked: {
                        const v = formulaire.valeurs()
                        v.pere = choixPere.choisi
                        v.mere = choixMere.choisi
                        const r = JSON.parse(w.noyau.ajouterPersonne(JSON.stringify(v)))
                        if (r.ok)
                            saisie.close()
                        w.informer("Ajouter", r.message)
                    }
                }
            }
        }
    }
}
