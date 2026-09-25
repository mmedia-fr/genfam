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

        /// Personnes proposées comme père ou mère : `[{id, libelle}]`.
        #[qinvokable]
        #[cxx_name = "choixParents"]
        fn choix_parents(&self) -> QString;

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
    }

    impl cxx_qt::Threading for Genealogie {}
}

use crate::familysearch::{self as fs, Environnement};
use crate::fiches::{self, Vue};
use crate::gedcom::{self, Document};
use crate::modele::{libelle_numeros, Arbre, Personne, Tri};
use crate::ordonnances::{self, Ordonnance, Registre};
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

    pub fn choix_parents(&self) -> QString {
        let mut v: Vec<&Personne> = self.rust().arbre.personnes.iter().collect();
        v.sort_by_key(|p| p.nom_affiche().to_lowercase());
        let l: Vec<Value> = v.iter().map(|p| json!({"id": p.id, "libelle": format!("{} [{}]", p.nom_affiche(), p.id)})).collect();
        q(&Value::Array(l).to_string())
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

    pub fn ajouter_personne(mut self: Pin<&mut Self>, champs: &QString) -> QString {
        let c: Value = serde_json::from_str(&champs.to_string()).unwrap_or_default();
        let t = |k: &str| c[k].as_str().unwrap_or("").trim().to_string();
        let (prenoms, nom) = (t("prenoms"), t("nom"));
        if prenoms.is_empty() && nom.is_empty() {
            return resultat(false, "Indiquez au moins un prénom ou un nom.");
        }
        let (pere, mere) = (t("pere"), t("mere"));
        if !pere.is_empty() && pere == mere {
            return resultat(false, "Le père et la mère doivent être deux personnes différentes.");
        }
        let mut doc = match self.document() {
            Ok(d) => d,
            Err(e) => return resultat(false, &e),
        };
        let id = doc.prochain_id_personne();
        let mut p = Personne::nouvelle(&id);
        p.nom_gedcom = if nom.is_empty() { prenoms.clone() } else { format!("{prenoms} /{nom}/").trim().to_string() };
        p.prenoms = prenoms;
        p.nom = nom;
        p.sexe = t("sexe").to_uppercase();
        p.naissance = t("naissance");
        p.lieu_naissance = t("lieuNaissance");
        p.deces = t("deces");
        p.lieu_deces = t("lieuDeces");
        let opt = |s: &String| if s.is_empty() { None } else { Some(s.clone()) };
        let (pere, mere) = (opt(&pere), opt(&mere));
        doc.ajouter_personne(&p, pere.as_deref(), mere.as_deref());
        if let Err(e) = self.as_mut().ecrire(&doc, None) {
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
        q(&json!({"ok": true, "id": id,
                  "message": format!("Personne ajoutée :\n{}\n\nID : {id}\n\n{rel}", p.nom_affiche())}).to_string())
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
