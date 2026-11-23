// SPDX-License-Identifier: GPL-3.0-or-later
// Saisie de l'identité d'une personne : prénoms, nom, sexe, naissance et décès (date
// et lieu). Sert à la création d'une personne comme à la modification d'une fiche.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

GridLayout {
    id: f

    /// Valeurs au moment du remplissage : la saisie est « modifiée » si elle en diffère.
    property var origine: ({})
    readonly property var sexes: [
        { libelle: "Non renseigné", valeur: "" },
        { libelle: "Masculin (M)", valeur: "M" },
        { libelle: "Féminin (F)", valeur: "F" },
        { libelle: "Inconnu (U)", valeur: "U" }
    ]
    readonly property bool modifie: {
        const o = f.origine
        return prenoms.text !== (o.prenoms || "") || nom.text !== (o.nom || "")
            || sexe.currentIndex !== f.indexSexe(o.sexe || "")
            || naissance.text !== (o.naissance || "") || lieuNaissance.text !== (o.lieuNaissance || "")
            || deces.text !== (o.deces || "") || lieuDeces.text !== (o.lieuDeces || "")
    }

    /// Entrée dans un champ.
    signal valide()
    /// Le prénom ou le nom a changé (recherche des homonymes).
    signal saisie()

    function indexSexe(v) {
        for (let i = 0; i < sexes.length; ++i)
            if (sexes[i].valeur === v)
                return i
        return 0
    }

    function remplir(v) {
        const o = {
            prenoms: v.prenoms || "", nom: v.nom || "", sexe: v.sexe || "",
            naissance: v.naissance || "", lieuNaissance: v.lieuNaissance || "",
            deces: v.deces || "", lieuDeces: v.lieuDeces || ""
        }
        prenoms.text = o.prenoms
        nom.text = o.nom
        sexe.currentIndex = indexSexe(o.sexe)
        naissance.text = o.naissance
        lieuNaissance.text = o.lieuNaissance
        deces.text = o.deces
        lieuDeces.text = o.lieuDeces
        origine = o
    }

    function vider() { remplir({}) }

    function valeurs() {
        return {
            prenoms: prenoms.text, nom: nom.text, sexe: sexes[Math.max(0, sexe.currentIndex)].valeur,
            naissance: naissance.text, lieuNaissance: lieuNaissance.text,
            deces: deces.text, lieuDeces: lieuDeces.text
        }
    }

    function definirSexe(v) { sexe.currentIndex = indexSexe(v) }
    function focaliser() { prenoms.forceActiveFocus() }

    columns: 2
    columnSpacing: 10
    rowSpacing: 4

    Label { text: "Prénoms" }
    TextField { id: prenoms; Layout.fillWidth: true; placeholderText: "ex. Jean Marie"; onAccepted: f.valide(); onTextChanged: f.saisie() }
    Label { text: "Nom" }
    TextField { id: nom; Layout.fillWidth: true; placeholderText: "ex. DUPONT"; onAccepted: f.valide(); onTextChanged: f.saisie() }
    Label { text: "Sexe" }
    ComboBox { id: sexe; Layout.preferredWidth: 200; model: f.sexes; textRole: "libelle" }
    Label { text: "Naissance" }
    RowLayout {
        Layout.fillWidth: true
        TextField { id: naissance; Layout.preferredWidth: 170; placeholderText: "date, ex. 12 MAR 1850"; onAccepted: f.valide() }
        TextField { id: lieuNaissance; Layout.fillWidth: true; placeholderText: "lieu"; onAccepted: f.valide() }
    }
    Label { text: "Décès" }
    RowLayout {
        Layout.fillWidth: true
        TextField { id: deces; Layout.preferredWidth: 170; placeholderText: "date, ex. 3 FEB 1920"; onAccepted: f.valide() }
        TextField { id: lieuDeces; Layout.fillWidth: true; placeholderText: "lieu"; onAccepted: f.valide() }
    }
}
