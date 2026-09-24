// Connexion à FamilySearch et import de l'ascendance d'une personne.
//
// L'App Key et l'environnement sont retenus d'une session à l'autre. Le mot de passe
// FamilySearch n'est jamais saisi ici : il se tape sur le site de FamilySearch.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt.labs.settings

Window {
    id: w

    property var noyau
    property string personne: ""

    title: "FamilySearch"
    width: 720
    height: 640
    minimumWidth: 620
    minimumHeight: 560
    color: "#EAF1F7"

    readonly property var environnements: [
        { cle: "integration", libelle: "Intégration — données de test (toute App Key)" },
        { cle: "beta", libelle: "Bêta — sur demande à FamilySearch" },
        { cle: "production", libelle: "Production — données réelles (App Key approuvée)" }
    ]

    Settings {
        id: reglages
        category: "familysearch"
        property string cle: ""
        property string environnement: "integration"
        property int generations: 4
    }

    function montrer() {
        cle.text = reglages.cle
        for (let i = 0; i < environnements.length; ++i)
            if (environnements[i].cle === reglages.environnement)
                env.currentIndex = i
        generations.value = reglages.generations
        fsid.text = personne !== "" ? noyau.fsidDe(personne) : ""
        show()
        raise()
    }

    onPersonneChanged: if (visible) fsid.text = personne !== "" ? noyau.fsidDe(personne) : ""

    ScrollView {
        anchors.fill: parent
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            spacing: 8

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 50
                color: "#234E70"
                Text { anchors.verticalCenter: parent.verticalCenter; x: 16; text: "🌐  FAMILYSEARCH"; color: "white"; font.pixelSize: 20; font.bold: true }
            }

            GroupBox {
                title: "1. Connexion"
                Layout.fillWidth: true
                Layout.margins: 12
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 6
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        color: "#425466"
                        text: "L'App Key est délivrée par FamilySearch au développeur de l'application ; elle est différente de votre identifiant et de votre mot de passe personnels."
                    }
                    Label { text: "App Key FamilySearch :"; font.bold: true }
                    TextField { id: cle; Layout.fillWidth: true; placeholderText: "ex. a02j000000XXXXXXXXX"; onEditingFinished: reglages.cle = text.trim() }
                    Label { text: "Environnement :"; font.bold: true }
                    ComboBox {
                        id: env
                        Layout.fillWidth: true
                        model: w.environnements
                        textRole: "libelle"
                        valueRole: "cle"
                        onActivated: reglages.environnement = currentValue
                    }
                    Label { text: "URI de rappel à déclarer pour cette App Key :"; font.bold: true }
                    TextField { Layout.fillWidth: true; readOnly: true; selectByMouse: true; text: w.noyau ? w.noyau.uriRappel() : "" }
                    RowLayout {
                        Bouton {
                            text: w.noyau && w.noyau.connecte ? "Se reconnecter" : "Se connecter à FamilySearch"
                            enabled: w.noyau && !w.noyau.occupe
                            onClicked: {
                                reglages.cle = cle.text.trim()
                                reglages.environnement = env.currentValue
                                const url = w.noyau.connecter(cle.text, env.currentValue)
                                if (url !== "")
                                    Qt.openUrlExternally(url)
                            }
                        }
                        Bouton {
                            text: "Se déconnecter"
                            couleur: "#D9E4ED"; couleurTexte: "#234E70"
                            visible: w.noyau && w.noyau.connecte
                            onClicked: w.noyau.deconnecter()
                        }
                        BusyIndicator { running: w.noyau && w.noyau.occupe; visible: running; Layout.preferredHeight: 32; Layout.preferredWidth: 32 }
                    }
                }
            }

            GroupBox {
                title: "2. Import de l'ascendance"
                Layout.fillWidth: true
                Layout.leftMargin: 12
                Layout.rightMargin: 12
                enabled: w.noyau && w.noyau.connecte && !w.noyau.occupe
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 6
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        text: w.personne !== "" && w.noyau
                              ? "Personne de départ : <b>" + w.noyau.nomDe(w.personne) + "</b> [" + w.personne + "]"
                              : "Sélectionnez d'abord une personne dans la liste principale."
                        textFormat: Text.StyledText
                    }
                    Label { text: "Identifiant FamilySearch de cette personne :"; font.bold: true }
                    TextField { id: fsid; Layout.preferredWidth: 220; placeholderText: "ex. KWCB-QZ3"; font.capitalization: Font.AllUppercase }
                    RowLayout {
                        Label { text: "Générations :"; font.bold: true }
                        SpinBox { id: generations; from: 1; to: 8; value: 4; onValueModified: reglages.generations = value }
                    }
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        color: "#425466"
                        text: "Les ancêtres absents du fichier sont ajoutés et reliés ; ceux qui y figurent déjà (même identifiant FamilySearch) ne sont pas dupliqués, et rien d'existant n'est modifié. Une copie .bak du fichier est faite avant l'écriture : « Annuler la dernière modification » la restaure."
                    }
                    Bouton {
                        text: "⬇ Importer depuis FamilySearch"
                        couleur: "#4F8A5B"
                        enabled: w.personne !== ""
                        onClicked: w.noyau.importer(w.personne, fsid.text, generations.value)
                    }
                }
            }

            Frame {
                Layout.fillWidth: true
                Layout.margins: 12
                visible: w.noyau && w.noyau.message !== ""
                background: Rectangle { color: "white"; border.color: "#B7C7D5" }
                Label {
                    anchors.fill: parent
                    wrapMode: Text.WordWrap
                    text: w.noyau ? w.noyau.message : ""
                    color: "#173A54"
                }
            }

            Bouton {
                Layout.alignment: Qt.AlignRight
                Layout.rightMargin: 12
                Layout.bottomMargin: 12
                text: "Fermer"
                couleur: "#D9E4ED"; couleurTexte: "#234E70"
                onClicked: w.close()
            }
        }
    }
}
