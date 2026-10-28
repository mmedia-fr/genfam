// SPDX-License-Identifier: GPL-3.0-or-later
// Recherche d'ascendance assistée : parents manquants génération par génération,
// candidats trouvés dans l'arbre FamilySearch ou relevés dans les archives, contrôle
// de cohérence, puis validation humaine avant toute écriture dans le fichier.
//
// D'après le cahier des charges « Projet Appli Généalogie » de Claude Boisseau.
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Qt.labs.settings

Window {
    id: w

    property var noyau
    /// Personne sélectionnée dans la fenêtre principale.
    property string personne: ""
    /// Demande d'aller sur une personne dans la fenêtre principale.
    signal aller(string id)

    property var etat: ({ active: false })
    property string choisi: ""
    property var travail: ({})

    title: "Recherche d'ascendance"
    width: 1120
    height: 780
    minimumWidth: 920
    minimumHeight: 620
    color: "#EAF1F7"

    Settings {
        id: reglages
        category: "recherche"
        property int generations: 10
    }

    function montrer() {
        generations.value = reglages.generations
        rafraichir()
        show()
        raise()
    }

    function rafraichir() {
        if (!noyau)
            return
        etat = JSON.parse(noyau.recherche())
        const liste = etat.active ? etat.travaux : []
        if (choisi !== "" && !liste.some(t => t.personne === choisi))
            choisi = ""
        if (choisi === "" && liste.length > 0) {
            const ouvert = liste.find(t => !t.complet && !t.sansIssue)
            choisi = (ouvert || liste[0]).personne
        }
        travail = choisi !== "" ? JSON.parse(noyau.rechercheTravail(choisi)) : ({})
    }

    function choisir(id) {
        choisi = id
        travail = JSON.parse(noyau.rechercheTravail(id))
    }

    function signaler(texte) {
        if (texte === "")
            return
        boite.text = texte
        boite.open()
    }

    function lancer() {
        reglages.generations = generations.value
        const r = JSON.parse(noyau.rechercheLancer(personne, generations.value))
        choisi = ""
        rafraichir()
        signaler(r.message)
    }

    function decider(id, action) {
        const r = JSON.parse(noyau.rechercheDecider(choisi, id, action))
        if (!r.ok || action === "accepter")
            signaler(r.message)
        rafraichir()
    }

    function saisir() {
        const champs = {
            role: role.currentValue, prenoms: prenoms.text, nom: nomSaisi.text,
            naissance: naissance.text, lieuNaissance: lieuNaissance.text,
            deces: deces.text, lieuDeces: lieuDeces.text,
            organisme: organisme.text, reference: reference.text, commune: communeActe.text,
            url: adresse.text, extrait: extrait.text
        }
        const r = JSON.parse(noyau.rechercheSaisir(choisi, JSON.stringify(champs)))
        signaler(r.message)
        if (r.ok) {
            for (const champ of [prenoms, nomSaisi, naissance, lieuNaissance, deces, lieuDeces, reference, adresse, extrait])
                champ.text = ""
        }
        rafraichir()
    }

    function couleurEtat(e) {
        return e === "accepte" ? "#E3F2E6" : e === "rejete" ? "#F1F1F1" : e === "a_verifier" ? "#FFF4E0" : "white"
    }

    Connections {
        target: w.noyau
        function onRevisionChanged() { if (w.visible) w.rafraichir() }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 6

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 50
            color: "#234E70"
            Text { anchors.verticalCenter: parent.verticalCenter; x: 16; text: "🔎  RECHERCHE D'ASCENDANCE"; color: "white"; font.pixelSize: 20; font.bold: true }
        }

        GroupBox {
            title: "Personne de départ"
            Layout.fillWidth: true
            Layout.leftMargin: 10
            Layout.rightMargin: 10
            RowLayout {
                anchors.fill: parent
                spacing: 8
                Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    textFormat: Text.StyledText
                    text: w.etat.active
                          ? "Recherche en cours depuis <b>" + w.etat.departNom + "</b> : " + w.etat.aFaire
                            + " personne(s) à compléter, jusqu'à la génération " + w.etat.generationsMax + "."
                          : (w.personne !== "" && w.noyau
                             ? "Aucune recherche en cours. Départ proposé : <b>" + w.noyau.nomDe(w.personne) + "</b>."
                             : "Aucune recherche en cours. Sélectionnez une personne dans la liste principale.")
                }
                Label { text: "Générations :"; font.bold: true }
                SpinBox { id: generations; from: 2; to: 20; value: 10 }
                Bouton {
                    text: w.etat.active && w.etat.depart === w.personne ? "Actualiser" : "Lancer depuis la personne sélectionnée"
                    enabled: w.personne !== ""
                    onClicked: {
                        if (w.etat.active && w.etat.depart !== w.personne)
                            remplacer.open()
                        else
                            w.lancer()
                    }
                }
            }
        }

        SplitView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.leftMargin: 10
            Layout.rightMargin: 10

            Frame {
                SplitView.preferredWidth: 390
                SplitView.minimumWidth: 300
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
                            spacing: 4
                            Text { text: "Gén."; color: "white"; font.bold: true; Layout.preferredWidth: 36 }
                            Text { text: "Personne"; color: "white"; font.bold: true; Layout.fillWidth: true }
                            Text { text: "Manque"; color: "white"; font.bold: true; Layout.preferredWidth: 96 }
                        }
                    }
                    ListView {
                        id: liste
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        model: w.etat.active ? w.etat.travaux : []
                        boundsBehavior: Flickable.StopAtBounds
                        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }
                        delegate: Rectangle {
                            id: ligne
                            required property int index
                            required property var modelData
                            width: ListView.view.width
                            height: 38
                            color: ligne.modelData.personne === w.choisi ? "#B9D7EE" : (index % 2 === 0 ? "white" : "#F5F7F9")
                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 6
                                anchors.rightMargin: 6
                                spacing: 4
                                Text { text: ligne.modelData.generation; Layout.preferredWidth: 36; color: "#253746" }
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 0
                                    Text { text: ligne.modelData.nom; Layout.fillWidth: true; elide: Text.ElideRight; color: "#253746"; font.bold: ligne.modelData.ouverts > 0 }
                                    Text {
                                        text: (ligne.modelData.naissance !== "" ? "° " + ligne.modelData.naissance : "")
                                              + (ligne.modelData.ouverts > 0 ? "   " + ligne.modelData.ouverts + " candidat(s) à examiner" : "")
                                        font.pixelSize: 10
                                        color: "#5B6B7A"
                                    }
                                }
                                Text {
                                    Layout.preferredWidth: 96
                                    elide: Text.ElideRight
                                    text: ligne.modelData.complet ? "✓ complet" : (ligne.modelData.sansIssue ? "sans issue" : ligne.modelData.manque)
                                    color: ligne.modelData.complet ? "#2E7D32" : (ligne.modelData.sansIssue ? "#8A8A8A" : "#B35C00")
                                }
                            }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: w.choisir(ligne.modelData.personne)
                                onDoubleClicked: w.aller(ligne.modelData.personne)
                            }
                        }
                    }
                }
            }

            ScrollView {
                SplitView.fillWidth: true
                contentWidth: availableWidth
                clip: true

                ColumnLayout {
                    width: parent.width
                    spacing: 8
                    visible: w.travail.personne !== undefined

                    Label {
                        Layout.fillWidth: true
                        Layout.leftMargin: 10
                        Layout.topMargin: 6
                        wrapMode: Text.WordWrap
                        textFormat: Text.StyledText
                        font.pixelSize: 15
                        text: w.travail.personne !== undefined
                              ? "<b>" + w.travail.nom + "</b> [" + w.travail.personne + "] — génération " + w.travail.generation : ""
                    }
                    Label {
                        Layout.fillWidth: true
                        Layout.leftMargin: 10
                        wrapMode: Text.WordWrap
                        color: "#425466"
                        text: w.travail.personne === undefined ? "" :
                              "Naissance : " + (w.travail.naissance || "?") + (w.travail.lieuNaissance ? ", " + w.travail.lieuNaissance : "")
                              + "\nPère : " + (w.travail.pere || "à trouver") + "   —   Mère : " + (w.travail.mere || "à trouver")
                              + (w.travail.fsid ? "\nFamilySearch : " + w.travail.fsid : "")
                    }
                    Label {
                        Layout.fillWidth: true
                        Layout.leftMargin: 10
                        visible: w.travail.prioritaire === false
                        wrapMode: Text.WordWrap
                        color: "#B35C00"
                        text: "⚠ Personne née il y a moins de 100 ans : ses actes de naissance et de mariage peuvent ne pas être publiés en ligne."
                    }

                    GroupBox {
                        title: "Chercher les parents"
                        Layout.fillWidth: true
                        Layout.leftMargin: 10
                        Layout.rightMargin: 10
                        ColumnLayout {
                            anchors.fill: parent
                            spacing: 6
                            RowLayout {
                                Bouton {
                                    text: "🌐 Interroger l'arbre FamilySearch"
                                    enabled: w.noyau && w.noyau.connecte && !w.noyau.occupe && (w.travail.fsid || "") !== ""
                                             && (w.travail.manque || []).length > 0
                                    onClicked: w.noyau.rechercheFamilySearch(w.choisi)
                                }
                                BusyIndicator { running: w.noyau && w.noyau.occupe; visible: running; Layout.preferredHeight: 30; Layout.preferredWidth: 30 }
                                Label {
                                    Layout.fillWidth: true
                                    wrapMode: Text.WordWrap
                                    color: "#5B6B7A"
                                    text: !(w.noyau && w.noyau.connecte) ? "Connexion par le menu FamilySearch."
                                          : (w.travail.fsid || "") === "" ? "Pas d'identifiant FamilySearch pour cette personne." : ""
                                }
                            }
                            Label {
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                                text: "Recherche guidée : le site s'ouvre dans le navigateur, déjà rempli. Relevez ensuite l'acte trouvé ci-dessous."
                            }
                            Flow {
                                Layout.fillWidth: true
                                spacing: 6
                                Repeater {
                                    model: w.travail.liens || []
                                    delegate: Bouton {
                                        required property var modelData
                                        text: modelData.libelle
                                        couleur: "#D9E4ED"
                                        couleurTexte: "#234E70"
                                        onClicked: Qt.openUrlExternally(modelData.url)
                                    }
                                }
                            }
                        }
                    }

                    GroupBox {
                        title: "Candidats"
                        Layout.fillWidth: true
                        Layout.leftMargin: 10
                        Layout.rightMargin: 10
                        ColumnLayout {
                            anchors.fill: parent
                            spacing: 6
                            Label {
                                visible: (w.travail.candidats || []).length === 0
                                text: "Aucun candidat pour l'instant."
                                color: "#5B6B7A"
                            }
                            Repeater {
                                model: w.travail.candidats || []
                                delegate: Frame {
                                    id: carte
                                    required property var modelData
                                    Layout.fillWidth: true
                                    background: Rectangle { color: w.couleurEtat(carte.modelData.etat); border.color: "#B7C7D5"; radius: 3 }
                                    ColumnLayout {
                                        width: parent.width
                                        spacing: 3
                                        RowLayout {
                                            Layout.fillWidth: true
                                            Text {
                                                Layout.fillWidth: true
                                                wrapMode: Text.WordWrap
                                                font.bold: true
                                                color: "#173A54"
                                                text: carte.modelData.roleLibelle + " : " + carte.modelData.nom
                                                      + (carte.modelData.fsid ? "  (" + carte.modelData.fsid + ")" : "")
                                            }
                                            Text {
                                                text: carte.modelData.etatLibelle + " — " + carte.modelData.score + "/100"
                                                font.bold: true
                                                color: carte.modelData.etat === "a_verifier" ? "#B35C00" : carte.modelData.etat === "accepte" ? "#2E7D32" : "#425466"
                                            }
                                        }
                                        Text {
                                            Layout.fillWidth: true
                                            wrapMode: Text.WordWrap
                                            color: "#425466"
                                            text: "Naissance : " + (carte.modelData.naissance || "?") + (carte.modelData.lieuNaissance ? ", " + carte.modelData.lieuNaissance : "")
                                                  + (carte.modelData.deces ? "   Décès : " + carte.modelData.deces + (carte.modelData.lieuDeces ? ", " + carte.modelData.lieuDeces : "") : "")
                                                  + (carte.modelData.provenance === "familysearch" ? "   — proposé par FamilySearch" : "   — relevé à la main")
                                        }
                                        Repeater {
                                            model: carte.modelData.motifs
                                            delegate: Text { required property var modelData; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: "#2E7D32"; text: "✓ " + modelData }
                                        }
                                        Repeater {
                                            model: carte.modelData.alertes
                                            delegate: Text { required property var modelData; Layout.fillWidth: true; wrapMode: Text.WordWrap; color: "#B35C00"; text: "⚠ " + modelData }
                                        }
                                        Repeater {
                                            model: carte.modelData.sources
                                            delegate: Text {
                                                required property var modelData
                                                Layout.fillWidth: true
                                                wrapMode: Text.WordWrap
                                                textFormat: Text.StyledText
                                                color: "#253746"
                                                text: "📄 " + (modelData.description || "Source")
                                                      + (modelData.url ? " — <a href=\"" + modelData.url + "\">voir</a>" : "")
                                                      + (modelData.extrait ? "<br/><i>« " + modelData.extrait + " »</i>" : "")
                                                onLinkActivated: (lien) => Qt.openUrlExternally(lien)
                                            }
                                        }
                                        Text {
                                            visible: carte.modelData.etat === "accepte" && carte.modelData.personne !== ""
                                            text: "Écrit dans le fichier : " + carte.modelData.personne
                                            color: "#2E7D32"
                                        }
                                        RowLayout {
                                            visible: carte.modelData.ouvert
                                            Bouton {
                                                text: "✔ Accepter"
                                                couleur: "#4F8A5B"
                                                enabled: carte.modelData.placeLibre
                                                onClicked: confirmer.demander(carte.modelData)
                                            }
                                            Bouton { text: "✖ Rejeter"; couleur: "#B83A3A"; onClicked: w.decider(carte.modelData.id, "rejeter") }
                                        }
                                        Bouton {
                                            visible: carte.modelData.etat === "rejete"
                                            text: "Réexaminer"
                                            couleur: "#D9E4ED"
                                            couleurTexte: "#234E70"
                                            onClicked: w.decider(carte.modelData.id, "attente")
                                        }
                                    }
                                }
                            }
                        }
                    }

                    GroupBox {
                        title: "Relever un acte"
                        Layout.fillWidth: true
                        Layout.leftMargin: 10
                        Layout.rightMargin: 10
                        enabled: (w.travail.manque || []).length > 0
                        GridLayout {
                            anchors.fill: parent
                            columns: 4
                            columnSpacing: 8
                            rowSpacing: 4
                            Label { text: "Lien :"; font.bold: true }
                            ComboBox {
                                id: role
                                Layout.fillWidth: true
                                textRole: "libelle"
                                valueRole: "cle"
                                model: (w.travail.manque || []).map(m => ({ cle: m, libelle: m === "pere" ? "Père" : "Mère" }))
                            }
                            Item { Layout.columnSpan: 2; Layout.fillWidth: true }
                            Label { text: "Prénoms :" }
                            TextField { id: prenoms; Layout.fillWidth: true }
                            Label { text: "Nom :" }
                            TextField { id: nomSaisi; Layout.fillWidth: true }
                            Label { text: "Naissance :" }
                            TextField { id: naissance; Layout.fillWidth: true; placeholderText: "ex. 12 MAR 1850" }
                            Label { text: "Lieu :" }
                            TextField { id: lieuNaissance; Layout.fillWidth: true }
                            Label { text: "Décès :" }
                            TextField { id: deces; Layout.fillWidth: true }
                            Label { text: "Lieu :" }
                            TextField { id: lieuDeces; Layout.fillWidth: true }
                            Label { text: "Archives :"; font.bold: true }
                            TextField { id: organisme; Layout.fillWidth: true; placeholderText: "ex. Archives départementales de Lot-et-Garonne" }
                            Label { text: "Cote / référence :"; font.bold: true }
                            TextField { id: reference; Layout.fillWidth: true; placeholderText: "ex. 4 E 12/3, vue 45" }
                            Label { text: "Commune de l'acte :" }
                            TextField { id: communeActe; Layout.fillWidth: true }
                            Label { text: "Adresse de l'image :" }
                            TextField { id: adresse; Layout.fillWidth: true; placeholderText: "https://…" }
                            Label { text: "Extrait :" }
                            TextField { id: extrait; Layout.columnSpan: 3; Layout.fillWidth: true; placeholderText: "ex. fils de Louis MARTIN, cultivateur, et de Jeanne BLANC" }
                            Item { Layout.columnSpan: 3; Layout.fillWidth: true }
                            Bouton { text: "➕ Ajouter le candidat"; couleur: "#4F8A5B"; onClicked: w.saisir() }
                        }
                    }

                    RowLayout {
                        Layout.leftMargin: 10
                        Bouton {
                            text: w.travail.sansIssue ? "Rouvrir cette branche" : "Marquer sans issue (aucune source pour l'instant)"
                            couleur: "#D9E4ED"
                            couleurTexte: "#234E70"
                            onClicked: w.signaler(JSON.parse(w.noyau.rechercheSansIssue(w.choisi, !w.travail.sansIssue)).message)
                        }
                        Bouton { text: "Aller à la fiche"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: w.aller(w.choisi) }
                    }

                    GroupBox {
                        title: "Tentatives"
                        Layout.fillWidth: true
                        Layout.leftMargin: 10
                        Layout.rightMargin: 10
                        Layout.bottomMargin: 10
                        visible: (w.travail.tentatives || []).length > 0
                        ColumnLayout {
                            anchors.fill: parent
                            spacing: 2
                            Repeater {
                                model: w.travail.tentatives || []
                                delegate: Text {
                                    required property var modelData
                                    Layout.fillWidth: true
                                    wrapMode: Text.WordWrap
                                    color: "#425466"
                                    text: modelData.date + " — " + modelData.source + " : " + modelData.resultat
                                }
                            }
                        }
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 10
            Layout.rightMargin: 10
            Layout.bottomMargin: 10
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: "#173A54"
                text: w.noyau ? w.noyau.message : ""
            }
            Bouton { text: "Fermer"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: w.close() }
        }
    }

    MessageDialog {
        id: boite
        title: "Recherche d'ascendance"
        buttons: MessageDialog.Ok
    }
    MessageDialog {
        id: remplacer
        title: "Recherche d'ascendance"
        text: "Une recherche est en cours depuis " + (w.etat.departNom || "") + ".\n\nLa remplacer par une recherche depuis la personne sélectionnée ? Les candidats déjà relevés seront perdus."
        buttons: MessageDialog.Yes | MessageDialog.No
        onButtonClicked: (bouton) => { if (bouton === MessageDialog.Yes) w.lancer() }
    }
    MessageDialog {
        id: confirmer
        property var candidat: null
        function demander(c) {
            candidat = c
            text = "Écrire dans le fichier : " + c.nom + ", " + c.roleLibelle.toLowerCase() + " de " + w.travail.nom + " ?"
                   + (c.alertes.length > 0 ? "\n\nPoints à vérifier :\n– " + c.alertes.join("\n– ") : "")
                   + "\n\nUne copie .bak du fichier est faite avant l'écriture."
            open()
        }
        title: "Accepter le lien"
        buttons: MessageDialog.Yes | MessageDialog.No
        onButtonClicked: (bouton) => { if (bouton === MessageDialog.Yes && candidat) w.decider(candidat.id, "accepter") }
    }
}
