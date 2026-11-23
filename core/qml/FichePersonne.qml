// SPDX-License-Identifier: GPL-3.0-or-later
// Fiche d'une personne, à onglets : identité et événements (modifiables), famille
// (parents, conjoints, enfants, chacun avec son menu « Actions »), ordonnances.
// Ouverte depuis la fenêtre principale, et par un double-clic sur une carte du pedigree.
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts

Window {
    id: w

    property var noyau
    property string personne: ""
    property var fiche: ({})
    property var historique: []
    /// Compte rendu de la dernière action, montré dans le bandeau.
    property string message: ""
    property bool messageErreur: false
    /// Retrait en attente de confirmation.
    property var retrait: ({})
    readonly property alias lien: lien

    /// Une autre personne est ouverte dans la fiche.
    signal aller(string id)

    title: "Fiche — " + (fiche.nomAffiche || "")
    width: 1000
    height: 720
    minimumWidth: 760
    minimumHeight: 520
    color: "#EAF1F7"

    function montrer(id, onglet) {
        if (visible && personne !== "" && personne !== id)
            historique = historique.concat([personne])
        personne = id
        message = ""
        charger(true)
        if (onglet === "identite")
            onglets.currentIndex = 0
        else if (onglet === "famille")
            onglets.currentIndex = 1
        else if (onglet === "ordonnances")
            onglets.currentIndex = 2
        show()
        raise()
        requestActivate()
    }

    /// Relit la fiche. La saisie en cours dans l'onglet Identité n'est pas écrasée par
    /// un rafraîchissement venu d'ailleurs, sauf si `forcer`.
    function charger(forcer) {
        const f = personne === "" ? {} : JSON.parse(noyau.fichePersonne(personne))
        fiche = f
        if (f.id === undefined) {
            identite.vider()
            return
        }
        if (forcer || !identite.modifie)
            identite.remplir(f)
        ordonnancesListe.remplir()
        listeParents.choisi = ""
        listeConjoints.choisi = ""
        listeEnfants.choisi = ""
    }

    function ouvrir(id, onglet) {
        if (id === "" || id === personne)
            return
        historique = historique.concat([personne])
        personne = id
        message = ""
        charger(true)
        if (onglet !== undefined)
            onglets.currentIndex = onglet
        w.aller(id)
    }

    function precedente() {
        if (historique.length === 0)
            return
        const h = historique.slice()
        personne = h.pop()
        historique = h
        message = ""
        charger(true)
        w.aller(personne)
    }

    function annoncer(r) {
        message = r.message
        messageErreur = !r.ok
        if (!r.ok) {
            boite.title = "Modification impossible"
            boite.text = r.message
            boite.open()
        }
    }

    function enregistrer() {
        const r = JSON.parse(noyau.modifierPersonne(personne, JSON.stringify(identite.valeurs())))
        annoncer(r)
        if (r.ok)
            charger(true)
    }

    function detail(p) {
        if (p.mariage !== undefined)
            return [p.mariage ? "mariage " + p.mariage : "", p.enfants + " enfant(s)"].filter(x => x !== "").join(" — ")
        if (p.autreParent !== undefined)
            return p.autreParent !== "" ? "avec " + p.autreParent : "autre parent inconnu"
        return ""
    }

    function ligne(liste, id) {
        for (const p of liste || [])
            if (p.id === id)
                return p
        return null
    }

    function confirmerRetrait(relation, id) {
        const nom = fiche.nomAffiche
        let t = ""
        if (relation === "parent") {
            const p = ligne(fiche.parents, id)
            t = "Retirer le lien entre " + nom + " et " + (p.role === "Mère" ? "sa mère, " : "son père, ") + p.nom + " ?\n\n"
                + nom + " ne sera plus rattaché(e) à " + p.nom + ". Les frères et sœurs ne changent pas."
        } else if (relation === "enfant") {
            const p = ligne(fiche.enfants, id)
            t = "Retirer " + p.nom + " des enfants de " + nom + " ?\n\n"
                + p.nom + " ne sera plus rattaché(e) à " + nom
                + (p.autreParent !== "" ? " ; il ou elle restera l'enfant de " + p.autreParent + "." : ".")
        } else {
            const p = ligne(fiche.conjoints, id)
            t = "Retirer l'union entre " + nom + " et " + p.nom + " ?\n\n"
                + (p.enfants > 0 ? "Les " + p.enfants + " enfant(s) de cette union resteront rattachés à " + nom + " seul(e)."
                                 : "L'union " + p.famille + (p.mariage ? " (mariage : " + p.mariage + ")" : "") + " sera supprimée du fichier.")
        }
        t += "\n\nAucune personne n'est supprimée. Une copie .bak du fichier est faite avant l'écriture ; "
             + "« Annuler la dernière modification » la restaure."
        retrait = { relation: relation, id: id }
        confirmation.text = t
        confirmation.open()
    }

    function ajouter(relation, mode) {
        lien.ouvrir(fiche, relation, mode)
    }

    Connections {
        target: w.noyau
        function onRevisionChanged() { if (w.visible && w.personne !== "") w.charger(false) }
    }

    // Une ligne de proche : nom, dates, détail (union, autre parent), identifiant.
    // Clic : sélection ; double-clic : ouvre sa fiche ; clic droit : menu « Actions ».
    Component {
        id: ligneProche
        Rectangle {
            id: rangee
            required property int index
            required property var modelData
            readonly property var vue: ListView.view
            width: vue ? vue.width : 0
            height: 28
            color: vue && vue.choisi === modelData.id ? "#B9D7EE" : (index % 2 === 0 ? "white" : "#F5F7F9")
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 10
                Text { visible: rangee.modelData.role !== undefined; text: rangee.modelData.role || ""; color: "#5A6872"; Layout.preferredWidth: 40 }
                Text { text: rangee.modelData.nom; font.bold: true; color: "#173A54"; Layout.fillWidth: true; elide: Text.ElideRight }
                Text { text: rangee.modelData.dates; color: "#425466" }
                Text { text: w.detail(rangee.modelData); color: "#5A6872"; Layout.maximumWidth: 260; elide: Text.ElideRight }
                Text { text: rangee.modelData.id; color: "#5A6872"; Layout.preferredWidth: 70 }
            }
            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                onClicked: (souris) => {
                    rangee.vue.choisi = rangee.modelData.id
                    if (souris.button === Qt.RightButton)
                        rangee.vue.menu.popup()
                }
                onDoubleClicked: w.ouvrir(rangee.modelData.id)
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 6

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 58
            color: "#234E70"
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 8
                spacing: 8
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 0
                    Text {
                        text: w.fiche.nomAffiche || "Personne introuvable"
                        color: "white"; font.pixelSize: 20; font.bold: true
                        Layout.fillWidth: true; elide: Text.ElideRight
                    }
                    Text {
                        text: [w.fiche.dates || "", w.fiche.id ? "ID " + w.fiche.id : "",
                               w.fiche.numero ? "Ahnentafel N° " + w.fiche.numero : "",
                               w.fiche.fsid ? "FamilySearch " + w.fiche.fsid : ""].filter(x => x !== "").join("   •   ")
                        color: "#DCEAF5"; font.pixelSize: 12
                    }
                }
                Bouton {
                    text: "◀ Précédente"
                    couleur: "#F4F7FA"; couleurTexte: "#234E70"
                    enabled: w.historique.length > 0
                    aide: "Revenir à la fiche ouverte juste avant"
                    onClicked: w.precedente()
                }
                Bouton {
                    text: "↶ Annuler la dernière modification"
                    couleur: "#D8872F"
                    enabled: w.noyau ? w.noyau.annulable : false
                    aide: "Rétablir le fichier tel qu'il était avant la dernière écriture (copie .bak)"
                    onClicked: {
                        const e = w.noyau.annuler()
                        w.annoncer({ ok: e === "", message: e === "" ? "La dernière modification a été annulée." : e })
                        w.charger(true)
                    }
                }
                Bouton {
                    text: w.visibility === Window.Maximized ? "🗗 Restaurer" : "🗖 Agrandir"
                    couleur: "#F4F7FA"; couleurTexte: "#234E70"
                    aide: w.visibility === Window.Maximized ? "Rendre à la fenêtre sa taille précédente" : "Agrandir la fiche à tout l'écran"
                    onClicked: w.visibility === Window.Maximized ? w.showNormal() : w.showMaximized()
                }
                Bouton {
                    text: "Fermer"
                    couleur: "#F4F7FA"; couleurTexte: "#234E70"
                    aide: "Fermer la fiche"
                    onClicked: w.close()
                }
            }
        }

        Rectangle {
            visible: w.message !== ""
            Layout.fillWidth: true
            Layout.preferredHeight: bandeau.implicitHeight + 12
            color: w.messageErreur ? "#F8E1E1" : "#E3F1E5"
            border.color: w.messageErreur ? "#B83A3A" : "#4F8A5B"
            radius: 3
            Text {
                id: bandeau
                anchors.fill: parent
                anchors.margins: 6
                text: w.message
                wrapMode: Text.WordWrap
                color: "#253746"
            }
        }

        TabBar {
            id: onglets
            Layout.fillWidth: true
            TabButton { text: "👤 Identité et événements" }
            TabButton { text: "👨‍👩‍👧 Famille" }
            TabButton { text: "📜 Ordonnances" }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: onglets.currentIndex

            // ---- Identité et événements --------------------------------------------
            ScrollView {
                contentWidth: availableWidth
                ColumnLayout {
                    width: parent.width
                    spacing: 8
                    Frame {
                        Layout.fillWidth: true
                        FormulaireIdentite {
                            id: identite
                            width: parent.width
                            onValide: if (identite.modifie) w.enregistrer()
                        }
                    }
                    RowLayout {
                        Bouton {
                            text: "💾 Enregistrer les modifications"
                            couleur: "#4F8A5B"
                            enabled: identite.modifie
                            aide: "Écrire la fiche dans le fichier GEDCOM (une copie .bak est faite avant)"
                            onClicked: w.enregistrer()
                        }
                        Bouton {
                            text: "↺ Rétablir"
                            couleur: "#D9E4ED"; couleurTexte: "#234E70"
                            enabled: identite.modifie
                            aide: "Oublier la saisie et revenir aux valeurs du fichier"
                            onClicked: identite.remplir(w.fiche)
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        color: "#425466"
                        visible: (w.fiche.mariage || "") !== "" || (w.fiche.lieuMariage || "") !== ""
                        text: "Mariage : " + [w.fiche.mariage || "", w.fiche.lieuMariage || ""].filter(x => x !== "").join(", ")
                              + "  (se modifie dans le logiciel d'origine ; il est conservé tel quel)"
                    }
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        color: "#5A6872"
                        text: "Seuls le nom, les prénoms, le sexe, la naissance et le décès sont réécrits : "
                              + "sources, notes et toute autre information de la fiche restent en place."
                    }
                    Label { visible: (w.fiche.notes || []).length > 0; text: "Notes"; font.bold: true; color: "#234E70" }
                    Repeater {
                        model: w.fiche.notes || []
                        Label {
                            required property var modelData
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                            text: modelData
                        }
                    }
                }
            }

            // ---- Famille ---------------------------------------------------------------
            ScrollView {
                id: defilFamille
                contentWidth: availableWidth
                ColumnLayout {
                    width: defilFamille.availableWidth
                    spacing: 10

                    // Parents
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "PARENTS"; font.bold: true; font.pixelSize: 13; color: "#234E70"; Layout.fillWidth: true }
                        Bouton {
                            id: actionsParents
                            text: "Actions ▾"
                            aide: "Associer ou créer le père ou la mère, modifier les parents, ouvrir ou retirer le parent sélectionné"
                            onClicked: menuParents.popup(actionsParents, 0, actionsParents.height)
                        }
                    }
                    Frame {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.max(1, (w.fiche.parents || []).length) * 28 + 4
                        padding: 1
                        Label { visible: (w.fiche.parents || []).length === 0; anchors.centerIn: parent; text: "Parents non renseignés"; color: "#777777"; font.italic: true }
                        ListView {
                            id: listeParents
                            property string choisi: ""
                            property var menu: menuParents
                            anchors.fill: parent
                            interactive: false
                            model: w.fiche.parents || []
                            delegate: ligneProche
                        }
                    }

                    // Conjoints
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "CONJOINT(S)"; font.bold: true; font.pixelSize: 13; color: "#234E70"; Layout.fillWidth: true }
                        Bouton {
                            id: actionsConjoints
                            text: "Actions ▾"
                            aide: "Associer ou créer un conjoint, ouvrir ou retirer le conjoint sélectionné"
                            onClicked: menuConjoints.popup(actionsConjoints, 0, actionsConjoints.height)
                        }
                    }
                    Frame {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.max(1, (w.fiche.conjoints || []).length) * 28 + 4
                        padding: 1
                        Label { visible: (w.fiche.conjoints || []).length === 0; anchors.centerIn: parent; text: "Conjoint non renseigné"; color: "#777777"; font.italic: true }
                        ListView {
                            id: listeConjoints
                            property string choisi: ""
                            property var menu: menuConjoints
                            anchors.fill: parent
                            interactive: false
                            model: w.fiche.conjoints || []
                            delegate: ligneProche
                        }
                    }

                    // Enfants
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "ENFANTS"; font.bold: true; font.pixelSize: 13; color: "#234E70"; Layout.fillWidth: true }
                        Bouton {
                            id: actionsEnfants
                            text: "Actions ▾"
                            aide: "Associer ou créer un enfant (en choisissant l'union), ouvrir ou retirer l'enfant sélectionné"
                            onClicked: menuEnfants.popup(actionsEnfants, 0, actionsEnfants.height)
                        }
                    }
                    Frame {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.max(1, (w.fiche.enfants || []).length) * 28 + 4
                        padding: 1
                        Label { visible: (w.fiche.enfants || []).length === 0; anchors.centerIn: parent; text: "Aucun enfant renseigné"; color: "#777777"; font.italic: true }
                        ListView {
                            id: listeEnfants
                            property string choisi: ""
                            property var menu: menuEnfants
                            anchors.fill: parent
                            interactive: false
                            model: w.fiche.enfants || []
                            delegate: ligneProche
                        }
                    }

                    // Fratrie, pour mémoire
                    Label { text: "FRATRIE"; font.bold: true; font.pixelSize: 13; color: "#234E70" }
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        color: "#425466"
                        text: (w.fiche.fratrie || []).length === 0 ? "Fratrie non renseignée"
                              : (w.fiche.fratrie || []).map(p => p.nom + (p.dates ? " (" + p.dates + ")" : "")).join("   •   ")
                    }
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        color: "#5A6872"
                        font.pixelSize: 11
                        text: "Clic : sélectionner • double-clic : ouvrir sa fiche • clic droit : menu Actions. "
                              + "Une copie .bak du fichier est faite avant chaque écriture."
                    }
                }
            }

            // ---- Ordonnances -----------------------------------------------------------
            ListeOrdonnances {
                id: ordonnancesListe
                noyau: w.noyau
                personne: w.personne
            }
        }
    }

    Menu {
        id: menuParents
        MenuItem { text: "🔗 Associer un père existant…"; enabled: !w.fiche.pere; onTriggered: w.ajouter("pere", "associer") }
        MenuItem { text: "🔗 Associer une mère existante…"; enabled: !w.fiche.mere; onTriggered: w.ajouter("mere", "associer") }
        MenuItem { text: "➕ Créer le père…"; enabled: !w.fiche.pere; onTriggered: w.ajouter("pere", "creer") }
        MenuItem { text: "➕ Créer la mère…"; enabled: !w.fiche.mere; onTriggered: w.ajouter("mere", "creer") }
        MenuSeparator {}
        MenuItem { text: "✎ Modifier les parents (remplacer, retirer)…"; onTriggered: dialogueParents.open() }
        MenuSeparator {}
        MenuItem { text: "👤 Ouvrir la fiche du parent sélectionné"; enabled: listeParents.choisi !== ""; onTriggered: w.ouvrir(listeParents.choisi) }
        MenuItem { text: "✏ Modifier le parent sélectionné…"; enabled: listeParents.choisi !== ""; onTriggered: w.ouvrir(listeParents.choisi, 0) }
        MenuItem { text: "✂ Retirer le lien avec le parent sélectionné…"; enabled: listeParents.choisi !== ""; onTriggered: w.confirmerRetrait("parent", listeParents.choisi) }
    }
    Menu {
        id: menuConjoints
        MenuItem { text: "🔗 Associer un conjoint existant…"; onTriggered: w.ajouter("conjoint", "associer") }
        MenuItem { text: "➕ Créer un conjoint…"; onTriggered: w.ajouter("conjoint", "creer") }
        MenuSeparator {}
        MenuItem { text: "👤 Ouvrir la fiche du conjoint sélectionné"; enabled: listeConjoints.choisi !== ""; onTriggered: w.ouvrir(listeConjoints.choisi) }
        MenuItem { text: "✏ Modifier le conjoint sélectionné…"; enabled: listeConjoints.choisi !== ""; onTriggered: w.ouvrir(listeConjoints.choisi, 0) }
        MenuItem { text: "✂ Retirer l'union avec le conjoint sélectionné…"; enabled: listeConjoints.choisi !== ""; onTriggered: w.confirmerRetrait("conjoint", listeConjoints.choisi) }
    }
    Menu {
        id: menuEnfants
        MenuItem { text: "🔗 Associer un enfant existant…"; onTriggered: w.ajouter("enfant", "associer") }
        MenuItem { text: "➕ Créer un enfant…"; onTriggered: w.ajouter("enfant", "creer") }
        MenuSeparator {}
        MenuItem { text: "👤 Ouvrir la fiche de l'enfant sélectionné"; enabled: listeEnfants.choisi !== ""; onTriggered: w.ouvrir(listeEnfants.choisi) }
        MenuItem { text: "✏ Modifier l'enfant sélectionné…"; enabled: listeEnfants.choisi !== ""; onTriggered: w.ouvrir(listeEnfants.choisi, 0) }
        MenuItem { text: "✂ Retirer le lien avec l'enfant sélectionné…"; enabled: listeEnfants.choisi !== ""; onTriggered: w.confirmerRetrait("enfant", listeEnfants.choisi) }
    }

    LienProche {
        id: lien
        noyau: w.noyau
        onFait: (texte, id) => w.annoncer({ ok: true, message: texte })
    }

    Dialog {
        id: dialogueParents
        title: "Modifier les parents de " + (w.fiche.nomAffiche || "")
        modal: true
        anchors.centerIn: Overlay.overlay
        width: Math.min(w.width - 40, 640)
        standardButtons: Dialog.Save | Dialog.Cancel
        onAboutToShow: {
            choixPere.definir(w.fiche.pere ? w.fiche.pere.id : "", w.fiche.pere ? w.fiche.pere.nom : "")
            choixMere.definir(w.fiche.mere ? w.fiche.mere.id : "", w.fiche.mere ? w.fiche.mere.nom : "")
        }
        onAccepted: w.annoncer(JSON.parse(w.noyau.definirParents(w.personne, choixPere.choisi, choixMere.choisi)))
        ColumnLayout {
            anchors.fill: parent
            spacing: 6
            Label { text: "Père"; font.bold: true }
            ChoixPersonne { id: choixPere; Layout.fillWidth: true; noyau: w.noyau; exclus: [w.personne] }
            Label { text: "Mère"; font.bold: true }
            ChoixPersonne { id: choixMere; Layout.fillWidth: true; noyau: w.noyau; exclus: [w.personne] }
            Label {
                Layout.fillWidth: true
                Layout.topMargin: 6
                wrapMode: Text.WordWrap
                color: "#425466"
                text: "Laissez une place vide pour retirer ce parent. Si ce couple a déjà une famille dans le fichier, "
                      + (w.fiche.nomAffiche || "la personne") + " y est placé(e) ; sinon une famille nouvelle est créée. "
                      + "Les frères et sœurs, et l'union des anciens parents, ne sont pas modifiés."
            }
        }
    }

    MessageDialog { id: boite; buttons: MessageDialog.Ok }
    MessageDialog {
        id: confirmation
        title: "Retirer un lien"
        buttons: MessageDialog.Yes | MessageDialog.No
        onButtonClicked: (bouton) => {
            if (bouton === MessageDialog.Yes)
                w.annoncer(JSON.parse(w.noyau.retirerLien(w.personne, w.retrait.relation, w.retrait.id)))
        }
    }
}
