// Fenêtre principale de GenFam : liste des personnes à gauche, informations à droite.
//
// Écrit pour Qt 6.4 (machine de développement) et compilé en 6.8 par l'intégration
// continue — d'où « Qt.labs.settings » plutôt que le module QtCore récent.
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Qt.labs.settings
import fr.mmedia.genfam

ApplicationWindow {
    id: fenetre

    width: 1180
    height: 740
    minimumWidth: 900
    minimumHeight: 600
    visible: true
    color: "#EAF1F7"
    title: "Généalogie V" + gen.version + " — " + (gen.fichier !== "" ? gen.fichier : "GEDCOM")

    /// Personne sélectionnée (identifiant GEDCOM) et vue du panneau de droite.
    property string courant: ""
    property string vue: "fiche"
    property string tri: "Alpha"
    property var lignes: []

    Genealogie { id: gen }

    Settings {
        id: reglages
        category: "fenetre"
        property alias x: fenetre.x
        property alias y: fenetre.y
        property alias largeur: fenetre.width
        property alias hauteur: fenetre.height
        property string dossier: ""
    }

    function rafraichir() {
        lignes = JSON.parse(gen.personnes(recherche.text, tri))
        if (courant !== "" && gen.nomDe(courant) === "")
            courant = ""
        afficher()
    }

    function afficher() {
        info.text = courant === "" ? "" : gen.presentation(courant, vue)
        info.cursorPosition = 0
    }

    function selectionner(id) {
        courant = id
        vue = "fiche"
        afficher()
        for (let i = 0; i < lignes.length; ++i) {
            if (lignes[i].id === id) {
                liste.currentIndex = i
                liste.positionViewAtIndex(i, ListView.Contain)
                break
            }
        }
    }

    function signaler(titre, texte) {
        boite.title = titre
        boite.text = texte
        boite.open()
    }

    function charger(url) {
        const e = gen.ouvrir(url)
        if (e !== "") {
            signaler("Erreur GEDCOM", e)
            return false
        }
        courant = ""
        recherche.text = ""
        rafraichir()
        return true
    }

    function exigerFichier() {
        if (gen.fichier === "") {
            signaler("Généalogie", "Créez ou ouvrez d'abord un fichier GEDCOM.")
            return false
        }
        return true
    }

    function exigerPersonne() {
        if (courant === "") {
            signaler("Généalogie", "Sélectionnez d'abord une personne.")
            return false
        }
        return true
    }

    /// Contrôle de fabrication (--smoke) : lit le fichier donné et exerce chaque vue.
    function controle() {
        if (gen.fichier === "")
            return "aucun fichier"
        const l = JSON.parse(gen.personnes("", "Ahnentafel"))
        if (l.length === 0)
            return "liste vide"
        for (const v of ["fiche", "famille", "ascendants", "descendants"])
            if (gen.presentation(l[0].id, v) === "")
                return "vue vide : " + v
        const p = JSON.parse(gen.pedigree(l[0].id, 3, 3, 0.85))
        if (!p.cartes || p.cartes.length === 0)
            return "pedigree vide"
        return "ok " + l.length + " personnes, " + p.cartes.length + " cartes"
    }

    Connections {
        target: gen
        function onRevisionChanged() { fenetre.rafraichir() }
    }

    Component.onCompleted: {
        if (typeof fichierInitial !== "undefined" && fichierInitial.toString() !== "")
            charger(fichierInitial)
        else
            accueil.open()
    }

    menuBar: MenuBar {
        Menu {
            title: "&Fichier"
            MenuItem { text: "Nouveau fichier GEDCOM"; onTriggered: dialogueNouveau.open() }
            MenuItem { text: "Ouvrir un fichier GEDCOM…"; onTriggered: dialogueOuvrir.open() }
            MenuItem { text: "Enregistrer sous…"; enabled: gen.fichier !== ""; onTriggered: dialogueEnregistrerSous.open() }
            MenuSeparator {}
            MenuItem { text: "Fermer GEDCOM"; enabled: gen.fichier !== ""; onTriggered: confirmerFermeture.open() }
            MenuSeparator {}
            MenuItem { text: "Quitter"; onTriggered: confirmerQuitter.open() }
        }
        Menu {
            title: "&FamilySearch"
            MenuItem { text: "Connexion et import…"; onTriggered: fenetreFs.montrer() }
        }
        Menu {
            title: "&Aide"
            MenuItem { text: "À propos"; onTriggered: signaler("À propos", "Généalogie — version " + gen.version + "\n\nProgramme conçu par Claude Boisseau.\nPortage Rust / Qt 6 et liaison FamilySearch : M-Media.") }
        }
    }

    header: ColumnLayout {
        spacing: 0
        Rectangle {
            Layout.fillWidth: true
            Layout.margins: 10
            Layout.bottomMargin: 5
            Layout.preferredHeight: 50
            color: "#234E70"
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 14
                spacing: 12
                Text { text: "🌳  GÉNÉALOGIE"; color: "white"; font.pixelSize: 23; font.bold: true }
                Text { text: "V" + gen.version; color: "#DCEAF5"; font.pixelSize: 12; Layout.alignment: Qt.AlignBottom; Layout.bottomMargin: 10 }
                Text { text: "Gestion et exploration de vos fichiers GEDCOM"; color: "#DCEAF5"; font.pixelSize: 12; Layout.alignment: Qt.AlignBottom; Layout.bottomMargin: 10 }
                Item { Layout.fillWidth: true }
                Text {
                    visible: gen.connecte
                    text: "● FamilySearch"
                    color: "#9FE0A8"; font.pixelSize: 12; font.bold: true
                    Layout.rightMargin: 14
                }
            }
        }
        Flow {
            Layout.fillWidth: true
            Layout.leftMargin: 8
            Layout.rightMargin: 8
            spacing: 6
            Bouton { text: "📄 Nouveau GEDCOM"; couleur: "#4F8A5B"; onClicked: dialogueNouveau.open() }
            Bouton { text: "📂 Ouvrir GEDCOM"; onClicked: dialogueOuvrir.open() }
            Bouton { text: "✖ Fermer GEDCOM"; couleur: "#B83A3A"; enabled: gen.fichier !== ""; onClicked: confirmerFermeture.open() }
            Bouton { text: "🌐 FamilySearch"; onClicked: fenetreFs.montrer() }
            Bouton { text: "📜 Ordonnances"; couleur: "#7256A3"; onClicked: if (exigerPersonne()) fenetreOrd.montrer(courant) }
            Bouton { text: "🔢 Ahnentafel"; couleur: "#D8872F"; onClicked: if (exigerFichier()) fenetreRacine.montrer() }
            Bouton { text: "👤 Ajouter / supprimer"; couleur: "#4F8A5B"; onClicked: if (exigerFichier()) fenetreGestion.montrer() }
            Bouton { text: "💾 Exporter"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: if (exigerPersonne()) dialogueExport.open() }
            Bouton { text: "⏻ Fermer le programme"; couleur: "#B83A3A"; onClicked: confirmerQuitter.open() }
        }
        Frame {
            Layout.fillWidth: true
            Layout.margins: 8
            RowLayout {
                anchors.fill: parent
                spacing: 6
                Label { text: "Recherche :"; font.bold: true }
                TextField {
                    id: recherche
                    Layout.preferredWidth: 380
                    placeholderText: "Nom, identifiant ou lieu"
                    onAccepted: fenetre.rafraichir()
                }
                Bouton { text: "🔎 Rechercher"; onClicked: fenetre.rafraichir() }
                Bouton { text: "Effacer"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: { recherche.text = ""; fenetre.rafraichir() } }
                Label { text: "  Trier par :"; font.bold: true }
                ComboBox {
                    id: choixTri
                    model: ["Alpha", "Numérotation", "Ahnentafel"]
                    onActivated: { fenetre.tri = currentText; fenetre.rafraichir() }
                }
                Item { Layout.fillWidth: true }
                Label { text: gen.fichier !== "" ? gen.fichier : "Aucun fichier chargé"; color: "#425466" }
            }
        }
    }

    SplitView {
        anchors.fill: parent
        anchors.margins: 8

        Frame {
            SplitView.preferredWidth: 440
            SplitView.minimumWidth: 300
            padding: 1
            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 30
                    color: "#234E70"
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        spacing: 0
                        Text { text: "N°"; color: "white"; font.bold: true; Layout.preferredWidth: 70; horizontalAlignment: Text.AlignHCenter }
                        Text { text: "ID"; color: "white"; font.bold: true; Layout.preferredWidth: 90 }
                        Text { text: "Nom (" + fenetre.lignes.length + ")"; color: "white"; font.bold: true; Layout.fillWidth: true }
                    }
                }
                ListView {
                    id: liste
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: fenetre.lignes
                    boundsBehavior: Flickable.StopAtBounds
                    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AlwaysOn }
                    keyNavigationEnabled: true
                    focus: true
                    onCurrentIndexChanged: {
                        if (currentIndex >= 0 && currentIndex < fenetre.lignes.length && fenetre.lignes[currentIndex].id !== fenetre.courant) {
                            fenetre.courant = fenetre.lignes[currentIndex].id
                            fenetre.vue = "fiche"
                            fenetre.afficher()
                        }
                    }
                    delegate: Rectangle {
                        required property int index
                        required property var modelData
                        width: ListView.view.width
                        height: 28
                        color: modelData.id === fenetre.courant ? "#B9D7EE" : (index % 2 === 0 ? "white" : "#F5F7F9")
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 6
                            spacing: 0
                            Text { text: modelData.numero; Layout.preferredWidth: 70; horizontalAlignment: Text.AlignHCenter; elide: Text.ElideRight; color: "#253746" }
                            Text { text: modelData.id; Layout.preferredWidth: 90; elide: Text.ElideRight; color: "#253746" }
                            Text { text: modelData.nom; Layout.fillWidth: true; elide: Text.ElideRight; color: "#253746" }
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: { liste.currentIndex = index; fenetre.selectionner(modelData.id) }
                            onDoubleClicked: fenetrePedigree.montrer(modelData.id)
                        }
                    }
                }
            }
        }

        ColumnLayout {
            SplitView.fillWidth: true
            spacing: 5
            Label { text: "Informations généalogiques"; font.bold: true; font.pixelSize: 14; Layout.leftMargin: 8 }
            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.leftMargin: 8
                background: Rectangle { color: "white"; border.color: "#C9D3DB" }
                TextArea {
                    id: info
                    readOnly: true
                    selectByMouse: true
                    textFormat: TextEdit.RichText
                    wrapMode: TextEdit.Wrap
                    leftPadding: 18
                    rightPadding: 18
                    topPadding: 14
                    font.pixelSize: 13
                    background: null
                    onLinkActivated: (lien) => fenetre.selectionner(lien)
                    HoverHandler { cursorShape: info.hoveredLink !== "" ? Qt.PointingHandCursor : Qt.IBeamCursor }
                }
            }
            Flow {
                Layout.fillWidth: true
                Layout.leftMargin: 8
                spacing: 6
                Bouton { text: "👤 Fiche complète"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: if (exigerPersonne()) { fenetre.vue = "fiche"; fenetre.afficher() } }
                Bouton { text: "👨‍👩‍👧 Famille"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: if (exigerPersonne()) { fenetre.vue = "famille"; fenetre.afficher() } }
                Bouton { text: "⬆ Ascendants"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: if (exigerPersonne()) { fenetre.vue = "ascendants"; fenetre.afficher() } }
                Bouton { text: "⬇ Descendants"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: if (exigerPersonne()) { fenetre.vue = "descendants"; fenetre.afficher() } }
                Bouton { text: "🌳 Pedigree"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: if (exigerPersonne()) fenetrePedigree.montrer(fenetre.courant) }
                Bouton { text: "📜 Ordonnances"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; onClicked: if (exigerPersonne()) fenetreOrd.montrer(fenetre.courant) }
                Bouton { text: "🌐 Importer de FamilySearch"; onClicked: if (exigerPersonne()) fenetreFs.montrer() }
            }
        }
    }

    footer: Rectangle {
        height: 26
        color: "#DDE5EC"
        border.color: "#B7C7D5"
        Label {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: 8
            text: gen.occupe ? "⏳ " + gen.message : gen.resume
            color: "#253746"
        }
    }

    Accueil {
        id: accueil
        noyau: gen
        onOuvrir: dialogueOuvrir.open()
        onCreer: dialogueNouveau.open()
        onQuitter: Qt.quit()
    }

    ChoixRacine { id: fenetreRacine; noyau: gen; onChoisie: (id) => { fenetre.tri = "Ahnentafel"; choixTri.currentIndex = 2; fenetre.rafraichir() } }
    GestionPersonnes { id: fenetreGestion; noyau: gen }
    Ordonnances { id: fenetreOrd; noyau: gen }
    Pedigree { id: fenetrePedigree; noyau: gen; onCentre: (id) => fenetre.selectionner(id) }
    FamilySearch { id: fenetreFs; noyau: gen; personne: fenetre.courant }

    function dossierInitial() {
        return reglages.dossier !== "" ? reglages.dossier : ""
    }

    function retenirDossier(url) {
        const s = url.toString()
        reglages.dossier = s.substring(0, s.lastIndexOf("/"))
    }

    FileDialog {
        id: dialogueOuvrir
        title: "Ouvrir un fichier GEDCOM"
        fileMode: FileDialog.OpenFile
        nameFilters: ["Fichiers GEDCOM (*.ged *.GED)", "Tous les fichiers (*)"]
        currentFolder: fenetre.dossierInitial()
        onAccepted: {
            fenetre.retenirDossier(selectedFile)
            if (fenetre.charger(selectedFile))
                accueil.close()
        }
    }
    FileDialog {
        id: dialogueNouveau
        title: "Créer un nouveau fichier GEDCOM"
        fileMode: FileDialog.SaveFile
        defaultSuffix: "ged"
        nameFilters: ["Fichiers GEDCOM (*.ged)"]
        currentFolder: fenetre.dossierInitial()
        onAccepted: {
            fenetre.retenirDossier(selectedFile)
            const e = gen.nouveau(selectedFile)
            if (e !== "") {
                fenetre.signaler("Erreur", e)
                return
            }
            accueil.close()
            fenetre.courant = ""
            fenetre.rafraichir()
            fenetre.signaler("Nouveau GEDCOM", "Le nouveau fichier GEDCOM a été créé.\n\n" + gen.cheminCourant()
                             + "\n\nVous pouvez maintenant ajouter la première personne avec « Ajouter / supprimer ».")
        }
    }
    FileDialog {
        id: dialogueEnregistrerSous
        title: "Enregistrer le GEDCOM sous…"
        fileMode: FileDialog.SaveFile
        defaultSuffix: "ged"
        nameFilters: ["Fichiers GEDCOM (*.ged)"]
        onAccepted: {
            const e = gen.enregistrerSous(selectedFile)
            fenetre.signaler(e === "" ? "Enregistrer sous" : "Erreur GEDCOM", e === "" ? "GEDCOM enregistré sous :\n\n" + gen.cheminCourant() : e)
        }
    }
    FileDialog {
        id: dialogueExport
        title: "Enregistrer le résultat"
        fileMode: FileDialog.SaveFile
        defaultSuffix: "txt"
        nameFilters: ["Fichier texte (*.txt)"]
        onAccepted: {
            const e = gen.exporter(selectedFile, fenetre.courant, fenetre.vue)
            fenetre.signaler("Export", e === "" ? "Résultat enregistré." : e)
        }
    }

    MessageDialog {
        id: boite
        buttons: MessageDialog.Ok
    }
    MessageDialog {
        id: confirmerFermeture
        title: "Fermer le GEDCOM"
        text: "Voulez-vous fermer le fichier :\n\n" + gen.cheminCourant() + " ?"
        buttons: MessageDialog.Yes | MessageDialog.No
        onButtonClicked: (bouton) => { if (bouton === MessageDialog.Yes) { gen.fermer(); fenetre.courant = ""; fenetre.rafraichir() } }
    }
    MessageDialog {
        id: confirmerQuitter
        title: "Fermer le programme"
        text: "Voulez-vous vraiment fermer le programme de généalogie ?"
        buttons: MessageDialog.Yes | MessageDialog.No
        onButtonClicked: (bouton) => { if (bouton === MessageDialog.Yes) Qt.quit() }
    }
}
