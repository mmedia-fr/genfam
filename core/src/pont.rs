// SPDX-License-Identifier: GPL-3.0-or-later
//! Objet exposé à QML : l'état du fichier ouvert et tous les services du noyau.
//!
//! Les listes et mises en page passent en JSON : QML les lit par `JSON.parse`, ce
//! qui évite un modèle Qt par vue pour des volumes de quelques milliers de lignes.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// Nom du fichier ouvert ; vide si aucun.
        #[qproperty(QString, fichier)]
        /// Ligne de la barre d'état.
        #[qproperty(QString, resume)]
        /// Incrémenté à chaque changement des données : la liste se rafraîchit.
        #[qproperty(i32, revision)]
        /// Vrai si la dernière modification du fichier peut être annulée.
        #[qproperty(bool, annulable)]
        /// Identifiant de la personne N° 1 Ahnentafel.
        #[qproperty(QString, racine)]
        /// Session FamilySearch ouverte.
        #[qproperty(bool, connecte)]
        /// Échange FamilySearch en cours.
        #[qproperty(bool, occupe)]
        /// Dernier message FamilySearch à montrer.
        #[qproperty(QString, message)]
        /// Version du programme.
        #[qproperty(QString, version)]
        type Genealogie = super::GenealogieRust;

        #[qinvokable]
        #[cxx_name = "ouvrir"]
        fn ouvrir(self: Pin<&mut Genealogie>, url: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "nouveau"]
        fn nouveau(self: Pin<&mut Genealogie>, url: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "enregistrerSous"]
        fn enregistrer_sous(self: Pin<&mut Genealogie>, url: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "fermer"]
        fn fermer(self: Pin<&mut Genealogie>);

        #[qinvokable]
        #[cxx_name = "cheminCourant"]
        fn chemin_courant(&self) -> QString;

        /// Liste filtrée et triée : `[{id, numero, nom}]`.
        #[qinvokable]
        #[cxx_name = "personnes"]
        fn personnes(&self, requete: &QString, tri: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "nomDe"]
        fn nom_de(&self, id: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "fsidDe"]
        fn fsid_de(&self, id: &QString) -> QString;

        /// Présentation HTML : vue « fiche », « famille », « ascendants », « descendants ».
        #[qinvokable]
        #[cxx_name = "presentation"]
        fn presentation(&self, id: &QString, vue: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "exporter"]
        fn exporter(&self, url: &QString, id: &QString, vue: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "definirRacine"]
        fn definir_racine(self: Pin<&mut Genealogie>, id: &QString);

        #[qinvokable]
        #[cxx_name = "reinitialiserRacine"]
        fn reinitialiser_racine(self: Pin<&mut Genealogie>);

        /// Recherche par saisie, limitée : `{total, liste: [{id, nom, nomAffiche, dates, lieu, sexe}]}`.
        /// Chaque mot de la requête doit figurer (nom, prénoms, identifiant, lieux, dates).
        #[qinvokable]
        #[cxx_name = "chercherPersonnes"]
        fn chercher_personnes(&self, requete: &QString, limite: i32) -> QString;

        /// Fiche d'une personne, pour l'édition : identité, parents, unions, conjoints, enfants.
        #[qinvokable]
        #[cxx_name = "fichePersonne"]
        fn fiche_personne(&self, id: &QString) -> QString;

        /// Réécrit l'identité (nom, sexe, naissance, décès) ; rend `{ok, message}`.
        #[qinvokable]
        #[cxx_name = "modifierPersonne"]
        fn modifier_personne(self: Pin<&mut Genealogie>, id: &QString, champs: &QString) -> QString;

        /// Relie une personne existante : relation « pere », « mere », « parent »,
        /// « conjoint » ou « enfant » ; `famille` : union choisie (vide : union nouvelle).
        #[qinvokable]
        #[cxx_name = "lierProche"]
        fn lier_proche(self: Pin<&mut Genealogie>, id: &QString, relation: &QString, autre: &QString, famille: &QString) -> QString;

        /// Crée une personne et la relie dans la même écriture ; rend `{ok, message, id}`.
        #[qinvokable]
        #[cxx_name = "creerProche"]
        fn creer_proche(self: Pin<&mut Genealogie>, id: &QString, relation: &QString, champs: &QString, famille: &QString) -> QString;

        /// Donne à une personne ce père et cette mère (identifiants, vides : aucun).
        #[qinvokable]
        #[cxx_name = "definirParents"]
        fn definir_parents(self: Pin<&mut Genealogie>, id: &QString, pere: &QString, mere: &QString) -> QString;

        /// Retire le lien entre la personne et un proche ; aucune personne n'est supprimée.
        #[qinvokable]
        #[cxx_name = "retirerLien"]
        fn retirer_lien(self: Pin<&mut Genealogie>, id: &QString, relation: &QString, autre: &QString) -> QString;

        /// Ajoute une personne ; rend `{ok, message, id}`.
        #[qinvokable]
        #[cxx_name = "ajouterPersonne"]
        fn ajouter_personne(self: Pin<&mut Genealogie>, champs: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "supprimerPersonne"]
        fn supprimer_personne(self: Pin<&mut Genealogie>, id: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "annuler"]
        fn annuler(self: Pin<&mut Genealogie>) -> QString;

        #[qinvokable]
        #[cxx_name = "ordonnances"]
        fn ordonnances(&self, id: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "ajouterOrdonnance"]
        fn ajouter_ordonnance(self: Pin<&mut Genealogie>, id: &QString, champs: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "supprimerOrdonnance"]
        fn supprimer_ordonnance(self: Pin<&mut Genealogie>, id: &QString, index: i32) -> QString;

        #[qinvokable]
        #[cxx_name = "listesOrdonnances"]
        fn listes_ordonnances(&self) -> QString;

        /// Import d'un relevé CSV d'ordonnances accomplies (export FamilySearch) pour tout
        /// le fichier ; rend `{ok, message}` avec le bilan.
        #[qinvokable]
        #[cxx_name = "importerOrdonnances"]
        fn importer_ordonnances(self: Pin<&mut Genealogie>, url: &QString) -> QString;

        /// « Coller un relevé » : un tableau collé vaut pour tout le fichier, un relevé en
        /// texte libre pour la personne `id`.
        #[qinvokable]
        #[cxx_name = "collerReleve"]
        fn coller_releve(self: Pin<&mut Genealogie>, id: &QString, texte: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "pedigree"]
        fn pedigree(&self, id: &QString, haut: i32, bas: i32, zoom: f64) -> QString;

        #[qinvokable]
        #[cxx_name = "uriRappel"]
        fn uri_rappel(&self) -> QString;

        /// Démarre la connexion ; rend l'URL à ouvrir dans le navigateur (vide si échec).
        #[qinvokable]
        #[cxx_name = "connecter"]
        fn connecter(self: Pin<&mut Genealogie>, cle: &QString, env: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "deconnecter"]
        fn deconnecter(self: Pin<&mut Genealogie>);

        /// Importe l'ascendance FamilySearch de la personne locale `id`.
        #[qinvokable]
        #[cxx_name = "importer"]
        fn importer(self: Pin<&mut Genealogie>, id: &QString, fsid: &QString, generations: i32);

        /// Recherche d'ascendance : état général (JSON).
        #[qinvokable]
        #[cxx_name = "recherche"]
        fn etat_recherche(&self) -> QString;

        /// Lance (ou actualise) la recherche depuis une personne.
        #[qinvokable]
        #[cxx_name = "rechercheLancer"]
        fn recherche_lancer(self: Pin<&mut Genealogie>, id: &QString, generations: i32) -> QString;

        /// Détail d'une personne à compléter : candidats, tentatives, liens de recherche.
        #[qinvokable]
        #[cxx_name = "rechercheTravail"]
        fn recherche_travail(&self, personne: &QString) -> QString;

        /// Ajoute un candidat relevé à la main dans un acte.
        #[qinvokable]
        #[cxx_name = "rechercheSaisir"]
        fn recherche_saisir(self: Pin<&mut Genealogie>, personne: &QString, champs: &QString) -> QString;

        /// « accepter » (écrit le lien dans le GEDCOM), « rejeter » ou « attente ».
        #[qinvokable]
        #[cxx_name = "rechercheDecider"]
        fn recherche_decider(self: Pin<&mut Genealogie>, personne: &QString, candidat: &QString, action: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "rechercheSansIssue"]
        fn recherche_sans_issue(self: Pin<&mut Genealogie>, personne: &QString, valeur: bool) -> QString;

        /// Interroge l'arbre FamilySearch (session ouverte) ; résultat dans `message`.
        #[qinvokable]
        #[cxx_name = "rechercheFamilySearch"]
        fn recherche_familysearch(self: Pin<&mut Genealogie>, personne: &QString);
    }

    impl cxx_qt::Threading for Genealogie {}
}

use crate::edition::{self, Relation};
use crate::familysearch::{self as fs, Environnement};
use crate::fiches::{self, Vue};
use crate::gedcom::{self, Document, Identite};
use crate::modele::{cle_numerique, correspond, dates_courtes, libelle_numeros, mots_requete, plier, Arbre, Personne, Tri};
use crate::ordonnances::{self, Ordonnance, Registre};
use crate::recherche::{self, Candidat, Recherche, Role, Source};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct GenealogieRust {
    fichier: QString,
    resume: QString,
    revision: i32,
    annulable: bool,
    racine: QString,
    connecte: bool,
    occupe: bool,
    message: QString,
    version: QString,
    chemin: Option<PathBuf>,
    arbre: Arbre,
    numeros: HashMap<String, Vec<u64>>,
    registre: Registre,
    jeton: Option<String>,
    env: Environnement,
    recherche: Option<Recherche>,
}

impl Default for GenealogieRust {
    fn default() -> Self {
        Self {
            fichier: QString::from(""),
            resume: QString::from("Aucun fichier GEDCOM chargé"),
            revision: 0,
            annulable: false,
            racine: QString::from(""),
            connecte: false,
            occupe: false,
            message: QString::from(""),
            version: QString::from(env!("CARGO_PKG_VERSION")),
            chemin: None,
            arbre: Arbre::default(),
            numeros: HashMap::new(),
            registre: Registre::new(),
            jeton: None,
            env: Environnement::Integration,
            recherche: None,
        }
    }
}

/// Convertit une URL de fichier (ce que rend `FileDialog`) en chemin local.
pub fn chemin_depuis_url(url: &str) -> String {
    let reste = match url.strip_prefix("file://") {
        Some(r) => match r.strip_prefix('/') {
            Some(c) => c.to_string(),
            None => format!("//{r}"),
        },
        None => return url.to_string(),
    };
    let o = reste.as_bytes();
    let mut s = Vec::with_capacity(o.len());
    let mut i = 0;
    while i < o.len() {
        if o[i] == b'%' && i + 2 < o.len() {
            if let (Some(a), Some(b)) = ((o[i + 1] as char).to_digit(16), (o[i + 2] as char).to_digit(16)) {
                s.push((a * 16 + b) as u8);
                i += 3;
                continue;
            }
        }
        s.push(o[i]);
        i += 1;
    }
    let chemin = String::from_utf8_lossy(&s).into_owned();
    let b = chemin.as_bytes();
    let lecteur = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
    if lecteur || chemin.starts_with("//") { chemin } else { format!("/{chemin}") }
}

fn q(s: &str) -> QString {
    QString::from(s)
}

fn resultat(ok: bool, message: &str) -> QString {
    q(&json!({"ok": ok, "message": message}).to_string())
}

fn issue(r: Result<String, String>) -> QString {
    match r {
        Ok(m) => resultat(true, &m),
        Err(e) => resultat(false, &e),
    }
}

/// Champs d'identité saisis dans un formulaire (JSON).
fn identite_depuis(c: &Value) -> Identite {
    let t = |k: &str| c[k].as_str().unwrap_or("").to_string();
    Identite {
        prenoms: t("prenoms"),
        nom: t("nom"),
        sexe: t("sexe"),
        naissance: t("naissance"),
        lieu_naissance: t("lieuNaissance"),
        deces: t("deces"),
        lieu_deces: t("lieuDeces"),
    }
}

fn non_vide(s: String) -> Option<String> {
    let s = s.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn relation_depuis(t: &QString) -> Result<Relation, String> {
    Relation::depuis(&t.to_string()).ok_or_else(|| "Relation inconnue.".to_string())
}

fn personne_json(p: &Personne) -> Value {
    json!({"id": p.id, "nom": p.nom_affiche(), "nomListe": p.nom_liste(), "dates": dates_courtes(p), "sexe": p.sexe})
}

impl GenealogieRust {
    fn recalculer(&mut self) {
        let racine = self.racine.to_string();
        self.numeros = if racine.is_empty() { HashMap::new() } else { self.arbre.ahnentafel(&racine) };
    }

    fn resume_courant(&self) -> String {
        format!("{} personnes — {} familles", self.arbre.personnes.len(), self.arbre.familles.len())
    }
}

impl qobject::Genealogie {
    fn apres_changement(mut self: Pin<&mut Self>, resume: Option<String>) {
        let texte = {
            let mut r = self.as_mut().rust_mut();
            r.recalculer();
            resume.unwrap_or_else(|| r.resume_courant())
        };
        self.as_mut().set_resume(q(&texte));
        let rev = *self.revision() + 1;
        self.as_mut().set_revision(rev);
    }

    /// Charge le fichier et remet l'état à neuf. `racine` : conserver la personne N° 1.
    fn charger(mut self: Pin<&mut Self>, chemin: &Path, garder_racine: bool) -> Result<(), String> {
        let octets = std::fs::read(chemin).map_err(|e| format!("Impossible de lire le fichier :\n\n{e}"))?;
        let arbre = gedcom::analyser(&gedcom::decoder(&octets));
        let ancienne = self.racine().to_string();
        let racine = if garder_racine && arbre.personne(&ancienne).is_some() {
            ancienne
        } else {
            arbre.personnes.first().map(|p| p.id.clone()).unwrap_or_default()
        };
        {
            let mut r = self.as_mut().rust_mut();
            r.registre = ordonnances::charger(chemin);
            let mut suivi = recherche::charger(chemin);
            if let Some(s) = suivi.as_mut() {
                s.actualiser(&arbre);
            }
            r.recherche = suivi;
            r.arbre = arbre;
            r.chemin = Some(chemin.to_path_buf());
        }
        let nom = chemin.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.as_mut().set_fichier(q(&nom));
        self.as_mut().set_racine(q(&racine));
        self.apres_changement(None);
        Ok(())
    }

    pub fn ouvrir(mut self: Pin<&mut Self>, url: &QString) -> QString {
        let chemin = PathBuf::from(chemin_depuis_url(&url.to_string()));
        match self.as_mut().charger(&chemin, false) {
            Ok(()) => {
                self.as_mut().set_annulable(false);
                q("")
            }
            Err(e) => q(&e),
        }
    }

    pub fn nouveau(mut self: Pin<&mut Self>, url: &QString) -> QString {
        let chemin = PathBuf::from(chemin_depuis_url(&url.to_string()));
        if let Err(e) = std::fs::write(&chemin, gedcom::ENTETE_NOUVEAU.replace('\n', "\r\n")) {
            return q(&format!("Impossible de créer le fichier GEDCOM :\n\n{e}"));
        }
        match self.as_mut().charger(&chemin, false) {
            Ok(()) => {
                self.as_mut().set_annulable(false);
                self.as_mut().set_resume(q("Nouveau fichier GEDCOM — prêt à créer la première personne"));
                q("")
            }
            Err(e) => q(&e),
        }
    }

    pub fn enregistrer_sous(mut self: Pin<&mut Self>, url: &QString) -> QString {
        let Some(actuel) = self.rust().chemin.clone() else { return q("Aucun fichier GEDCOM n'est actuellement ouvert.") };
        let cible = PathBuf::from(chemin_depuis_url(&url.to_string()));
        if cible == actuel {
            return q("");
        }
        let doc = match Document::lire(&actuel) {
            Ok(d) => d,
            Err(e) => return q(&format!("Impossible de lire le fichier :\n\n{e}")),
        };
        if let Err(e) = std::fs::write(&cible, doc.texte()) {
            return q(&format!("Impossible d'enregistrer le fichier :\n\n{e}"));
        }
        let registre = self.rust().registre.clone();
        let _ = ordonnances::enregistrer(&cible, &registre);
        if let Some(s) = self.rust().recherche.clone() {
            let _ = recherche::enregistrer(&cible, &s);
        }
        self.as_mut().rust_mut().chemin = Some(cible.clone());
        let nom = cible.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.as_mut().set_fichier(q(&nom));
        self.as_mut().set_annulable(false);
        q("")
    }

    pub fn fermer(mut self: Pin<&mut Self>) {
        {
            let mut r = self.as_mut().rust_mut();
            r.chemin = None;
            r.arbre = Arbre::default();
            r.registre.clear();
            r.recherche = None;
        }
        self.as_mut().set_fichier(q(""));
        self.as_mut().set_racine(q(""));
        self.as_mut().set_annulable(false);
        self.apres_changement(Some("Aucun fichier GEDCOM chargé".into()));
    }

    pub fn chemin_courant(&self) -> QString {
        q(&self.rust().chemin.as_ref().map(|c| c.to_string_lossy().into_owned()).unwrap_or_default())
    }

    pub fn personnes(&self, requete: &QString, tri: &QString) -> QString {
        let r = self.rust();
        let liste: Vec<Value> = r
            .arbre
            .rechercher(&requete.to_string(), Tri::depuis(&tri.to_string()), &r.numeros)
            .into_iter()
            .map(|p| json!({"id": p.id, "numero": libelle_numeros(&r.numeros, &p.id), "nom": p.nom_liste()}))
            .collect();
        q(&Value::Array(liste).to_string())
    }

    pub fn nom_de(&self, id: &QString) -> QString {
        q(&self.rust().arbre.personne(&id.to_string()).map(Personne::nom_affiche).unwrap_or_default())
    }

    pub fn fsid_de(&self, id: &QString) -> QString {
        q(&self.rust().arbre.personne(&id.to_string()).map(|p| p.fsid.clone()).unwrap_or_default())
    }

    fn rendu(&self, id: &str, vue: &str) -> (String, String) {
        let r = self.rust();
        let ords = r.registre.get(id).cloned().unwrap_or_default();
        fiches::rendre(&r.arbre, id, Vue::depuis(vue), &ords)
    }

    pub fn presentation(&self, id: &QString, vue: &QString) -> QString {
        q(&self.rendu(&id.to_string(), &vue.to_string()).0)
    }

    pub fn exporter(&self, url: &QString, id: &QString, vue: &QString) -> QString {
        let texte = self.rendu(&id.to_string(), &vue.to_string()).1;
        if texte.trim().is_empty() {
            return q("Aucun résultat à exporter.");
        }
        let chemin = chemin_depuis_url(&url.to_string());
        match std::fs::write(&chemin, texte.replace('\n', "\r\n")) {
            Ok(()) => q(""),
            Err(e) => q(&e.to_string()),
        }
    }

    pub fn definir_racine(mut self: Pin<&mut Self>, id: &QString) {
        let nom = self.rust().arbre.personne(&id.to_string()).map(Personne::nom_affiche);
        if let Some(nom) = nom {
            self.as_mut().set_racine(id.clone());
            self.apres_changement(Some(format!("Ahnentafel : {nom} est la personne N° 1")));
        }
    }

    pub fn reinitialiser_racine(mut self: Pin<&mut Self>) {
        self.as_mut().set_racine(q(""));
        self.apres_changement(Some("Numérotation Ahnentafel réinitialisée".into()));
    }

    pub fn chercher_personnes(&self, requete: &QString, limite: i32) -> QString {
        let mots = mots_requete(&requete.to_string());
        if mots.is_empty() {
            return q(&json!({"total": 0, "liste": []}).to_string());
        }
        let mut v: Vec<&Personne> = self.rust().arbre.personnes.iter().filter(|p| correspond(p, &mots)).collect();
        let total = v.len();
        v.sort_by_cached_key(|p| (plier(&p.nom_liste()), cle_numerique(&p.id)));
        v.truncate(limite.max(1) as usize);
        let liste: Vec<Value> = v
            .iter()
            .map(|p| {
                let mut j = personne_json(p);
                j["lieu"] = json!(p.lieu_naissance);
                j
            })
            .collect();
        q(&json!({"total": total, "liste": liste}).to_string())
    }

    pub fn fiche_personne(&self, id: &QString) -> QString {
        let r = self.rust();
        let a = &r.arbre;
        let Some(p) = a.personne(&id.to_string()) else { return q("{}") };
        let ident = Identite::de(p);
        let mut parents = Vec::new();
        for f in p.familles_enfant.iter().filter_map(|f| a.famille(f)) {
            for (x, role) in [(&f.mari, "Père"), (&f.epouse, "Mère")] {
                if let Some(x) = x.as_deref().and_then(|x| a.personne(x)) {
                    let mut j = personne_json(x);
                    j["role"] = json!(role);
                    j["famille"] = json!(f.id);
                    parents.push(j);
                }
            }
        }
        let unions = edition::unions(a, p);
        let mut conjoints = Vec::new();
        let mut enfants = Vec::new();
        let mut liste_unions = Vec::new();
        for u in &unions {
            let conjoint = u.conjoint.as_deref().and_then(|c| a.personne(c));
            let qui = conjoint.map_or_else(|| "sans conjoint".to_string(), |c| format!("avec {}", c.nom_affiche()));
            liste_unions.push(json!({
                "famille": u.famille, "conjoint": u.conjoint, "enfants": u.enfants, "mariage": u.mariage,
                "libelle": format!("{qui} — {} enfant(s) [{}]", u.enfants, u.famille),
            }));
            if let Some(c) = conjoint {
                let mut j = personne_json(c);
                j["famille"] = json!(u.famille);
                j["mariage"] = json!(u.mariage);
                j["enfants"] = json!(u.enfants);
                conjoints.push(j);
            }
            if let Some(f) = a.famille(&u.famille) {
                for e in f.enfants.iter().filter_map(|e| a.personne(e)) {
                    let mut j = personne_json(e);
                    j["famille"] = json!(u.famille);
                    j["autreParent"] = json!(conjoint.map(|c| c.nom_affiche()).unwrap_or_default());
                    enfants.push(j);
                }
            }
        }
        let origine = edition::famille_origine(a, p);
        let place = |x: Option<&String>| x.and_then(|x| a.personne(x)).map(personne_json).unwrap_or(Value::Null);
        let fratrie: Vec<Value> = a.fratrie(p).into_iter().map(personne_json).collect();
        q(&json!({
            "id": p.id,
            "nomAffiche": p.nom_affiche(),
            "dates": dates_courtes(p),
            "prenoms": ident.prenoms, "nom": ident.nom, "sexe": ident.sexe,
            "naissance": ident.naissance, "lieuNaissance": ident.lieu_naissance,
            "deces": ident.deces, "lieuDeces": ident.lieu_deces,
            "mariage": p.mariage, "lieuMariage": p.lieu_mariage,
            "fsid": p.fsid,
            "notes": p.notes,
            "numero": libelle_numeros(&r.numeros, &p.id),
            "pere": place(origine.and_then(|f| f.mari.as_ref())),
            "mere": place(origine.and_then(|f| f.epouse.as_ref())),
            "parents": parents,
            "unions": liste_unions,
            "conjoints": conjoints,
            "enfants": enfants,
            "fratrie": fratrie,
        })
        .to_string())
    }

    /// Applique une modification au fichier : rien n'est écrit si elle échoue ou ne
    /// change rien ; sinon le fichier est réécrit après sa copie `.bak`, puis relu.
    fn modifier_document(
        mut self: Pin<&mut Self>,
        operation: impl FnOnce(&mut Document) -> Result<String, String>,
    ) -> Result<String, String> {
        let mut doc = self.document()?;
        let avant = doc.lignes.clone();
        let message = operation(&mut doc)?;
        if doc.lignes != avant {
            self.as_mut().ecrire(&doc, None).map_err(|e| format!("Modification non enregistrée :\n\n{e}"))?;
            let r = format!("{} — modification enregistrée (copie .bak faite avant l'écriture)", self.rust().resume_courant());
            self.as_mut().set_resume(q(&r));
        }
        Ok(message)
    }

    pub fn modifier_personne(self: Pin<&mut Self>, id: &QString, champs: &QString) -> QString {
        let id = id.to_string();
        let identite = identite_depuis(&serde_json::from_str(&champs.to_string()).unwrap_or_default());
        issue(self.modifier_document(|doc| {
            Ok(if edition::modifier_identite(doc, &id, &identite)? {
                "Fiche enregistrée.".to_string()
            } else {
                "Aucun changement à enregistrer.".to_string()
            })
        }))
    }

    pub fn lier_proche(self: Pin<&mut Self>, id: &QString, relation: &QString, autre: &QString, famille: &QString) -> QString {
        let relation = match relation_depuis(relation) {
            Ok(r) => r,
            Err(e) => return resultat(false, &e),
        };
        let (id, autre, famille) = (id.to_string(), autre.to_string(), non_vide(famille.to_string()));
        issue(self.modifier_document(|doc| edition::lier(doc, &id, relation, &autre, famille.as_deref())))
    }

    pub fn creer_proche(self: Pin<&mut Self>, id: &QString, relation: &QString, champs: &QString, famille: &QString) -> QString {
        let relation = match relation_depuis(relation) {
            Ok(r) => r,
            Err(e) => return resultat(false, &e),
        };
        let (id, famille) = (id.to_string(), non_vide(famille.to_string()));
        let identite = identite_depuis(&serde_json::from_str(&champs.to_string()).unwrap_or_default());
        let mut cree = String::new();
        let r = self.modifier_document(|doc| {
            let nouveau = edition::creer_personne(doc, &identite)?;
            let lien = edition::lier(doc, &id, relation, &nouveau, famille.as_deref())?;
            cree = nouveau.clone();
            Ok(format!("Personne créée : {} [{nouveau}].\n\n{lien}", identite.nettoyee()?.nom_gedcom().replace('/', "")))
        });
        match r {
            Ok(m) => q(&json!({"ok": true, "message": m, "id": cree}).to_string()),
            Err(e) => resultat(false, &e),
        }
    }

    pub fn definir_parents(self: Pin<&mut Self>, id: &QString, pere: &QString, mere: &QString) -> QString {
        let (id, pere, mere) = (id.to_string(), non_vide(pere.to_string()), non_vide(mere.to_string()));
        issue(self.modifier_document(|doc| edition::definir_parents(doc, &id, pere.as_deref(), mere.as_deref())))
    }

    pub fn retirer_lien(self: Pin<&mut Self>, id: &QString, relation: &QString, autre: &QString) -> QString {
        let relation = match relation_depuis(relation) {
            Ok(r) => r,
            Err(e) => return resultat(false, &e),
        };
        let (id, autre) = (id.to_string(), autre.to_string());
        issue(self.modifier_document(|doc| edition::retirer_lien(doc, &id, relation, &autre)))
    }

    /// Réécrit le fichier (avec sauvegarde .bak) puis le relit.
    fn ecrire(mut self: Pin<&mut Self>, doc: &Document, resume: Option<String>) -> Result<(), String> {
        let chemin = self.rust().chemin.clone().ok_or("Créez ou ouvrez d'abord un fichier GEDCOM.")?;
        gedcom::ecrire_avec_sauvegarde(&chemin, doc).map_err(|e| e.to_string())?;
        self.as_mut().charger(&chemin, true)?;
        self.as_mut().set_annulable(true);
        if let Some(r) = resume {
            self.as_mut().set_resume(q(&r));
        }
        Ok(())
    }

    fn document(&self) -> Result<Document, String> {
        let chemin = self.rust().chemin.clone().ok_or("Créez ou ouvrez d'abord un fichier GEDCOM.")?;
        Document::lire(&chemin).map_err(|e| e.to_string())
    }

    /// Nouvelle personne, reliée si on les donne à son père et à sa mère : la famille de
    /// ce couple exact est reprise si elle existe, sinon une famille nouvelle est créée.
    pub fn ajouter_personne(mut self: Pin<&mut Self>, champs: &QString) -> QString {
        let c: Value = serde_json::from_str(&champs.to_string()).unwrap_or_default();
        let identite = identite_depuis(&c);
        let pere = non_vide(c["pere"].as_str().unwrap_or("").to_string());
        let mere = non_vide(c["mere"].as_str().unwrap_or("").to_string());
        let mut id = String::new();
        let r = self.as_mut().modifier_document(|doc| {
            let nouveau = edition::creer_personne(doc, &identite)?;
            if pere.is_some() || mere.is_some() {
                edition::definir_parents(doc, &nouveau, pere.as_deref(), mere.as_deref())?;
            }
            id = nouveau;
            Ok(String::new())
        });
        if let Err(e) = r {
            return resultat(false, &format!("Impossible d'ajouter la personne :\n\n{e}"));
        }
        let r = self.rust();
        let mut rel = Vec::new();
        if let Some(x) = pere.as_deref().and_then(|i| r.arbre.personne(i)) {
            rel.push(format!("Père : {}", x.nom_affiche()));
        }
        if let Some(x) = mere.as_deref().and_then(|i| r.arbre.personne(i)) {
            rel.push(format!("Mère : {}", x.nom_affiche()));
        }
        let rel = if rel.is_empty() { "Aucun parent relié".to_string() } else { rel.join("\n") };
        let nom = r.arbre.personne(&id).map(Personne::nom_affiche).unwrap_or_default();
        q(&json!({"ok": true, "id": id,
                  "message": format!("Personne ajoutée :\n{nom}\n\nID : {id}\n\n{rel}")}).to_string())
    }

    pub fn supprimer_personne(mut self: Pin<&mut Self>, id: &QString) -> QString {
        let id = id.to_string();
        let mut doc = match self.document() {
            Ok(d) => d,
            Err(e) => return q(&e),
        };
        doc.supprimer_personne(&id);
        match self.as_mut().ecrire(&doc, None) {
            Ok(()) => q(""),
            Err(e) => q(&format!("Impossible de supprimer la personne :\n\n{e}")),
        }
    }

    /// Restaure le fichier tel qu'il était avant la dernière modification.
    pub fn annuler(mut self: Pin<&mut Self>) -> QString {
        let Some(chemin) = self.rust().chemin.clone() else { return q("Aucun fichier GEDCOM n'est actuellement ouvert.") };
        let bak = gedcom::sauvegarde(&chemin);
        if !*self.annulable() || !bak.is_file() {
            return q("Aucune modification récente ne peut être annulée.");
        }
        if let Err(e) = std::fs::copy(&bak, &chemin) {
            return q(&format!("Impossible d'annuler :\n\n{e}"));
        }
        if let Err(e) = self.as_mut().charger(&chemin, true) {
            return q(&e);
        }
        self.as_mut().set_annulable(false);
        let r = format!("{} — dernière modification annulée", self.rust().resume_courant());
        self.as_mut().set_resume(q(&r));
        q("")
    }

    pub fn ordonnances(&self, id: &QString) -> QString {
        let l = self.rust().registre.get(&id.to_string()).cloned().unwrap_or_default();
        let v: Vec<Value> = l.iter().map(|o| json!({"type": o.genre, "statut": o.statut, "date": o.date, "temple": o.temple})).collect();
        q(&Value::Array(v).to_string())
    }

    fn sauver_registre(mut self: Pin<&mut Self>) -> QString {
        let Some(chemin) = self.rust().chemin.clone() else { return q("Aucun fichier GEDCOM n'est ouvert.") };
        let registre = self.rust().registre.clone();
        let e = ordonnances::enregistrer(&chemin, &registre).err().map(|e| e.to_string()).unwrap_or_default();
        let rev = *self.revision() + 1;
        self.as_mut().set_revision(rev);
        q(&e)
    }

    pub fn ajouter_ordonnance(mut self: Pin<&mut Self>, id: &QString, champs: &QString) -> QString {
        let c: Value = serde_json::from_str(&champs.to_string()).unwrap_or_default();
        let t = |k: &str| c[k].as_str().unwrap_or("").trim().to_string();
        let o = Ordonnance { genre: t("type"), statut: t("statut"), date: t("date"), temple: t("temple"), notes: String::new() };
        self.as_mut().rust_mut().registre.entry(id.to_string()).or_default().push(o);
        self.sauver_registre()
    }

    pub fn supprimer_ordonnance(mut self: Pin<&mut Self>, id: &QString, index: i32) -> QString {
        {
            let mut r = self.as_mut().rust_mut();
            if let Some(l) = r.registre.get_mut(&id.to_string()) {
                if index >= 0 && (index as usize) < l.len() {
                    l.remove(index as usize);
                }
            }
        }
        self.sauver_registre()
    }

    pub fn listes_ordonnances(&self) -> QString {
        q(&json!({"types": ordonnances::TYPES, "statuts": ordonnances::STATUTS}).to_string())
    }

    /// Verse un relevé dans le registre et l'enregistre (copie `.bak` du registre avant).
    fn verser_releve(mut self: Pin<&mut Self>, verser: impl FnOnce(&mut Registre, &Arbre) -> ordonnances::Bilan) -> QString {
        let Some(chemin) = self.rust().chemin.clone() else { return resultat(false, "Créez ou ouvrez d'abord un fichier GEDCOM.") };
        let mut registre = self.rust().registre.clone();
        let bilan = verser(&mut registre, &self.rust().arbre);
        if bilan.modifie() {
            if let Err(e) = ordonnances::enregistrer_avec_sauvegarde(&chemin, &registre) {
                return resultat(false, &format!("Ordonnances non enregistrées :\n\n{e}"));
            }
            self.as_mut().rust_mut().registre = registre;
            let rev = *self.revision() + 1;
            self.as_mut().set_revision(rev);
        }
        let mut texte = bilan.texte();
        if bilan.modifie() {
            texte += "\n\nLe registre précédent a été copié dans « .ordonnances.json.bak ».";
        }
        resultat(true, &texte)
    }

    pub fn importer_ordonnances(self: Pin<&mut Self>, url: &QString) -> QString {
        let chemin = chemin_depuis_url(&url.to_string());
        let texte = match std::fs::read(&chemin) {
            Ok(o) => gedcom::decoder(&o),
            Err(e) => return resultat(false, &format!("Impossible de lire le fichier :\n\n{e}")),
        };
        match ordonnances::lire_tableau(&texte) {
            Ok(lignes) if lignes.is_empty() => resultat(false, "Aucune ordonnance trouvée dans ce fichier."),
            Ok(lignes) => self.verser_releve(|r, a| ordonnances::appliquer(r, a, &lignes)),
            Err(e) => resultat(false, &e),
        }
    }

    pub fn coller_releve(self: Pin<&mut Self>, id: &QString, texte: &QString) -> QString {
        let (id, texte) = (id.to_string(), texte.to_string());
        if let Ok(lignes) = ordonnances::lire_tableau(&texte) {
            if !lignes.is_empty() {
                return self.verser_releve(|r, a| ordonnances::appliquer(r, a, &lignes));
            }
        }
        if self.rust().arbre.personne(&id).is_none() {
            return resultat(false, "Sélectionnez d'abord la personne à qui appartient ce relevé.");
        }
        let liste = ordonnances::lire_releve(&texte);
        if liste.is_empty() {
            return resultat(false, "Aucune ordonnance reconnue dans le texte collé (Baptême, Confirmation, Initiatoire, Dotation, Scellement…).");
        }
        self.verser_releve(|r, _| ordonnances::appliquer_releve(r, &id, liste))
    }

    pub fn pedigree(&self, id: &QString, haut: i32, bas: i32, zoom: f64) -> QString {
        let v = crate::pedigree::mise_en_page(&self.rust().arbre, &id.to_string(), haut.max(0) as usize, bas.max(0) as usize, zoom);
        q(&v.to_string())
    }

    pub fn uri_rappel(&self) -> QString {
        q(fs::URI_RAPPEL)
    }

    pub fn connecter(mut self: Pin<&mut Self>, cle: &QString, env: &QString) -> QString {
        let cle = cle.to_string().trim().to_string();
        if cle.is_empty() {
            self.as_mut().set_message(q("Veuillez saisir l'App Key."));
            return q("");
        }
        if *self.occupe() {
            self.as_mut().set_message(q("Un échange avec FamilySearch est déjà en cours."));
            return q("");
        }
        let env = Environnement::depuis(&env.to_string());
        let ecoute = match fs::ecouter() {
            Ok(e) => e,
            Err(e) => {
                self.as_mut().set_message(q(&e));
                return q("");
            }
        };
        let connexion = fs::Connexion::preparer(env, &cle);
        let url = connexion.url.clone();
        self.as_mut().rust_mut().env = env;
        self.as_mut().set_occupe(true);
        self.as_mut().set_message(q("Connexion en cours : terminez-la dans le navigateur…"));
        let fil = self.qt_thread();
        std::thread::spawn(move || {
            let issue = fs::attendre_code(ecoute, &connexion.etat, Duration::from_secs(300))
                .and_then(|code| fs::echanger(&connexion, &code))
                .map(|jeton| {
                    let nom = fs::utilisateur(connexion.env, &jeton).map(|u| u.0).unwrap_or_default();
                    (jeton, nom)
                });
            let _ = fil.queue(move |mut o: Pin<&mut qobject::Genealogie>| {
                o.as_mut().set_occupe(false);
                match issue {
                    Ok((jeton, nom)) => {
                        o.as_mut().rust_mut().jeton = Some(jeton);
                        o.as_mut().set_connecte(true);
                        let env = o.rust().env.libelle();
                        let qui = if nom.is_empty() { String::new() } else { format!(" en tant que {nom}") };
                        o.as_mut().set_message(q(&format!("✓ Connecté à FamilySearch{qui} — environnement {env}.")));
                    }
                    Err(e) => o.as_mut().set_message(q(&e)),
                }
            });
        });
        q(&url)
    }

    pub fn deconnecter(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().jeton = None;
        self.as_mut().set_connecte(false);
        self.as_mut().set_message(q("Session FamilySearch fermée."));
    }

    pub fn importer(mut self: Pin<&mut Self>, id: &QString, fsid: &QString, generations: i32) {
        let id = id.to_string();
        let Some(jeton) = self.rust().jeton.clone() else {
            self.as_mut().set_message(q("Connectez-vous d'abord à FamilySearch."));
            return;
        };
        if self.rust().arbre.personne(&id).is_none() {
            self.as_mut().set_message(q("Sélectionnez d'abord une personne."));
            return;
        }
        let mut fsid = fsid.to_string().trim().to_uppercase();
        if fsid.is_empty() {
            fsid = self.rust().arbre.personne(&id).map(|p| p.fsid.clone()).unwrap_or_default();
        }
        if fsid.is_empty() {
            self.as_mut().set_message(q("Cette personne n'a pas d'identifiant FamilySearch : saisissez-le (ex. KWCB-QZ3)."));
            return;
        }
        let env = self.rust().env;
        self.as_mut().set_occupe(true);
        self.as_mut().set_message(q(&format!("Import de l'ascendance de {fsid} en cours…")));
        let fil = self.qt_thread();
        std::thread::spawn(move || {
            let issue = fs::ascendance(env, &jeton, &fsid, generations.max(1) as u32);
            let _ = fil.queue(move |mut o: Pin<&mut qobject::Genealogie>| {
                o.as_mut().set_occupe(false);
                let liste = match issue {
                    Ok(l) if l.is_empty() => {
                        o.as_mut().set_message(q("FamilySearch n'a rendu aucune personne."));
                        return;
                    }
                    Ok(l) => l,
                    Err(e) => {
                        o.as_mut().set_message(q(&e));
                        return;
                    }
                };
                let mut doc = match o.document() {
                    Ok(d) => d,
                    Err(e) => {
                        o.as_mut().set_message(q(&e));
                        return;
                    }
                };
                let arbre = o.rust().arbre.clone();
                let b = fs::verser(&mut doc, &arbre, &id, &liste);
                let texte = format!(
                    "Import FamilySearch : {} personne(s) reçue(s), {} ajoutée(s), {} déjà présente(s), {} lien(s) créé(s){}.",
                    liste.len(), b.ajoutees, b.reconnues, b.liens,
                    if b.ecartes > 0 { format!(", {} lien(s) écarté(s) car la personne a déjà d'autres parents", b.ecartes) } else { String::new() }
                );
                match o.as_mut().ecrire(&doc, None) {
                    Ok(()) => o.as_mut().set_message(q(&texte)),
                    Err(e) => o.as_mut().set_message(q(&format!("Import non enregistré : {e}"))),
                }
            });
        });
    }

    // ---- Recherche d'ascendance ------------------------------------------------

    fn sauver_recherche(&self) -> Result<(), String> {
        let r = self.rust();
        let (Some(chemin), Some(s)) = (r.chemin.as_ref(), r.recherche.as_ref()) else { return Ok(()) };
        recherche::enregistrer(chemin, s).map_err(|e| format!("Suivi de la recherche non enregistré :\n\n{e}"))
    }

    fn signaler_recherche(mut self: Pin<&mut Self>) {
        let rev = *self.revision() + 1;
        self.as_mut().set_revision(rev);
    }

    pub fn etat_recherche(&self) -> QString {
        let r = self.rust();
        let Some(s) = r.recherche.as_ref() else { return q(&json!({"active": false}).to_string()) };
        let mut a_faire = 0;
        let travaux: Vec<Value> = s
            .travaux
            .iter()
            .map(|t| {
                let p = r.arbre.personne(&t.personne);
                let manque = p.map(|p| recherche::manquants(&r.arbre, p)).unwrap_or_default();
                if p.is_some() && !manque.is_empty() && !t.sans_issue {
                    a_faire += 1;
                }
                json!({
                    "personne": t.personne,
                    "nom": p.map(|p| p.nom_affiche()).unwrap_or_else(|| format!("{} (absente du fichier)", t.personne)),
                    "naissance": p.map(|p| p.naissance.clone()).unwrap_or_default(),
                    "generation": t.generation,
                    "manque": recherche::libelle_manque(&manque),
                    "complet": p.is_some() && manque.is_empty(),
                    "sansIssue": t.sans_issue,
                    "candidats": t.candidats.len(),
                    "ouverts": t.candidats.iter().filter(|c| c.etat.ouvert()).count(),
                })
            })
            .collect();
        q(&json!({
            "active": true,
            "depart": s.depart,
            "departNom": r.arbre.personne(&s.depart).map(|p| p.nom_affiche()).unwrap_or_default(),
            "generationsMax": s.generations_max,
            "aFaire": a_faire,
            "travaux": travaux,
        })
        .to_string())
    }

    pub fn recherche_lancer(mut self: Pin<&mut Self>, id: &QString, generations: i32) -> QString {
        let id = id.to_string();
        if self.rust().chemin.is_none() {
            return resultat(false, "Créez ou ouvrez d'abord un fichier GEDCOM.");
        }
        if self.rust().arbre.personne(&id).is_none() {
            return resultat(false, "Sélectionnez d'abord une personne dans la liste principale.");
        }
        let g = generations.clamp(1, recherche::GENERATIONS_MAX as i32) as u32;
        let suivi = {
            let r = self.rust();
            match r.recherche.as_ref() {
                Some(s) if s.depart == id => {
                    let mut s = s.clone();
                    s.generations_max = g;
                    s.actualiser(&r.arbre);
                    s
                }
                _ => Recherche::nouvelle(&r.arbre, &id, g),
            }
        };
        let n = suivi.travaux.len();
        self.as_mut().rust_mut().recherche = Some(suivi);
        if let Err(e) = self.sauver_recherche() {
            return resultat(false, &e);
        }
        self.as_mut().signaler_recherche();
        let texte = if n == 0 {
            format!("Aucun parent manquant jusqu'à la génération {g}.")
        } else {
            format!("{n} personne(s) de l'ascendance à compléter, jusqu'à la génération {g}.")
        };
        resultat(true, &texte)
    }

    pub fn recherche_travail(&self, personne: &QString) -> QString {
        let r = self.rust();
        let id = personne.to_string();
        let (Some(s), Some(p)) = (r.recherche.as_ref(), r.arbre.personne(&id)) else { return q("{}") };
        let Some(t) = s.travail(&id) else { return q("{}") };
        let (pere, mere) = r.arbre.pere_mere(p);
        let manque = recherche::manquants(&r.arbre, p);
        let candidats: Vec<Value> = t
            .candidats
            .iter()
            .map(|c| {
                let sources: Vec<Value> = c
                    .sources
                    .iter()
                    .map(|x| json!({"description": x.description(), "url": x.url, "extrait": x.extrait, "consulteLe": x.consulte_le}))
                    .collect();
                json!({
                    "id": c.id,
                    "role": c.role.cle(),
                    "roleLibelle": c.role.libelle(),
                    "nom": c.nom_complet(),
                    "naissance": c.naissance,
                    "lieuNaissance": c.lieu_naissance,
                    "deces": c.deces,
                    "lieuDeces": c.lieu_deces,
                    "fsid": c.fsid,
                    "provenance": c.provenance,
                    "score": c.score,
                    "motifs": c.motifs,
                    "alertes": c.alertes,
                    "etat": c.etat.cle(),
                    "etatLibelle": c.etat.libelle(),
                    "ouvert": c.etat.ouvert(),
                    "placeLibre": manque.contains(&c.role),
                    "personne": c.personne,
                    "sources": sources,
                })
            })
            .collect();
        let tentatives: Vec<Value> =
            t.tentatives.iter().rev().take(30).map(|x| json!({"date": x.date, "source": x.source, "resultat": x.resultat})).collect();
        let liens: Vec<Value> = recherche::liens(p).into_iter().map(|(l, u)| json!({"libelle": l, "url": u})).collect();
        let manque: Vec<&str> = manque.iter().map(|m| m.cle()).collect();
        q(&json!({
            "personne": id,
            "nom": p.nom_affiche(),
            "naissance": p.naissance,
            "lieuNaissance": p.lieu_naissance,
            "deces": p.deces,
            "lieuDeces": p.lieu_deces,
            "generation": t.generation,
            "fsid": p.fsid,
            "pere": pere.map(|x| x.nom_affiche()).unwrap_or_default(),
            "mere": mere.map(|x| x.nom_affiche()).unwrap_or_default(),
            "manque": manque,
            "sansIssue": t.sans_issue,
            "prioritaire": recherche::prioritaire(p),
            "candidats": candidats,
            "tentatives": tentatives,
            "liens": liens,
        })
        .to_string())
    }

    pub fn recherche_saisir(mut self: Pin<&mut Self>, personne: &QString, champs: &QString) -> QString {
        let c: Value = serde_json::from_str(&champs.to_string()).unwrap_or_default();
        let t = |k: &str| c[k].as_str().unwrap_or("").trim().to_string();
        let Some(role) = Role::depuis(&t("role")) else { return resultat(false, "Indiquez s'il s'agit du père ou de la mère.") };
        let source = Source {
            organisme: t("organisme"),
            reference: t("reference"),
            commune: t("commune"),
            url: t("url"),
            consulte_le: recherche::aujourdhui(),
            extrait: t("extrait"),
        };
        let candidat = Candidat {
            role,
            prenoms: t("prenoms"),
            nom: t("nom"),
            naissance: t("naissance"),
            lieu_naissance: t("lieuNaissance"),
            deces: t("deces"),
            lieu_deces: t("lieuDeces"),
            provenance: "saisie".into(),
            sources: vec![source],
            ..Default::default()
        };
        let id = personne.to_string();
        let Some(mut s) = self.as_mut().rust_mut().recherche.take() else { return resultat(false, "Lancez d'abord une recherche.") };
        let issue = s.ajouter_candidat(&self.rust().arbre, &id, candidat);
        if let Ok((cid, _)) = &issue {
            s.noter(&id, "Saisie d'un acte", &format!("candidat {cid} relevé"));
        }
        self.as_mut().rust_mut().recherche = Some(s);
        match issue {
            Ok((cid, nouveau)) => {
                if let Err(e) = self.sauver_recherche() {
                    return resultat(false, &e);
                }
                self.as_mut().signaler_recherche();
                let texte = if nouveau {
                    format!("Candidat ajouté ({cid}) : examinez son contrôle de cohérence avant de l'accepter.")
                } else {
                    format!("Ce candidat figurait déjà ({cid}) : la nouvelle source lui a été ajoutée.")
                };
                resultat(true, &texte)
            }
            Err(e) => resultat(false, &e),
        }
    }

    pub fn recherche_decider(mut self: Pin<&mut Self>, personne: &QString, candidat: &QString, action: &QString) -> QString {
        let (id, cid, action) = (personne.to_string(), candidat.to_string(), action.to_string());
        let Some(mut s) = self.as_mut().rust_mut().recherche.take() else { return resultat(false, "Lancez d'abord une recherche.") };
        let issue: Result<String, String> = match action.as_str() {
            "rejeter" => s.rejeter(&id, &cid).map(|()| "Candidat écarté.".to_string()),
            "attente" => s.reexaminer(&id, &cid).map(|()| "Candidat remis à l'examen.".to_string()),
            "accepter" => match self.document() {
                Err(e) => Err(e),
                Ok(mut doc) => {
                    let arbre = self.rust().arbre.clone();
                    match recherche::accepter(&mut doc, &arbre, &mut s, &id, &cid) {
                        Err(e) => Err(e),
                        // ecrire() relit le fichier et son suivi depuis le disque : le suivi
                        // tenu en mémoire est remis juste après.
                        Ok(pid) => match self.as_mut().ecrire(&doc, None) {
                            Err(e) => Err(format!("Lien non enregistré :\n\n{e}")),
                            Ok(()) => {
                                s.actualiser(&self.rust().arbre);
                                let nom = self.rust().arbre.personne(&pid).map(|p| p.nom_affiche()).unwrap_or_default();
                                s.noter(&id, "Validation", &format!("{nom} [{pid}] accepté(e)"));
                                Ok(format!("Lien écrit dans le fichier : {nom} [{pid}].\n\nUne copie .bak a été faite avant l'écriture."))
                            }
                        },
                    }
                }
            },
            _ => Err("Action inconnue.".into()),
        };
        self.as_mut().rust_mut().recherche = Some(s);
        match issue {
            Ok(texte) => {
                if let Err(e) = self.sauver_recherche() {
                    return resultat(false, &e);
                }
                self.as_mut().signaler_recherche();
                resultat(true, &texte)
            }
            Err(e) => resultat(false, &e),
        }
    }

    pub fn recherche_sans_issue(mut self: Pin<&mut Self>, personne: &QString, valeur: bool) -> QString {
        let id = personne.to_string();
        let Some(mut s) = self.as_mut().rust_mut().recherche.take() else { return resultat(false, "Lancez d'abord une recherche.") };
        let issue = s.marquer_sans_issue(&id, valeur);
        if issue.is_ok() {
            s.noter(&id, "Décision", if valeur { "branche marquée sans issue" } else { "branche rouverte" });
        }
        self.as_mut().rust_mut().recherche = Some(s);
        match issue {
            Ok(()) => {
                if let Err(e) = self.sauver_recherche() {
                    return resultat(false, &e);
                }
                self.as_mut().signaler_recherche();
                resultat(true, "")
            }
            Err(e) => resultat(false, &e),
        }
    }

    pub fn recherche_familysearch(mut self: Pin<&mut Self>, personne: &QString) {
        let id = personne.to_string();
        let Some(jeton) = self.rust().jeton.clone() else {
            self.as_mut().set_message(q("Connectez-vous d'abord à FamilySearch (menu FamilySearch)."));
            return;
        };
        let fsid = self.rust().arbre.personne(&id).map(|p| p.fsid.clone()).unwrap_or_default();
        if fsid.is_empty() {
            self.as_mut().set_message(q("Cette personne n'a pas d'identifiant FamilySearch : utilisez la recherche guidée, puis relevez l'acte trouvé."));
            return;
        }
        if *self.occupe() {
            self.as_mut().set_message(q("Un échange avec FamilySearch est déjà en cours."));
            return;
        }
        let env = self.rust().env;
        self.as_mut().set_occupe(true);
        self.as_mut().set_message(q(&format!("Recherche des parents de {fsid} dans l'arbre FamilySearch…")));
        let fil = self.qt_thread();
        std::thread::spawn(move || {
            let issue = fs::ascendance(env, &jeton, &fsid, 1);
            let _ = fil.queue(move |mut o: Pin<&mut qobject::Genealogie>| {
                o.as_mut().set_occupe(false);
                let liste = match issue {
                    Ok(l) => l,
                    Err(e) => {
                        o.as_mut().set_message(q(&e));
                        return;
                    }
                };
                let Some(mut s) = o.as_mut().rust_mut().recherche.take() else { return };
                let arbre = o.rust().arbre.clone();
                let manque = arbre.personne(&id).map(|p| recherche::manquants(&arbre, p)).unwrap_or_default();
                let mut nouveaux = 0;
                for c in recherche::candidats_familysearch(&liste).into_iter().filter(|c| manque.contains(&c.role)) {
                    if let Ok((_, true)) = s.ajouter_candidat(&arbre, &id, c) {
                        nouveaux += 1;
                    }
                }
                let texte = if nouveaux == 0 {
                    "FamilySearch ne propose aucun nouveau parent pour cette personne.".to_string()
                } else {
                    format!("{nouveaux} candidat(s) proposé(s) par l'arbre FamilySearch : à confirmer par un acte avant de les accepter.")
                };
                s.noter(&id, "FamilySearch — arbre familial", &texte);
                o.as_mut().rust_mut().recherche = Some(s);
                let texte = match o.sauver_recherche() {
                    Ok(()) => texte,
                    Err(e) => e,
                };
                o.as_mut().set_message(q(&texte));
                o.as_mut().signaler_recherche();
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::chemin_depuis_url;

    #[test]
    fn urls_de_fichier() {
        assert_eq!(chemin_depuis_url("file:///home/manu/a%20b.ged"), "/home/manu/a b.ged");
        assert_eq!(chemin_depuis_url("file:///C:/G%C3%A9n%C3%A9alogie/x.ged"), "C:/Généalogie/x.ged");
        assert_eq!(chemin_depuis_url("file://serveur/part/x.ged"), "//serveur/part/x.ged");
    }
}
