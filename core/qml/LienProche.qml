// SPDX-License-Identifier: GPL-3.0-or-later
// Ajout d'un proche (père, mère, conjoint, enfant) depuis la fiche d'une personne :
// on cherche d'abord s'il figure déjà dans le fichier, pour l'« Associer » ; à défaut,
// on le crée en tapant son nom. Pour un enfant ou un conjoint, l'union se choisit.
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts

Window {
    id: w

    property var noyau
    property var fiche: ({})
    /// « pere », « mere », « conjoint » ou « enfant ».
    property string relation: "conjoint"
    /// Unions proposées : `[{famille, libelle}]` (famille vide : union nouvelle).
    property var unions: []
    property var resultats: []
    property int total: 0
    property string choisi: ""
    property var semblables: []

    /// Le lien est écrit : message à montrer, identifiant du proche.
    signal fait(string message, string id)

    readonly property string libelleRelation: ({
        pere: "le père", mere: "la mère", conjoint: "un conjoint", enfant: "un enfant"
    })[relation] || ""

    title: "Ajouter " + libelleRelation + " — " + (fiche.nomAffiche || "")
    width: 720
    height: 640
    minimumWidth: 600
    minimumHeight: 520
    modality: Qt.ApplicationModal
    color: "#EAF1F7"

    /// Prépare la fenêtre sans l'afficher (le contrôle de fabrication s'en sert).
    function preparer(f, rel, mode) {
        fiche = f
        relation = rel
        const opts = []
        if (rel === "enfant") {
            for (const u of f.unions)
                opts.push({ famille: u.famille, libelle: "Union " + u.libelle })
            if (!f.unions.some(u => !u.conjoint))
                opts.push({ famille: "", libelle: "Union nouvelle, sans conjoint (autre parent inconnu)" })
        } else if (rel === "conjoint") {
            opts.push({ famille: "", libelle: "Union nouvelle" })
            for (const u of f.unions)
                if (!u.conjoint)
                    opts.push({ famille: u.famille, libelle: "Compléter l'union " + u.famille + ", sans conjoint (" + u.enfants + " enfant(s))" })
        }
        unions = opts
        // Une seule union possible, ou une seule union existante : elle est proposée
        // d'office. Plusieurs unions : le choix est exigé.
        choixUnion.currentIndex = (rel === "enfant" && f.unions.length > 1) ? -1 : 0
        recherche.text = ""
        resultats = []
        total = 0
        choisi = ""
        formulaire.vider()
        formulaire.definirSexe(rel === "pere" ? "M" : rel === "mere" ? "F"
                               : rel === "conjoint" ? (f.sexe === "M" ? "F" : f.sexe === "F" ? "M" : "") : "")
        semblables = []
        onglets.currentIndex = mode === "creer" ? 1 : 0
    }

    function ouvrir(f, rel, mode) {
        preparer(f, rel, mode)
        show()
        raise()
        requestActivate()
        if (onglets.currentIndex === 0)
            recherche.forceActiveFocus()
        else
            formulaire.focaliser()
    }

    function signaler(titre, texte) {
        boite.title = titre
        boite.text = texte
        boite.open()
    }

    function exclus() {
        const e = [fiche.id]
        const deja = relation === "conjoint" ? fiche.conjoints : relation === "enfant" ? fiche.enfants : fiche.parents
        for (const p of deja || [])
            e.push(p.id)
        return e
    }

    function chercher() {
        if (recherche.text.trim().length < 2) {
            resultats = []
            total = 0
            return
        }
        const r = JSON.parse(noyau.chercherPersonnes(recherche.text, 80))
        const e = exclus()
        resultats = r.liste.filter(p => e.indexOf(p.id) < 0)
        total = r.total
        choisi = resultats.length === 1 ? resultats[0].id : ""
    }

    function chercherSemblables() {
        const v = formulaire.valeurs()
        const q = (v.prenoms + " " + v.nom).trim()
        if (q.length < 2) {
            semblables = []
            return
        }
        const e = exclus()
        semblables = JSON.parse(noyau.chercherPersonnes(q, 8)).liste.filter(p => e.indexOf(p.id) < 0)
    }

    /// Famille choisie ; `null` si le choix de l'union est exigé et n'a pas été fait.
    function familleChoisie() {
        if (relation !== "enfant" && relation !== "conjoint")
            return ""
        if (choixUnion.currentIndex < 0 || choixUnion.currentIndex >= unions.length)
            return null
        return unions[choixUnion.currentIndex].famille
    }

    /// « Jean dupont » → prénoms « Jean », nom « DUPONT » ; les mots en capitales
    /// forment le nom, à défaut le dernier mot.
    function decouper(texte) {
        const mots = texte.trim().split(/\s+/).filter(m => m !== "")
        if (mots.length === 0)
            return { prenoms: "", nom: "" }
        const capitales = mots.filter(m => m.length > 1 && m === m.toUpperCase() && m !== m.toLowerCase())
        let nom, prenoms
        if (capitales.length > 0 && capitales.length < mots.length) {
            nom = capitales
            prenoms = mots.filter(m => capitales.indexOf(m) < 0)
        } else if (mots.length === 1) {
            nom = mots
            prenoms = []
        } else {
            nom = [mots[mots.length - 1]]
            prenoms = mots.slice(0, mots.length - 1)
        }
        const majuscule = m => m.charAt(0).toUpperCase() + m.slice(1)
        return { prenoms: prenoms.map(majuscule).join(" "), nom: nom.join(" ").toUpperCase() }
    }

    function versCreation() {
        const d = decouper(recherche.text)
        const sexe = formulaire.valeurs().sexe
        formulaire.remplir({ prenoms: d.prenoms, nom: d.nom, sexe: sexe })
        onglets.currentIndex = 1
        chercherSemblables()
        formulaire.focaliser()
    }

    function terminer(r, id) {
        if (!r.ok) {
            signaler("Ajout impossible", r.message)
            return
        }
        w.fait(r.message, id)
        close()
    }

    function associer(id) {
        const f = familleChoisie()
        if (f === null) {
            signaler("Union à choisir", "Cette personne a plusieurs unions : choisissez celle à laquelle rattacher l'enfant.")
            return
        }
        if (id === "") {
            signaler("Associer", "Choisissez d'abord une personne dans la liste.")
            return
        }
        terminer(JSON.parse(noyau.lierProche(fiche.id, relation, id, f)), id)
    }

    function creer() {
        const f = familleChoisie()
        if (f === null) {
            signaler("Union à choisir", "Cette personne a plusieurs unions : choisissez celle à laquelle rattacher l'enfant.")
            return
        }
        const r = JSON.parse(noyau.creerProche(fiche.id, relation, JSON.stringify(formulaire.valeurs()), f))
        terminer(r, r.id || "")
    }

    Timer { id: minuterie; interval: 250; onTriggered: w.chercher() }
    Timer { id: minuterieSemblables; interval: 300; onTriggered: w.chercherSemblables() }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        Label {
            text: "AJOUTER " + w.libelleRelation.toUpperCase() + " À " + (w.fiche.nomAffiche || "").toUpperCase()
            font.bold: true
            font.pixelSize: 14
            color: "#234E70"
            Layout.fillWidth: true
            elide: Text.ElideRight
        }

        RowLayout {
            visible: w.unions.length > 0 && (w.relation === "enfant" || w.unions.length > 1)
            Layout.fillWidth: true
            Label { text: "Union :"; font.bold: true }
            ComboBox {
                id: choixUnion
                Layout.fillWidth: true
                model: w.unions
                textRole: "libelle"
                displayText: currentIndex < 0 ? "— Choisissez l'union —" : currentText
            }
        }

        TabBar {
            id: onglets
            Layout.fillWidth: true
            TabButton { text: "🔎 Rechercher et associer" }
            TabButton { text: "➕ Créer une nouvelle personne" }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: onglets.currentIndex

            ColumnLayout {
                spacing: 6
                Label {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    color: "#425466"
                    text: "Tapez quelques lettres du nom, du prénom, ou une année : les personnes déjà présentes "
                          + "dans le fichier s'affichent. Choisissez-en une, puis « Associer ». Si elle n'y est pas, créez-la."
                }
                TextField {
                    id: recherche
                    Layout.fillWidth: true
                    placeholderText: "ex. dupont jean 1850"
                    onTextChanged: minuterie.restart()
                    onAccepted: if (w.choisi !== "") w.associer(w.choisi)
                }
                Frame {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    padding: 1
                    ListView {
                        anchors.fill: parent
                        clip: true
                        model: w.resultats
                        ScrollBar.vertical: ScrollBar {}
                        delegate: Rectangle {
                            required property int index
                            required property var modelData
                            width: ListView.view.width
                            height: 26
                            color: modelData.id === w.choisi ? "#B9D7EE" : (index % 2 === 0 ? "white" : "#F5F7F9")
                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 6
                                anchors.rightMargin: 6
                                Text { text: modelData.nomListe; font.bold: true; Layout.fillWidth: true; elide: Text.ElideRight; color: "#253746" }
                                Text { text: modelData.dates; color: "#425466" }
                                Text { text: modelData.lieu; color: "#5A6872"; Layout.maximumWidth: 150; elide: Text.ElideRight }
                                Text { text: modelData.id; color: "#5A6872"; Layout.preferredWidth: 70 }
                            }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: w.choisi = modelData.id
                                onDoubleClicked: w.associer(modelData.id)
                            }
                        }
                    }
                }
                Label {
                    visible: recherche.text.trim().length >= 2
                    color: "#5A6872"
                    text: w.total === 0 ? "Aucune personne de ce nom dans le fichier."
                          : w.total > w.resultats.length ? w.total + " personnes correspondent, " + w.resultats.length + " affichées : précisez la saisie."
                          : w.total + " personne(s) trouvée(s)."
                }
                RowLayout {
                    Bouton {
                        text: "🔗 Associer"
                        couleur: "#4F8A5B"
                        enabled: w.choisi !== ""
                        aide: "Relier la personne choisie dans la liste, sans rien créer"
                        onClicked: w.associer(w.choisi)
                    }
                    Bouton {
                        text: recherche.text.trim() === "" ? "➕ Créer une nouvelle personne" : "➕ Créer « " + recherche.text.trim() + " »"
                        Layout.maximumWidth: 360
                        aide: "Elle n'est pas dans la liste : la créer avec le nom tapé, à compléter avant l'enregistrement"
                        onClicked: w.versCreation()
                    }
                }
            }

            ColumnLayout {
                spacing: 6
                FormulaireIdentite {
                    id: formulaire
                    Layout.fillWidth: true
                    onValide: w.creer()
                    onSaisie: minuterieSemblables.restart()
                }
                Label {
                    visible: w.semblables.length > 0
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    text: "⚠ Personnes semblables déjà dans le fichier — s'il s'agit de l'une d'elles, associez-la plutôt que d'en créer un doublon :"
                    color: "#8A4B0F"
                    font.bold: true
                }
                Frame {
                    visible: w.semblables.length > 0
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    padding: 1
                    ListView {
                        anchors.fill: parent
                        clip: true
                        model: w.semblables
                        ScrollBar.vertical: ScrollBar {}
                        delegate: Rectangle {
                            required property int index
                            required property var modelData
                            width: ListView.view.width
                            height: 30
                            color: index % 2 === 0 ? "white" : "#FFF7EC"
                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 6
                                anchors.rightMargin: 4
                                Text { text: modelData.nomListe; font.bold: true; Layout.fillWidth: true; elide: Text.ElideRight }
                                Text { text: modelData.dates; color: "#425466" }
                                Text { text: modelData.id; color: "#5A6872"; Layout.preferredWidth: 64 }
                                Bouton {
                                    text: "🔗 Associer celle-ci"
                                    topPadding: 3
                                    bottomPadding: 3
                                    couleur: "#4F8A5B"
                                    aide: "Relier cette personne déjà présente au lieu d'en créer une nouvelle"
                                    onClicked: w.associer(modelData.id)
                                }
                            }
                        }
                    }
                }
                Item { Layout.fillHeight: true; visible: w.semblables.length === 0 }
                Bouton {
                    Layout.fillWidth: true
                    text: "✓ Créer et relier"
                    couleur: "#4F8A5B"
                    aide: "Créer la personne et la relier, en une seule écriture du fichier (copie .bak faite avant)"
                    onClicked: w.creer()
                }
            }
        }

        RowLayout {
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                color: "#5A6872"
                font.pixelSize: 11
                text: "Une copie .bak du fichier est faite avant chaque écriture."
            }
            Bouton { text: "Fermer"; couleur: "#D9E4ED"; couleurTexte: "#234E70"; aide: "Fermer sans rien relier"; onClicked: w.close() }
        }
    }

    MessageDialog { id: boite; buttons: MessageDialog.Ok }
}
