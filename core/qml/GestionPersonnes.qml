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
    property var parents: []

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
        lignes = JSON.parse(noyau.personnes("", "Ahnentafel"))
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
            Bouton { text: "➕ Ajouter un nom"; couleur: "#4F8A5B"; onClicked: saisie.open() }
            Bouton {
                text: "🗑 Supprimer le nom sélectionné"; couleur: "#B83A3A"
                onClicked: {
                    if (w.choix === "") { w.informer("Supprimer", "Sélectionnez une personne."); return }
                    confirmation.text = "Supprimer définitivement cette personne du GEDCOM ?\n\n" + w.noyau.nomDe(w.choix) + "\n" + w.choix
                    confirmation.open()
                }
            }
            Bouton {
                text: "↶ Annuler la dernière modification"; couleur: "#D8872F"
                onClicked: {
                    const e = w.noyau.annuler()
                    w.informer("Annulation", e === "" ? "La dernière modification a été annulée." : e)
                }
            }
            Item { Layout.fillWidth: true }
            Bouton { text: "Fermer"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: w.close() }
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
        width: 560
        height: Math.min(w.height - 20, 620)
        standardButtons: Dialog.Cancel
        onAboutToShow: {
            for (const c of [prenoms, nom, sexe, naissance, lieuNaissance, deces, lieuDeces])
                c.text = ""
            w.parents = [{ id: "", libelle: "(aucun)" }].concat(JSON.parse(w.noyau.choixParents()))
            pere.currentIndex = 0
            mere.currentIndex = 0
        }
        ScrollView {
            anchors.fill: parent
            contentWidth: availableWidth
            ColumnLayout {
                width: parent.width
                spacing: 3
                Label { text: "Prénoms" }
                TextField { id: prenoms; Layout.fillWidth: true }
                Label { text: "Nom" }
                TextField { id: nom; Layout.fillWidth: true }
                Label { text: "Sexe (M/F)" }
                TextField { id: sexe; Layout.preferredWidth: 60; maximumLength: 1 }
                Label { text: "Date de naissance" }
                TextField { id: naissance; Layout.fillWidth: true; placeholderText: "ex. 12 MAR 1950" }
                Label { text: "Lieu de naissance" }
                TextField { id: lieuNaissance; Layout.fillWidth: true }
                Label { text: "Date de décès" }
                TextField { id: deces; Layout.fillWidth: true }
                Label { text: "Lieu de décès" }
                TextField { id: lieuDeces; Layout.fillWidth: true }
                Label { text: "Père (facultatif)" }
                ComboBox { id: pere; Layout.fillWidth: true; model: w.parents; textRole: "libelle"; valueRole: "id" }
                Label { text: "Mère (facultatif)" }
                ComboBox { id: mere; Layout.fillWidth: true; model: w.parents; textRole: "libelle"; valueRole: "id" }
                Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    color: "#425466"
                    text: "Vous pouvez relier immédiatement la nouvelle personne à un père et/ou une mère déjà présents."
                }
                Bouton {
                    Layout.fillWidth: true
                    Layout.topMargin: 8
                    text: "✓ VALIDER L'AJOUT"
                    couleur: "#4F8A5B"
                    onClicked: {
                        const r = JSON.parse(w.noyau.ajouterPersonne(JSON.stringify({
                            prenoms: prenoms.text, nom: nom.text, sexe: sexe.text,
                            naissance: naissance.text, lieuNaissance: lieuNaissance.text,
                            deces: deces.text, lieuDeces: lieuDeces.text,
                            pere: pere.currentValue || "", mere: mere.currentValue || ""
                        })))
                        if (r.ok)
                            saisie.close()
                        w.informer(r.ok ? "Ajouter" : "Ajouter", r.message)
                    }
                }
            }
        }
    }
}
