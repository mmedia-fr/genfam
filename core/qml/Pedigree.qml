// Pedigree familial interactif (show_pedigree) : un clic sur une carte la place au
// centre ; le nombre de générations et le zoom se règlent dans la barre d'outils.
// La mise en page est calculée par le noyau ; ici, on ne fait que la dessiner.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: w

    property var noyau
    property string personne: ""
    property real zoom: 0.85
    property var page: ({ largeur: 0, hauteur: 0, cartes: [], traits: [], etiquettes: [], centre: { x: 0, y: 0 } })

    signal centre(string id)

    title: "Pedigree familial — " + (noyau && personne !== "" ? noyau.nomDe(personne) : "")
    width: 1400
    height: 880
    minimumWidth: 1000
    minimumHeight: 650
    color: "#EAF1F7"

    function montrer(id) {
        personne = id
        dessiner()
        showMaximized()
        raise()
    }

    function dessiner() {
        if (personne === "")
            return
        page = JSON.parse(noyau.pedigree(personne, haut.value, bas.value, zoom))
        Qt.callLater(recentrer)
    }

    function recentrer() {
        vue.contentX = Math.max(0, Math.min(page.largeur - vue.width, page.centre.x - vue.width / 2))
        vue.contentY = Math.max(0, Math.min(page.hauteur - vue.height, page.centre.y - vue.height / 2))
    }

    function choisir(id) {
        personne = id
        w.centre(id)
        dessiner()
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 6

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 56
            color: "#234E70"
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 16
                Text { text: "🌳  PEDIGREE FAMILIAL"; color: "white"; font.pixelSize: 22; font.bold: true }
                Text { text: "— " + (w.page.nom || ""); color: "#DCEAF5"; font.pixelSize: 16 }
                Item { Layout.fillWidth: true }
                Text { text: "N. Naissance   •   M. Mariage   •   D. Décès"; color: "#DCEAF5"; font.pixelSize: 12 }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 46
            color: "#D9E4ED"
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 10
                anchors.rightMargin: 6
                spacing: 6
                Label { text: "Ascendants :"; font.bold: true; color: "#234E70" }
                SpinBox { id: haut; from: 0; to: 20; value: 3; editable: true }
                Label { text: "Descendants :"; font.bold: true; color: "#234E70" }
                SpinBox { id: bas; from: 0; to: 20; value: 3; editable: true }
                Bouton { text: "Appliquer"; onClicked: w.dessiner() }
                Bouton { text: "＋ Zoom"; couleur: "#F4F7FA"; couleurTexte: "#234E70"; onClicked: { w.zoom = Math.min(1.5, w.zoom + 0.1); w.dessiner() } }
                Bouton { text: "− Zoom"; couleur: "#F4F7FA"; couleurTexte: "#234E70"; onClicked: { w.zoom = Math.max(0.5, w.zoom - 0.1); w.dessiner() } }
                Bouton { text: "100 %"; couleur: "#F4F7FA"; couleurTexte: "#234E70"; onClicked: { w.zoom = 0.85; w.dessiner() } }
                Bouton { text: "⛶ Ajuster"; couleur: "#F4F7FA"; couleurTexte: "#234E70"; onClicked: { w.zoom = 0.65; w.dessiner() } }
                Bouton { text: "🏠 Personne centrale"; couleur: "#F4F7FA"; couleurTexte: "#234E70"; onClicked: w.recentrer() }
                Item { Layout.fillWidth: true }
                Bouton { text: "Fermer"; couleur: "#F4F7FA"; couleurTexte: "#234E70"; onClicked: w.close() }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#FCFBF7"
            border.color: "#B7C7D5"
            clip: true

            Flickable {
                id: vue
                anchors.fill: parent
                anchors.margins: 1
                contentWidth: w.page.largeur
                contentHeight: w.page.hauteur
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar { policy: ScrollBar.AlwaysOn }
                ScrollBar.horizontal: ScrollBar { policy: ScrollBar.AlwaysOn }

                // Traits : tous horizontaux ou verticaux, dessinés en rectangles.
                Repeater {
                    model: w.page.traits
                    Rectangle {
                        required property var modelData
                        readonly property real ep: modelData[4]
                        x: Math.min(modelData[0], modelData[2]) - ep / 2
                        y: Math.min(modelData[1], modelData[3]) - ep / 2
                        width: Math.abs(modelData[2] - modelData[0]) + ep
                        height: Math.abs(modelData[3] - modelData[1]) + ep
                        color: modelData[5]
                    }
                }

                Repeater {
                    model: w.page.etiquettes
                    Text {
                        required property var modelData
                        x: modelData.x
                        y: modelData.y - height / 2
                        text: modelData.texte
                        color: "#234E70"
                        font.bold: true
                        font.pixelSize: Math.max(9, 13 * w.zoom)
                    }
                }

                Repeater {
                    model: w.page.cartes
                    Item {
                        id: carte
                        required property var modelData
                        readonly property real s: w.page.echelle
                        readonly property var teinte: ({
                            centrale:   { fond: "#E4F1FA", bande: "#C7E0F1", bord: "#1D4F73" },
                            conjoint:   { fond: "#FBF7EC", bande: "#F1E8CF", bord: "#8B7650" },
                            ascendant:  { fond: "#F2F7FB", bande: "#DCEAF3", bord: "#70889A" },
                            descendant: { fond: "#F3F8F3", bande: "#DFEBE0", bord: "#718A78" },
                            racine:     { fond: "#FFFFFF", bande: "#E8EFF3", bord: "#9AAEBB" }
                        })[modelData.categorie]
                        x: modelData.x
                        y: modelData.y
                        width: w.page.carteL
                        height: w.page.carteH

                        function ou(v) { return v && v !== "" ? v : "Non renseigné" }

                        Rectangle { x: 4 * carte.s; y: 5 * carte.s; width: parent.width; height: parent.height; radius: 14 * carte.s; color: "#D8DEE2" }
                        Rectangle {
                            anchors.fill: parent
                            radius: 14 * carte.s
                            color: carte.teinte.fond
                            border.color: carte.teinte.bord
                            border.width: carte.modelData.categorie === "centrale" ? Math.max(3, 3 * carte.s) : Math.max(1, 1.5 * carte.s)
                            clip: true
                            Rectangle {
                                x: 1; y: 1
                                width: parent.width - 2
                                height: 44 * carte.s
                                radius: parent.radius
                                color: carte.teinte.bande
                            }
                        }
                        Rectangle { x: 12 * carte.s; y: 14 * carte.s; width: 8 * carte.s; height: width; radius: width / 2; color: carte.teinte.bord }
                        Text {
                            x: 24 * carte.s
                            width: parent.width - 36 * carte.s
                            height: 44 * carte.s
                            verticalAlignment: Text.AlignVCenter
                            horizontalAlignment: Text.AlignHCenter
                            text: carte.modelData.nom
                            color: "#173A54"
                            font.bold: true
                            font.pixelSize: Math.max(8, 14 * carte.s)
                            wrapMode: Text.WordWrap
                            maximumLineCount: 2
                            elide: Text.ElideRight
                        }
                        Rectangle { x: 12 * carte.s; y: 44 * carte.s; width: parent.width - 24 * carte.s; height: 1; color: "#C8D5DE" }
                        Column {
                            x: 14 * carte.s
                            y: 52 * carte.s
                            width: parent.width - 26 * carte.s
                            spacing: 4 * carte.s
                            Repeater {
                                model: [["N.", carte.modelData.naissance, carte.modelData.lieuNaissance],
                                        ["M.", carte.modelData.mariage, carte.modelData.lieuMariage],
                                        ["D.", carte.modelData.deces, carte.modelData.lieuDeces]]
                                Column {
                                    required property var modelData
                                    width: parent.width
                                    Text { text: modelData[0] + " " + carte.ou(modelData[1]); color: "#1D5A7A"; font.bold: true; font.pixelSize: Math.max(7, 12 * carte.s) }
                                    Text {
                                        leftPadding: 23 * carte.s
                                        width: parent.width
                                        text: carte.ou(modelData[2])
                                        color: "#536878"
                                        font.pixelSize: Math.max(7, 11 * carte.s)
                                        elide: Text.ElideRight
                                    }
                                }
                            }
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: w.choisir(carte.modelData.id)
                        }
                    }
                }
            }
        }

        Text {
            text: "● Cliquez sur une personne pour la placer au centre     │ Glissez pour déplacer le pedigree"
            color: "#425466"
        }
    }
}
