# GenFam — Généalogie

Programme de généalogie de bureau : ouverture et modification de fichiers GEDCOM,
exploration d'une famille, pedigree interactif, numérotation Ahnentafel, ordonnances,
et import de l'ascendance depuis FamilySearch.

Conçu par **Claude Boisseau** (version Python / Tkinter, `GENFAMILLE.pyw`, V7.6).
Portage en Rust / Qt 6 et liaison FamilySearch : M-Media. Tous droits réservés à
l'auteur tant qu'aucune licence n'a été choisie.

## Fonctions

- **Fichier** : nouveau GEDCOM, ouvrir, enregistrer sous, fermer. Un fichier `.ged`
  passé en argument (double-clic dans l'explorateur) est ouvert directement.
- **Liste des personnes** : recherche (nom, identifiant, lieux), tri alphabétique,
  par numéro GEDCOM ou par numéro Ahnentafel.
- **Informations** : fiche complète (les proches sont des liens cliquables), famille,
  ascendants et descendants par génération, export texte.
- **Pedigree** : ascendants et descendants (0 à 20 générations), zoom, recentrage
  par clic sur une carte.
- **Ahnentafel** : choix de la personne N° 1 ; un ancêtre en implexe reçoit plusieurs numéros.
- **Ajouter / supprimer** : nouvelle personne reliée à son père et/ou sa mère ;
  suppression avec nettoyage des familles vidées ; annulation de la dernière
  modification (copie `<fichier>.bak`).
- **Ordonnances** : enregistrées dans `<fichier>.ordonnances.json`, à côté du GEDCOM.
- **FamilySearch** : connexion OAuth 2.0 (le mot de passe se saisit sur le site de
  FamilySearch, jamais dans le programme), puis import de l'ascendance (1 à 8
  générations) d'une personne. Les ancêtres absents sont créés et reliés, reconnus
  ensuite par leur identifiant FamilySearch (`_FSFTID`, balise également posée par
  Ancestral Quest) ; rien de ce qui existe n'est modifié.

Tout ce que le programme ne comprend pas dans un GEDCOM (sources, médias, balises
propres à un autre logiciel) est conservé tel quel à l'écriture.

## FamilySearch : prérequis

L'API FamilySearch exige une **App Key**, délivrée au développeur après acceptation
de sa candidature (<https://developers.familysearch.org>). Sur la fiche de l'application,
déclarer l'URI de rappel :

    http://127.0.0.1:57938/familysearch-auth

La clé fonctionne d'emblée sur l'environnement **intégration** (données de test) ;
la **production** (arbre réel) n'est ouverte qu'après la revue de compatibilité
de FamilySearch.

## Fabrication

Noyau Rust (`core/`, crate `genfam_core`, liaison Qt par `cxx-qt` 0.10), interface
QML (`core/qml/`), point d'entrée C++ (`cpp/main.cpp`). Qt 6.4 minimum.

    cmake -S . -B _build -G Ninja -DCMAKE_BUILD_TYPE=Release
    cmake --build _build
    cargo test --manifest-path core/Cargo.toml
    QT_QPA_PLATFORM=offscreen _build/genfam --smoke core/tests/donnees/exemple.ged

L'intégration continue (`.github/workflows/fabrication.yml`) produit l'installeur
Windows (Inno Setup, `build/genfam.iss`) et une AppImage Linux.
