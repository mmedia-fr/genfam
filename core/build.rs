use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // Même précaution que MMdedit : pas de compression zstd des ressources QML.
    std::env::set_var("CXX_QT_AUTORCC_OPTIONS", "--no-zstd");

    CxxQtBuilder::new_qml_module(QmlModule::new("fr.mmedia.genfam").qml_files([
        "qml/Main.qml",
        "qml/Accueil.qml",
        "qml/ChoixRacine.qml",
        "qml/GestionPersonnes.qml",
        "qml/Ordonnances.qml",
        "qml/Pedigree.qml",
        "qml/FamilySearch.qml",
        "qml/Bouton.qml",
    ]))
    .qt_module("Qml")
    .files(["src/pont.rs"])
    .build();
}
