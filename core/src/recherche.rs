// SPDX-License-Identifier: GPL-3.0-or-later
//! Recherche d'ascendance assistée.
//!
//! À partir d'une personne, la recherche dresse la liste des parents manquants
//! génération par génération (jusqu'à 20), rassemble des candidats — trouvés dans
//! l'arbre FamilySearch ou relevés à la main dans les archives —, contrôle leur
//! cohérence, et n'écrit un lien de parenté dans le GEDCOM qu'après validation,
//! avec ses sources.
//!
//! D'après le cahier des charges « Projet Appli Généalogie » de Claude Boisseau
//! (V1.2, octobre 2026) : validation humaine des filiations, méthode mixte
//! (automatique quand la source le permet, guidée sinon), états proposé / à vérifier
//! / accepté / rejeté, tentatives journalisées, reprise après interruption. Le travail
//! est conservé dans `<fichier>.recherche.json`, à côté du GEDCOM.

use crate::familysearch::{encoder, PersonneFs};
use crate::gedcom::Document;
use crate::modele::{Arbre, Personne};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Profondeur maximale retenue par le cahier des charges.
pub const GENERATIONS_MAX: u32 = 20;
/// Critère de priorité du cahier des charges : personnes nées depuis au moins 100 ans.
pub const ANS_PRIORITE: i32 = 100;

/// Place cherchée : le père ou la mère.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Pere,
    Mere,
}

impl Role {
    pub fn depuis(t: &str) -> Option<Self> {
        match t.trim() {
            "pere" | "père" | "Père" => Some(Self::Pere),
            "mere" | "mère" | "Mère" => Some(Self::Mere),
            _ => None,
        }
    }

    pub fn cle(self) -> &'static str {
        match self {
            Self::Pere => "pere",
            Self::Mere => "mere",
        }
    }

    pub fn libelle(self) -> &'static str {
        match self {
            Self::Pere => "Père",
            Self::Mere => "Mère",
        }
    }

    fn sexe(self) -> &'static str {
        match self {
            Self::Pere => "M",
            Self::Mere => "F",
        }
    }

    fn balise(self) -> &'static str {
        match self {
            Self::Pere => "HUSB",
            Self::Mere => "WIFE",
        }
    }
}

/// États de validation d'un lien de parenté (cahier des charges V1.2, § 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Etat {
    /// Aucune incohérence relevée : en attente de validation.
    #[default]
    Propose,
    /// Contrôle humain nécessaire : incohérence ou source absente.
    AVerifier,
    /// Lien validé et écrit dans le GEDCOM.
    Accepte,
    /// Lien écarté.
    Rejete,
}

impl Etat {
    pub fn cle(self) -> &'static str {
        match self {
            Self::Propose => "propose",
            Self::AVerifier => "a_verifier",
            Self::Accepte => "accepte",
            Self::Rejete => "rejete",
        }
    }

    pub fn libelle(self) -> &'static str {
        match self {
            Self::Propose => "Proposé",
            Self::AVerifier => "À vérifier",
            Self::Accepte => "Accepté",
            Self::Rejete => "Rejeté",
        }
    }

    /// Encore à décider.
    pub fn ouvert(self) -> bool {
        matches!(self, Self::Propose | Self::AVerifier)
    }
}

/// Référence documentaire d'un candidat : organisme, cote, commune, image, extrait.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Source {
    pub organisme: String,
    pub reference: String,
    pub commune: String,
    pub url: String,
    pub consulte_le: String,
    pub extrait: String,
}

impl Source {
    /// Description sur une ligne : ce que porte la citation GEDCOM.
    pub fn description(&self) -> String {
        [&self.organisme, &self.reference, &self.commune]
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn est_vide(&self) -> bool {
        self.description().is_empty() && self.url.trim().is_empty()
    }
}

/// Personne proposée comme père ou mère.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Candidat {
    pub id: String,
    pub role: Role,
    pub prenoms: String,
    pub nom: String,
    pub naissance: String,
    pub lieu_naissance: String,
    pub deces: String,
    pub lieu_deces: String,
    /// Identifiant de l'arbre FamilySearch, s'il est connu.
    pub fsid: String,
    /// « familysearch » ou « saisie ».
    pub provenance: String,
    pub sources: Vec<Source>,
    pub score: u8,
    /// Éléments concordants.
    pub motifs: Vec<String>,
    /// Incohérences ou manques : un seul suffit à demander une vérification.
    pub alertes: Vec<String>,
    pub etat: Etat,
    /// Personne du GEDCOM créée ou reprise à l'acceptation.
    pub personne: String,
}

impl Candidat {
    pub fn nom_complet(&self) -> String {
        format!("{} {}", self.prenoms.trim(), self.nom.trim()).trim().to_string()
    }

    fn etat_initial(&self) -> Etat {
        if self.alertes.is_empty() { Etat::Propose } else { Etat::AVerifier }
    }
}

/// Une recherche menée, même infructueuse : le cahier des charges demande de les garder.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tentative {
    pub date: String,
    pub source: String,
    pub resultat: String,
}

/// Parents à trouver pour une personne de l'ascendance.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Travail {
    /// Personne dont on cherche les parents (identifiant GEDCOM).
    pub personne: String,
    /// Génération de cette personne, la personne de départ étant la génération 1.
    pub generation: u32,
    /// Branche marquée sans issue : aucune source exploitable pour l'instant.
    pub sans_issue: bool,
    pub candidats: Vec<Candidat>,
    pub tentatives: Vec<Tentative>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Recherche {
    pub depart: String,
    pub generations_max: u32,
    pub travaux: Vec<Travail>,
}

impl Recherche {
    pub fn nouvelle(arbre: &Arbre, depart: &str, generations: u32) -> Self {
        let mut r = Self { depart: depart.to_string(), generations_max: generations.clamp(1, GENERATIONS_MAX), travaux: Vec::new() };
        r.actualiser(arbre);
        r
    }

    /// Recalcule les travaux d'après l'arbre : une entrée par ascendant de la personne
    /// de départ (elle comprise) à qui il manque le père ou la mère, tant que ces parents
    /// restent dans la limite de générations. Un travail déjà ouvert garde ses candidats
    /// et ses tentatives, même devenu complet : c'est l'historique des preuves. Un lien
    /// accepté qui a disparu du fichier (annulation) rouvre son candidat.
    pub fn actualiser(&mut self, arbre: &Arbre) {
        let mut file = VecDeque::from([(self.depart.clone(), 1u32)]);
        let mut vus = HashSet::new();
        while let Some((id, g)) = file.pop_front() {
            if !vus.insert(id.clone()) {
                continue;
            }
            let Some(p) = arbre.personne(&id) else { continue };
            if g >= self.generations_max {
                continue;
            }
            let (pere, mere) = arbre.pere_mere(p);
            if pere.is_none() || mere.is_none() {
                match self.travaux.iter_mut().find(|t| t.personne == id) {
                    Some(t) => t.generation = g,
                    None => self.travaux.push(Travail { personne: id.clone(), generation: g, ..Default::default() }),
                }
            }
            for x in [pere, mere].into_iter().flatten() {
                file.push_back((x.id.clone(), g + 1));
            }
        }
        for t in &mut self.travaux {
            let Some(p) = arbre.personne(&t.personne) else { continue };
            let manque = manquants(arbre, p);
            for c in &mut t.candidats {
                if c.etat == Etat::Accepte && manque.contains(&c.role) {
                    c.personne.clear();
                    c.etat = c.etat_initial();
                }
            }
        }
        self.travaux.sort_by(|a, b| a.generation.cmp(&b.generation).then_with(|| cle_id(&a.personne).cmp(&cle_id(&b.personne))));
    }

    pub fn travail(&self, personne: &str) -> Option<&Travail> {
        self.travaux.iter().find(|t| t.personne == personne)
    }

    fn travail_mut(&mut self, personne: &str) -> Result<&mut Travail, String> {
        self.travaux
            .iter_mut()
            .find(|t| t.personne == personne)
            .ok_or_else(|| "Cette personne ne fait pas partie de la recherche en cours.".to_string())
    }

    /// Ajoute un candidat après contrôle de cohérence. Un candidat déjà présent (même
    /// identifiant FamilySearch, ou mêmes nom et année de naissance) n'est pas dupliqué :
    /// ses nouvelles sources lui sont ajoutées. Rend son identifiant et s'il est nouveau.
    pub fn ajouter_candidat(&mut self, arbre: &Arbre, personne: &str, mut c: Candidat) -> Result<(String, bool), String> {
        let enfant = arbre.personne(personne).ok_or("Personne introuvable dans le fichier GEDCOM.")?;
        if c.nom_complet().is_empty() {
            return Err("Indiquez au moins un prénom ou un nom.".into());
        }
        if !manquants(arbre, enfant).contains(&c.role) {
            return Err(format!("{} a déjà un(e) {} dans le fichier.", enfant.nom_affiche(), c.role.libelle().to_lowercase()));
        }
        let t = self.travail_mut(personne)?;
        if let Some(d) = t.candidats.iter_mut().find(|x| x.role == c.role && meme_personne(x, &c)) {
            for s in c.sources {
                if !s.est_vide() && !d.sources.contains(&s) {
                    d.sources.push(s);
                }
            }
            if d.fsid.is_empty() {
                d.fsid = c.fsid;
            }
            let (score, motifs, alertes) = evaluer(enfant, d);
            d.score = score;
            d.motifs = motifs;
            d.alertes = alertes;
            if d.etat.ouvert() {
                d.etat = d.etat_initial();
            }
            return Ok((d.id.clone(), false));
        }
        c.sources.retain(|s| !s.est_vide());
        let (score, motifs, alertes) = evaluer(enfant, &c);
        c.score = score;
        c.motifs = motifs;
        c.alertes = alertes;
        c.etat = c.etat_initial();
        c.personne.clear();
        if c.provenance.is_empty() {
            c.provenance = "saisie".into();
        }
        c.id = format!("c{}", t.candidats.len() + 1);
        let id = c.id.clone();
        t.candidats.push(c);
        Ok((id, true))
    }

    pub fn noter(&mut self, personne: &str, source: &str, resultat: &str) {
        if let Ok(t) = self.travail_mut(personne) {
            t.tentatives.push(Tentative { date: aujourdhui(), source: source.into(), resultat: resultat.into() });
        }
    }

    pub fn rejeter(&mut self, personne: &str, candidat: &str) -> Result<(), String> {
        let c = candidat_mut(self.travail_mut(personne)?, candidat)?;
        if c.etat == Etat::Accepte {
            return Err("Ce lien est déjà écrit dans le fichier : supprimez-le depuis la fiche, ou annulez la dernière modification.".into());
        }
        c.etat = Etat::Rejete;
        Ok(())
    }

    /// Remet en attente un candidat rejeté.
    pub fn reexaminer(&mut self, personne: &str, candidat: &str) -> Result<(), String> {
        let c = candidat_mut(self.travail_mut(personne)?, candidat)?;
        if c.etat == Etat::Rejete {
            c.alertes.retain(|a| *a != AUTRE_ACCEPTE);
            c.etat = c.etat_initial();
        }
        Ok(())
    }

    pub fn marquer_sans_issue(&mut self, personne: &str, valeur: bool) -> Result<(), String> {
        self.travail_mut(personne)?.sans_issue = valeur;
        Ok(())
    }
}

const AUTRE_ACCEPTE: &str = "Un autre candidat a été accepté pour cette place.";

fn candidat_mut<'a>(t: &'a mut Travail, id: &str) -> Result<&'a mut Candidat, String> {
    t.candidats.iter_mut().find(|c| c.id == id).ok_or_else(|| "Candidat introuvable.".to_string())
}

fn meme_personne(a: &Candidat, b: &Candidat) -> bool {
    if !a.fsid.is_empty() && a.fsid == b.fsid {
        return true;
    }
    normaliser(&a.nom_complet()) == normaliser(&b.nom_complet()) && annee(&a.naissance) == annee(&b.naissance)
}

/// Places encore vides pour cette personne dans l'arbre.
pub fn manquants(arbre: &Arbre, p: &Personne) -> Vec<Role> {
    let (pere, mere) = arbre.pere_mere(p);
    let mut r = Vec::new();
    if pere.is_none() {
        r.push(Role::Pere);
    }
    if mere.is_none() {
        r.push(Role::Mere);
    }
    r
}

pub fn libelle_manque(m: &[Role]) -> &'static str {
    match m {
        [] => "",
        [Role::Pere] => "père",
        [Role::Mere] => "mère",
        _ => "père et mère",
    }
}

fn cle_id(id: &str) -> (u64, String) {
    let chiffres: String = id.chars().filter(char::is_ascii_digit).collect();
    (chiffres.parse().unwrap_or(u64::MAX), id.to_string())
}

/// Année d'une date GEDCOM : « 12 MAR 1850 », « ABT 1850 », « BET 1850 AND 1860 » → 1850.
pub fn annee(date: &str) -> Option<i32> {
    date.split(|c: char| !c.is_ascii_digit())
        .filter(|t| t.len() == 3 || t.len() == 4)
        .filter_map(|t| t.parse::<i32>().ok())
        .find(|a| (500..=2200).contains(a))
}

/// Majuscules sans accents ni ponctuation : « Lefèvre-Dupré » → « LEFEVREDUPRE ».
pub fn normaliser(t: &str) -> String {
    let mut r = String::with_capacity(t.len());
    for c in t.chars().flat_map(char::to_uppercase) {
        let s = match c {
            'À' | 'Â' | 'Ä' | 'Á' | 'Ã' => "A",
            'É' | 'È' | 'Ê' | 'Ë' => "E",
            'Î' | 'Ï' | 'Í' => "I",
            'Ô' | 'Ö' | 'Ó' | 'Õ' => "O",
            'Ù' | 'Û' | 'Ü' | 'Ú' => "U",
            'Ç' => "C",
            'Ÿ' => "Y",
            'Ñ' => "N",
            'Œ' => "OE",
            'Æ' => "AE",
            c if c.is_ascii_alphabetic() => {
                r.push(c);
                continue;
            }
            _ => continue,
        };
        r.push_str(s);
    }
    r
}

/// Commune d'un lieu GEDCOM (« Astaffort, Lot-et-Garonne, France » → « ASTAFFORT »).
fn commune(lieu: &str) -> String {
    normaliser(lieu.split(',').next().unwrap_or(""))
}

fn proches(a: &str, b: &str) -> bool {
    a.contains(b) || b.contains(a) || (a.len() >= 4 && b.len() >= 4 && a[..4] == b[..4])
}

/// Contrôle de cohérence d'un candidat par rapport à l'enfant : score sur 100,
/// éléments concordants, alertes. Une seule alerte fait passer le lien « à vérifier ».
pub fn evaluer(enfant: &Personne, c: &Candidat) -> (u8, Vec<String>, Vec<String>) {
    let mut score = 0u32;
    let mut motifs = Vec::new();
    let mut alertes = Vec::new();

    let nom_enfant = enfant.nom_famille();
    match c.role {
        Role::Pere => {
            let (a, b) = (normaliser(&c.nom), normaliser(&nom_enfant));
            if !a.is_empty() && !b.is_empty() {
                if a == b {
                    score += 35;
                    motifs.push("Même nom de famille que l'enfant.".to_string());
                } else if proches(&a, &b) {
                    score += 15;
                    motifs.push("Nom de famille proche de celui de l'enfant (variante orthographique possible).".to_string());
                } else {
                    alertes.push(format!("Nom de famille différent de celui de l'enfant ({nom_enfant})."));
                }
            }
        }
        Role::Mere => {
            if !c.nom.trim().is_empty() {
                score += 15;
                motifs.push("Nom de naissance de la mère indiqué.".to_string());
            }
        }
    }

    let ne = annee(&enfant.naissance);
    if let (Some(an), Some(ae)) = (annee(&c.naissance), ne) {
        let age = ae - an;
        let (min, max) = match c.role {
            Role::Pere => (15, 75),
            Role::Mere => (13, 52),
        };
        if (min..=max).contains(&age) {
            score += 30;
            motifs.push(format!("{} de {age} ans à la naissance de l'enfant.", if c.role == Role::Pere { "Père âgé" } else { "Mère âgée" }));
        } else {
            alertes.push(format!("Âge invraisemblable à la naissance de l'enfant : {age} ans."));
        }
    }
    if let (Some(ad), Some(ae)) = (annee(&c.deces), ne) {
        // Un père peut mourir avant la naissance de son enfant, pas plus d'un an avant.
        let limite = if c.role == Role::Pere { ae - 1 } else { ae };
        if ad < limite {
            alertes.push(format!("Décès en {ad}, avant la naissance de l'enfant ({ae})."));
        } else {
            score += 5;
        }
    }

    let ce = commune(&enfant.lieu_naissance);
    if !ce.is_empty() && [&c.lieu_naissance, &c.lieu_deces].iter().any(|l| commune(l) == ce) {
        score += 10;
        motifs.push("Même commune que la naissance de l'enfant.".to_string());
    }

    match c.sources.iter().find(|s| !s.est_vide()) {
        None => alertes.push("Aucune source citée.".to_string()),
        Some(_) if c.provenance == "familysearch" && c.sources.iter().all(|s| s.organisme.starts_with("FamilySearch")) => {
            score += 10;
            motifs.push("Arbre familial FamilySearch : arbre collaboratif, à confirmer par un acte.".to_string());
        }
        Some(s) => {
            score += 20;
            motifs.push(format!("Source : {}.", if s.description().is_empty() { s.url.trim().to_string() } else { s.description() }));
        }
    }
    (score.min(100) as u8, motifs, alertes)
}

/// Écrit le lien validé : le parent est créé (ou repris s'il figure déjà avec le même
/// identifiant FamilySearch), rattaché à la famille d'origine de l'enfant où sa place est
/// libre (sinon à une famille nouvelle), et ses sources sont citées sur cette famille.
/// Les autres candidats encore ouverts pour la même place sont écartés. Rend
/// l'identifiant du parent.
pub fn accepter(doc: &mut Document, arbre: &Arbre, r: &mut Recherche, personne: &str, candidat: &str) -> Result<String, String> {
    let enfant = arbre.personne(personne).ok_or("Personne introuvable dans le fichier GEDCOM.")?;
    let t = r.travail_mut(personne)?;
    let c = t.candidats.iter().find(|x| x.id == candidat).cloned().ok_or("Candidat introuvable.")?;
    if c.etat == Etat::Accepte {
        return Err("Ce candidat est déjà accepté.".into());
    }
    let (pere, mere) = arbre.pere_mere(enfant);
    if let Some(x) = if c.role == Role::Pere { pere } else { mere } {
        return Err(format!("{} a déjà un(e) {} dans le fichier : {}.", enfant.nom_affiche(), c.role.libelle().to_lowercase(), x.nom_affiche()));
    }

    let existant = if c.fsid.is_empty() { None } else { arbre.par_fsid(&c.fsid) };
    let pid = match existant {
        Some(p) => p.id.clone(),
        None => {
            let id = doc.prochain_id_personne();
            let mut p = Personne::nouvelle(&id);
            p.prenoms = c.prenoms.trim().to_string();
            p.nom = c.nom.trim().to_string();
            p.nom_gedcom = if p.nom.is_empty() { p.prenoms.clone() } else { format!("{} /{}/", p.prenoms, p.nom).trim().to_string() };
            p.sexe = c.role.sexe().to_string();
            p.naissance = c.naissance.trim().to_string();
            p.lieu_naissance = c.lieu_naissance.trim().to_string();
            p.deces = c.deces.trim().to_string();
            p.lieu_deces = c.lieu_deces.trim().to_string();
            p.fsid = c.fsid.trim().to_string();
            doc.ajouter_personne(&p, None, None);
            id
        }
    };

    let libre = enfant.familles_enfant.iter().find(|f| {
        arbre.famille(f).is_some_and(|f| if c.role == Role::Pere { f.mari.is_none() } else { f.epouse.is_none() })
    });
    let fam = match libre {
        Some(f) => {
            doc.ajouter_reference(f, "FAM", c.role.balise(), &pid);
            doc.ajouter_reference(&pid, "INDI", "FAMS", f);
            f.clone()
        }
        None if c.role == Role::Pere => doc.creer_famille(Some(&pid), None, personne),
        None => doc.creer_famille(None, Some(&pid), personne),
    };

    let mut lignes = Vec::new();
    for s in c.sources.iter().filter(|s| !s.est_vide()) {
        let d = s.description();
        lignes.push(format!("1 SOUR {}", if d.is_empty() { s.url.trim().to_string() } else { d }));
        if !s.extrait.trim().is_empty() {
            lignes.push(format!("2 TEXT {}", une_ligne(&s.extrait)));
        }
        if !s.url.trim().is_empty() {
            lignes.push(format!("2 NOTE {}", s.url.trim()));
        }
        if !s.consulte_le.trim().is_empty() {
            lignes.push(format!("2 NOTE Consulté le {}", s.consulte_le.trim()));
        }
    }
    lignes.push(format!(
        "1 NOTE {} de {} validé dans GenFam (recherche d'ascendance) le {}.",
        c.role.libelle(), enfant.nom_affiche(), aujourdhui()
    ));
    doc.ajouter_lignes(&fam, "FAM", &lignes);

    for x in &mut t.candidats {
        if x.id == candidat {
            x.etat = Etat::Accepte;
            x.personne = pid.clone();
        } else if x.role == c.role && x.etat.ouvert() {
            x.etat = Etat::Rejete;
            x.alertes.push(AUTRE_ACCEPTE.to_string());
        }
    }
    Ok(pid)
}

fn une_ligne(t: &str) -> String {
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Candidats tirés de l'arbre familial FamilySearch : N° 2 (père) et N° 3 (mère) d'une
/// ascendance d'une génération dont le N° 1 est la personne recherchée.
pub fn candidats_familysearch(liste: &[PersonneFs]) -> Vec<Candidat> {
    let date = aujourdhui();
    liste
        .iter()
        .filter_map(|p| {
            let role = match p.numero {
                2 => Role::Pere,
                3 => Role::Mere,
                _ => return None,
            };
            Some(Candidat {
                role,
                prenoms: p.prenoms.clone(),
                nom: p.nom.clone(),
                naissance: p.naissance.clone(),
                lieu_naissance: p.lieu_naissance.clone(),
                deces: p.deces.clone(),
                lieu_deces: p.lieu_deces.clone(),
                fsid: p.fsid.clone(),
                provenance: "familysearch".into(),
                sources: vec![Source {
                    organisme: "FamilySearch — arbre familial".into(),
                    reference: p.fsid.clone(),
                    url: format!("https://www.familysearch.org/tree/person/details/{}", encoder(&p.fsid)),
                    consulte_le: date.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            })
        })
        .collect()
}

/// Recherche guidée : pages de recherche ouvertes dans le navigateur, préremplies avec
/// la personne dont on cherche les parents. Ces sites interdisent la collecte
/// automatique ; l'acte trouvé se relève ensuite à la main.
pub fn liens(p: &Personne) -> Vec<(String, String)> {
    let (prenoms, nom) = (p.prenoms_affiches(), p.nom_famille());
    let mut fs = format!(
        "https://www.familysearch.org/search/record/results?q.givenName={}&q.surname={}",
        encoder(&prenoms), encoder(&nom)
    );
    if let Some(a) = annee(&p.naissance) {
        fs.push_str(&format!("&q.birthLikeDate.from={}&q.birthLikeDate.to={}", a - 1, a + 1));
    }
    let lieu = p.lieu_naissance.split(',').next().unwrap_or("").trim().to_string();
    if !lieu.is_empty() {
        fs.push_str(&format!("&q.birthLikePlace={}", encoder(&lieu)));
    }
    let mut r = vec![
        ("FamilySearch — actes indexés (naissance, mariage)".to_string(), fs),
        (
            "Geneanet — relevés et arbres publiés".to_string(),
            format!("https://www.geneanet.org/fonds/individus/?go=1&nom={}&prenom={}", encoder(&nom), encoder(&prenoms)),
        ),
    ];
    let q = if lieu.is_empty() { "état civil registres paroissiaux".to_string() } else { format!("état civil {lieu}") };
    r.push(("FranceArchives — archives départementales et communales".to_string(), format!("https://francearchives.gouv.fr/fr/search?q={}", encoder(&q))));
    r
}

/// Faux pour une personne née il y a moins de 100 ans : ses actes de naissance et de
/// mariage peuvent ne pas être publiés. C'est un critère de priorité, pas une interdiction.
pub fn prioritaire(p: &Personne) -> Option<bool> {
    let a = annee(&p.naissance)?;
    Some(annee_courante() - a >= ANS_PRIORITE)
}

fn annee_courante() -> i32 {
    aujourdhui()[..4].parse().unwrap_or(2026)
}

/// Date du jour, AAAA-MM-JJ (temps universel).
pub fn aujourdhui() -> String {
    let jours = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() / 86_400).unwrap_or(0) as i64;
    let (a, m, j) = civil(jours);
    format!("{a:04}-{m:02}-{j:02}")
}

/// Date civile d'un nombre de jours depuis le 1er janvier 1970 (algorithme de H. Hinnant).
fn civil(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = yoe + era * 400;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn chemin_pour(gedcom: &Path) -> PathBuf {
    let mut s = gedcom.as_os_str().to_owned();
    s.push(".recherche.json");
    s.into()
}

pub fn charger(gedcom: &Path) -> Option<Recherche> {
    serde_json::from_slice(&std::fs::read(chemin_pour(gedcom)).ok()?).ok()
}

pub fn enregistrer(gedcom: &Path, r: &Recherche) -> std::io::Result<()> {
    let json = serde_json::to_vec_pretty(r).map_err(std::io::Error::other)?;
    std::fs::write(chemin_pour(gedcom), json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gedcom::analyser;

    const EXEMPLE: &str = include_str!("../tests/donnees/exemple.ged");

    fn source() -> Source {
        Source {
            organisme: "Archives départementales de Lot-et-Garonne".into(),
            reference: "4 E 12/3".into(),
            commune: "Astaffort".into(),
            url: "https://example.org/acte/42".into(),
            consulte_le: "2026-10-28".into(),
            extrait: "fils de Louis Martin\net de Jeanne Blanc".into(),
        }
    }

    fn candidat(role: Role, prenoms: &str, nom: &str, naissance: &str) -> Candidat {
        Candidat { role, prenoms: prenoms.into(), nom: nom.into(), naissance: naissance.into(), sources: vec![source()], ..Default::default() }
    }

    #[test]
    fn travaux_depuis_exemple() {
        let a = analyser(EXEMPLE);
        let r = Recherche::nouvelle(&a, "@I1@", 20);
        let t: Vec<(&str, u32)> = r.travaux.iter().map(|t| (t.personne.as_str(), t.generation)).collect();
        assert_eq!(t, vec![("@I2@", 2), ("@I3@", 2), ("@I5@", 3)]);
        assert_eq!(manquants(&a, a.personne("@I2@").unwrap()), vec![Role::Mere]);
        // Limite de générations : les parents cherchés ne dépassent jamais la limite.
        assert!(Recherche::nouvelle(&a, "@I1@", 2).travaux.is_empty());
        assert_eq!(Recherche::nouvelle(&a, "@I1@", 3).travaux.len(), 2);
    }

    #[test]
    fn annees_et_noms() {
        assert_eq!(annee("12 MAR 1850"), Some(1850));
        assert_eq!(annee("ABT 1850"), Some(1850));
        assert_eq!(annee("BET 1850 AND 1860"), Some(1850));
        assert_eq!(annee("28 April 1702"), Some(1702));
        assert_eq!(annee(""), None);
        assert_eq!(normaliser("Lefèvre-Dupré"), "LEFEVREDUPRE");
        assert_eq!(commune("Astaffort, Lot-et-Garonne, France"), "ASTAFFORT");
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(civil(20_754), (2026, 10, 28));
    }

    #[test]
    fn coherence() {
        let a = analyser(EXEMPLE);
        let paul = a.personne("@I1@").unwrap();
        let (s, m, al) = evaluer(paul, &candidat(Role::Pere, "Jacques", "Martin", "1920"));
        assert!(al.is_empty(), "{al:?}");
        assert_eq!(s, 85);
        assert_eq!(m.len(), 3);
        let (_, _, al) = evaluer(paul, &candidat(Role::Mere, "Marie", "Durand", "1945"));
        assert!(al.iter().any(|x| x.contains("5 ans")));
        let mut sans = candidat(Role::Pere, "Jacques", "Dubois", "1920");
        sans.sources.clear();
        let (_, _, al) = evaluer(paul, &sans);
        assert_eq!(al.len(), 2);
        let mut morte = candidat(Role::Mere, "Marie", "Durand", "1925");
        morte.deces = "1949".into();
        assert!(evaluer(paul, &morte).2.iter().any(|x| x.starts_with("Décès en 1949")));
    }

    #[test]
    fn candidats_et_doublons() {
        let a = analyser(EXEMPLE);
        let mut r = Recherche::nouvelle(&a, "@I1@", 20);
        let (id, nouveau) = r.ajouter_candidat(&a, "@I2@", candidat(Role::Mere, "Jeanne", "Blanc", "1898")).unwrap();
        assert_eq!((id.as_str(), nouveau), ("c1", true));
        let mut autre = candidat(Role::Mere, "Jeanne", "BLANC", "12 JAN 1898");
        autre.sources[0].reference = "4 E 12/4".into();
        let (id, nouveau) = r.ajouter_candidat(&a, "@I2@", autre).unwrap();
        assert_eq!((id.as_str(), nouveau), ("c1", false));
        assert_eq!(r.travail("@I2@").unwrap().candidats[0].sources.len(), 2);
        // La place du père est déjà prise.
        assert!(r.ajouter_candidat(&a, "@I2@", candidat(Role::Pere, "Louis", "Martin", "1890")).is_err());
        // Personne hors recherche.
        assert!(r.ajouter_candidat(&a, "@I6@", candidat(Role::Pere, "X", "Petit", "1920")).is_err());
    }

    #[test]
    fn acceptation_dans_la_famille_existante() {
        let a = analyser(EXEMPLE);
        let mut doc = Document::depuis_texte(EXEMPLE);
        let mut r = Recherche::nouvelle(&a, "@I1@", 20);
        r.ajouter_candidat(&a, "@I2@", candidat(Role::Mere, "Jeanne", "Blanc", "1898")).unwrap();
        r.ajouter_candidat(&a, "@I2@", candidat(Role::Mere, "Rose", "Noir", "1899")).unwrap();
        let pid = accepter(&mut doc, &a, &mut r, "@I2@", "c1").unwrap();
        assert_eq!(pid, "@I8@");
        let texte = doc.texte();
        assert!(texte.contains("1 SOUR Archives départementales de Lot-et-Garonne, 4 E 12/3, Astaffort"));
        assert!(texte.contains("2 TEXT fils de Louis Martin et de Jeanne Blanc"));
        let b = analyser(&texte);
        let f2 = b.famille("@F2@").unwrap();
        assert_eq!(f2.epouse.as_deref(), Some("@I8@"));
        assert_eq!(b.parents(b.personne("@I2@").unwrap()).len(), 2);
        assert_eq!(b.personne("@I8@").unwrap().sexe, "F");
        let t = r.travail("@I2@").unwrap();
        assert_eq!(t.candidats[0].etat, Etat::Accepte);
        assert_eq!(t.candidats[1].etat, Etat::Rejete);
        // La nouvelle mère devient à son tour une personne à compléter.
        r.actualiser(&b);
        assert!(r.travail("@I8@").is_some_and(|t| t.generation == 3));
        // Refus d'un second lien pour la même place.
        assert!(accepter(&mut doc, &b, &mut r, "@I2@", "c2").is_err());
        // Annulation de l'écriture : le candidat accepté est rouvert.
        r.actualiser(&a);
        assert_eq!(r.travail("@I2@").unwrap().candidats[0].etat, Etat::Propose);
    }

    #[test]
    fn acceptation_famille_nouvelle_et_reprise_fsid() {
        let a = analyser(EXEMPLE);
        let mut doc = Document::depuis_texte(EXEMPLE);
        let mut r = Recherche::nouvelle(&a, "@I1@", 20);
        // Marie DURAND n'a aucune famille d'origine : une famille est créée.
        r.ajouter_candidat(&a, "@I3@", candidat(Role::Pere, "Henri", "Durand", "1920")).unwrap();
        let pid = accepter(&mut doc, &a, &mut r, "@I3@", "c1").unwrap();
        let b = analyser(&doc.texte());
        let marie = b.personne("@I3@").unwrap();
        assert_eq!(marie.familles_enfant, vec!["@F4@".to_string()]);
        assert_eq!(b.famille("@F4@").unwrap().mari.as_deref(), Some(pid.as_str()));
        // Un candidat dont l'identifiant FamilySearch figure déjà dans le fichier est repris.
        let mut r = Recherche::nouvelle(&b, "@I1@", 20);
        let mut c = candidat(Role::Pere, "Paul", "Martin", "1950");
        c.fsid = "LZ1A-B2C".into();
        r.ajouter_candidat(&b, "@I5@", c).unwrap();
        let mut doc2 = Document::depuis_texte(&doc.texte());
        assert_eq!(accepter(&mut doc2, &b, &mut r, "@I5@", "c1").unwrap(), "@I1@");
    }

    #[test]
    fn familysearch_et_liens() {
        let liste = vec![
            PersonneFs { numero: 1, fsid: "AAAA-111".into(), ..Default::default() },
            PersonneFs { numero: 2, fsid: "BBBB-222".into(), prenoms: "Louis".into(), nom: "Martin".into(), ..Default::default() },
            PersonneFs { numero: 3, fsid: "CCCC-333".into(), prenoms: "Jeanne".into(), nom: "Blanc".into(), ..Default::default() },
        ];
        let c = candidats_familysearch(&liste);
        assert_eq!(c.len(), 2);
        assert_eq!((c[0].role, c[1].role), (Role::Pere, Role::Mere));
        assert!(c[0].sources[0].url.ends_with("/BBBB-222"));
        let a = analyser(EXEMPLE);
        let l = liens(a.personne("@I1@").unwrap());
        assert_eq!(l.len(), 3);
        assert!(l[0].1.contains("q.givenName=Paul&q.surname=MARTIN&q.birthLikeDate.from=1949&q.birthLikeDate.to=1951&q.birthLikePlace=Agen"));
        assert!(l[1].1.contains("nom=MARTIN&prenom=Paul"));
        assert_eq!(prioritaire(a.personne("@I1@").unwrap()), Some(false));
    }

    #[test]
    fn aller_retour_json() {
        let a = analyser(EXEMPLE);
        let mut r = Recherche::nouvelle(&a, "@I1@", 12);
        r.ajouter_candidat(&a, "@I3@", candidat(Role::Pere, "Henri", "Durand", "1920")).unwrap();
        r.noter("@I3@", "Saisie", "candidat ajouté");
        let dir = std::env::temp_dir().join(format!("genfam-recherche-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ged = dir.join("essai.ged");
        enregistrer(&ged, &r).unwrap();
        assert_eq!(charger(&ged), Some(r));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
