// SPDX-License-Identifier: GPL-3.0-or-later
//! Ordonnances : données locales, hors GEDCOM.
//!
//! La version Python les gardait en mémoire seulement — elles étaient perdues à la
//! fermeture. Elles sont désormais enregistrées à côté du GEDCOM, dans
//! `<fichier>.ordonnances.json`.

use crate::modele::{plier, Arbre};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

pub const TYPES: [&str; 6] = [
    "Baptême",
    "Confirmation",
    "Initiatoire",
    "Dotation",
    "Scellement au conjoint",
    "Scellement aux parents",
];

pub const STATUTS: [&str; 4] = ["À vérifier", "À faire", "Réservée", "Accomplie"];
pub const ACCOMPLIE: &str = "Accomplie";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Ordonnance {
    #[serde(rename = "type")]
    pub genre: String,
    pub statut: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub temple: String,
    #[serde(default)]
    pub notes: String,
}

/// Ordonnances de toutes les personnes d'un fichier, par identifiant GEDCOM.
pub type Registre = BTreeMap<String, Vec<Ordonnance>>;

pub fn chemin_pour(gedcom: &Path) -> PathBuf {
    let mut s = gedcom.as_os_str().to_owned();
    s.push(".ordonnances.json");
    s.into()
}

pub fn charger(gedcom: &Path) -> Registre {
    std::fs::read(chemin_pour(gedcom))
        .ok()
        .and_then(|o| serde_json::from_slice(&o).ok())
        .unwrap_or_default()
}

pub fn enregistrer(gedcom: &Path, registre: &Registre) -> std::io::Result<()> {
    let chemin = chemin_pour(gedcom);
    if registre.values().all(|v| v.is_empty()) {
        if chemin.exists() {
            std::fs::remove_file(chemin)?;
        }
        return Ok(());
    }
    let json = serde_json::to_vec_pretty(registre).map_err(std::io::Error::other)?;
    std::fs::write(chemin, json)
}

/// Enregistre le registre après avoir copié l'état précédent dans
/// `<fichier>.ordonnances.json.bak` : un import touche des centaines de fiches d'un coup.
pub fn enregistrer_avec_sauvegarde(gedcom: &Path, registre: &Registre) -> std::io::Result<()> {
    let chemin = chemin_pour(gedcom);
    if chemin.is_file() {
        let mut bak = chemin.as_os_str().to_owned();
        bak.push(".bak");
        std::fs::copy(&chemin, PathBuf::from(bak))?;
    }
    enregistrer(gedcom, registre)
}

// ---- Relevés FamilySearch : tableau CSV ou texte copié ---------------------------

/// Lettres et chiffres d'un texte, en minuscules sans accents.
fn compact(t: &str) -> String {
    plier(t).chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Type d'ordonnance nommé dans un texte, en français ou en anglais.
pub fn genre_depuis(t: &str) -> Option<&'static str> {
    let n = compact(t);
    if n.contains("bapt") {
        Some("Baptême")
    } else if n.contains("confirm") {
        Some("Confirmation")
    } else if n.contains("initiat") {
        Some("Initiatoire")
    } else if n.contains("dotation") || n.contains("endow") {
        Some("Dotation")
    } else if n.contains("scell") || n.contains("seal") {
        if n.contains("parent") {
            Some("Scellement aux parents")
        } else if ["conjoint", "spouse", "epou", "mari", "couple"].iter().any(|m| n.contains(m)) {
            Some("Scellement au conjoint")
        } else {
            None
        }
    } else {
        None
    }
}

/// Statut nommé dans un texte (« Completed », « Accomplie », « Reserved »…).
pub fn statut_depuis(t: &str) -> Option<&'static str> {
    let n = compact(t);
    let a = |mots: &[&str]| mots.iter().any(|m| n.contains(m));
    if a(&["nondisponible", "indisponible", "notavailable", "needsmore", "needmore", "averifier", "verify"]) {
        Some("À vérifier")
    } else if a(&["accompli", "complete", "termine", "effectue", "done"]) {
        Some(ACCOMPLIE)
    } else if a(&["reserv", "inprogress", "encours", "partage", "shared"]) {
        Some("Réservée")
    } else if a(&["afaire", "ready", "pret", "disponible", "available"]) {
        Some("À faire")
    } else {
        None
    }
}

fn rang(statut: &str) -> usize {
    STATUTS.iter().position(|s| *s == statut).unwrap_or(0)
}

const MOIS: [(&str, &str); 41] = [
    ("janvier", "JAN"), ("january", "JAN"), ("janv", "JAN"), ("jan", "JAN"),
    ("fevrier", "FEB"), ("february", "FEB"), ("fevr", "FEB"), ("fev", "FEB"), ("feb", "FEB"),
    ("mars", "MAR"), ("march", "MAR"), ("mar", "MAR"),
    ("avril", "APR"), ("april", "APR"), ("avr", "APR"), ("apr", "APR"),
    ("mai", "MAY"), ("may", "MAY"),
    ("juin", "JUN"), ("june", "JUN"), ("jun", "JUN"),
    ("juillet", "JUL"), ("july", "JUL"), ("juil", "JUL"), ("jul", "JUL"),
    ("aout", "AUG"), ("august", "AUG"), ("aug", "AUG"),
    ("septembre", "SEP"), ("september", "SEP"), ("sept", "SEP"), ("sep", "SEP"),
    ("octobre", "OCT"), ("october", "OCT"), ("oct", "OCT"),
    ("novembre", "NOV"), ("november", "NOV"), ("nov", "NOV"),
    ("decembre", "DEC"), ("december", "DEC"), ("dec", "DEC"),
];

const NUMEROS_MOIS: [&str; 12] = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];

/// Date au format GEDCOM (« 12 MAR 1990 ») lue dans un texte : « 12 mars 1990 »,
/// « March 12, 1990 », « 12/03/1990 », « 1990-03-12 ». `None` sans année lisible.
pub fn date_gedcom(t: &str) -> Option<String> {
    let p = plier(t);
    let mots: Vec<&str> = p.split(|c: char| !c.is_alphanumeric()).filter(|m| !m.is_empty()).collect();
    let chiffres = |m: &str| m.chars().all(|c| c.is_ascii_digit());
    if mots.len() == 3 && mots.iter().all(|m| chiffres(m)) {
        let (j, m, a) = if mots[0].len() == 4 { (mots[2], mots[1], mots[0]) } else { (mots[0], mots[1], mots[2]) };
        let (j, m): (u32, usize) = (j.parse().ok()?, m.parse().ok()?);
        if a.len() == 4 && (1..=31).contains(&j) && (1..=12).contains(&m) {
            return Some(format!("{j} {} {a}", NUMEROS_MOIS[m - 1]));
        }
        return None;
    }
    let annee = mots.iter().find(|m| m.len() == 4 && chiffres(m))?;
    let mois = mots.iter().find_map(|m| MOIS.iter().find(|(nom, _)| nom == m).map(|(_, abr)| *abr));
    let premier = mots.iter().any(|m| *m == "1er" || *m == "premier");
    let jour = mots
        .iter()
        .filter(|m| (1..=2).contains(&m.len()) && chiffres(m))
        .filter_map(|m| m.parse::<u32>().ok())
        .find(|j| (1..=31).contains(j))
        .or(premier.then_some(1));
    Some(match (jour, mois) {
        (Some(j), Some(m)) => format!("{j} {m} {annee}"),
        (None, Some(m)) => format!("{m} {annee}"),
        _ => annee.to_string(),
    })
}

fn meme_date(a: &str, b: &str) -> bool {
    let n = |d: &str| date_gedcom(d).unwrap_or_else(|| d.trim().to_uppercase());
    n(a) == n(b)
}

/// Effet du versement d'une ordonnance dans la liste d'une personne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effet {
    Ajoutee,
    MiseAJour,
    Inchangee,
    /// Même ordonnance déjà notée avec une autre date : la note existante est gardée.
    Conflit,
}

/// Verse une ordonnance relevée : complète celle du même type si les dates concordent
/// (un statut ne recule jamais : « Accomplie » l'emporte), sinon l'ajoute. Un
/// scellement au conjoint peut se répéter (un par conjoint) ; les autres non.
pub fn fusionner(liste: &mut Vec<Ordonnance>, o: Ordonnance) -> Effet {
    let mut meme_type = false;
    for x in liste.iter_mut().filter(|x| x.genre == o.genre) {
        meme_type = true;
        if x.date.is_empty() || o.date.is_empty() || meme_date(&x.date, &o.date) {
            let mut change = false;
            if rang(&o.statut) > rang(&x.statut) {
                x.statut = o.statut.clone();
                change = true;
            }
            for (champ, valeur) in [(&mut x.date, &o.date), (&mut x.temple, &o.temple)] {
                if champ.is_empty() && !valeur.is_empty() {
                    *champ = valeur.clone();
                    change = true;
                }
            }
            return if change { Effet::MiseAJour } else { Effet::Inchangee };
        }
    }
    if meme_type && o.genre != "Scellement au conjoint" {
        return Effet::Conflit;
    }
    liste.push(o);
    Effet::Ajoutee
}

/// Une ligne d'un relevé en tableau : la personne (identifiant FamilySearch, à défaut
/// identifiant GEDCOM), son nom tel qu'écrit, et l'ordonnance.
#[derive(Debug, Clone, PartialEq)]
pub struct LigneReleve {
    pub cle: String,
    pub nom: String,
    pub ordonnance: Ordonnance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Sous {
    Valeur,
    Date,
    Temple,
    Statut,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Colonne {
    Id,
    IdGedcom,
    Nom,
    Genre,
    Statut,
    Date,
    Temple,
    /// Tableau « large » : une colonne (ou un groupe) par type d'ordonnance.
    Type(&'static str, Sous),
}

fn classer(entete: &str) -> Option<Colonne> {
    let n = compact(entete);
    if let Some(g) = genre_depuis(entete) {
        let sous = if n.contains("date") {
            Sous::Date
        } else if n.contains("temple") {
            Sous::Temple
        } else if n.contains("statut") || n.contains("status") {
            Sous::Statut
        } else {
            Sous::Valeur
        };
        return Some(Colonne::Type(g, sous));
    }
    let parmi = |l: &[&str]| l.contains(&n.as_str());
    Some(if parmi(&["idgedcom", "gedcomid", "refgedcom", "xref", "gedcom", "rin"]) {
        Colonne::IdGedcom
    } else if parmi(&["id", "pid", "fsid", "idfs", "personid", "identifiant", "idpersonne", "treeid", "familysearch"])
        || (n.contains("familysearch") && n.contains("id"))
        || n.starts_with("identifiant")
    {
        Colonne::Id
    } else if parmi(&["nom", "name", "nomcomplet", "fullname", "personne", "person", "nometprenoms", "nomprenoms", "prenomsnom", "nometprenom"]) {
        Colonne::Nom
    } else if parmi(&["ordonnance", "ordinance", "type", "typedordonnance", "typeordonnance", "ordinancetype"]) {
        Colonne::Genre
    } else if parmi(&["statut", "status", "etat", "state"]) {
        Colonne::Statut
    } else if n.contains("date") || parmi(&["completed", "accomplie", "le"]) {
        Colonne::Date
    } else if n.contains("temple") {
        Colonne::Temple
    } else {
        return None;
    })
}

/// Champs d'une ligne CSV : guillemets doubles, `""` pour un guillemet.
fn champs_csv(ligne: &str, sep: char) -> Vec<String> {
    let mut r = Vec::new();
    let mut courant = String::new();
    let mut guillemets = false;
    let mut car = ligne.chars().peekable();
    while let Some(c) = car.next() {
        match c {
            '"' if guillemets && car.peek() == Some(&'"') => {
                courant.push('"');
                car.next();
            }
            '"' => guillemets = !guillemets,
            c if c == sep && !guillemets => r.push(std::mem::take(&mut courant).trim().to_string()),
            c => courant.push(c),
        }
    }
    r.push(courant.trim().to_string());
    r
}

fn separateur(ligne: &str) -> char {
    ['\t', ';', ','].into_iter().max_by_key(|s| ligne.matches(*s).count()).unwrap_or(',')
}

/// Lit un relevé en tableau (CSV exporté de FamilySearch, ou tableau copié avec ses
/// tabulations). Deux formes sont reconnues : une ligne par ordonnance (colonne
/// « Ordonnance » / « Ordinance »), ou une ligne par personne avec une colonne par type
/// (« Baptême », « Baptism Date », « Endowment Temple »…). Une ordonnance sans statut
/// lisible mais datée est tenue pour accomplie : c'est l'objet de l'export.
pub fn lire_tableau(texte: &str) -> Result<Vec<LigneReleve>, String> {
    let lignes: Vec<&str> = texte.lines().map(|l| l.trim_start_matches('\u{feff}')).filter(|l| !l.trim().is_empty()).collect();
    let mut entete = None;
    for (i, l) in lignes.iter().enumerate().take(10) {
        let sep = separateur(l);
        let cols: Vec<Option<Colonne>> = champs_csv(l, sep).iter().map(|c| classer(c)).collect();
        let a_cle = cols.iter().any(|c| matches!(c, Some(Colonne::Id | Colonne::IdGedcom)));
        let a_type = cols.iter().any(|c| matches!(c, Some(Colonne::Genre | Colonne::Type(..))));
        if a_cle && a_type {
            entete = Some((i, sep, cols));
            break;
        }
    }
    let Some((debut, sep, cols)) = entete else {
        return Err("En-tête non reconnu : le tableau doit avoir une colonne d'identifiant FamilySearch \
                    (ou GEDCOM) et une colonne d'ordonnance."
            .into());
    };
    let col = |k: Colonne| cols.iter().position(|c| *c == Some(k));
    let (c_id, c_ged, c_nom) = (col(Colonne::Id), col(Colonne::IdGedcom), col(Colonne::Nom));
    let (c_genre, c_statut, c_date, c_temple) = (col(Colonne::Genre), col(Colonne::Statut), col(Colonne::Date), col(Colonne::Temple));
    let mut sortie = Vec::new();
    for l in &lignes[debut + 1..] {
        let v = champs_csv(l, sep);
        let cell = |c: Option<usize>| c.and_then(|i| v.get(i)).map(String::as_str).unwrap_or("").trim().to_string();
        let cle = [cell(c_id), cell(c_ged)].into_iter().find(|c| !c.is_empty()).unwrap_or_default();
        if cle.is_empty() {
            continue;
        }
        let nom = cell(c_nom);
        let fabriquer = |genre: &'static str, valeur: &str, statut: &str, date: &str, temple: &str| -> Option<Ordonnance> {
            if [valeur, statut, date, temple].iter().all(|x| x.trim().is_empty()) {
                return None;
            }
            let date = date_gedcom(date).or_else(|| date_gedcom(valeur)).unwrap_or_else(|| date.trim().to_string());
            let statut = statut_depuis(statut).or_else(|| statut_depuis(valeur)).map(str::to_string).unwrap_or_else(|| {
                if !date.is_empty() || !valeur.trim().is_empty() { ACCOMPLIE.to_string() } else { STATUTS[0].to_string() }
            });
            Some(Ordonnance { genre: genre.to_string(), statut, date, temple: temple.trim().to_string(), notes: String::new() })
        };
        if let Some(cg) = c_genre {
            if let Some(genre) = genre_depuis(&cell(Some(cg))) {
                let s = cell(c_statut);
                let s = if s.is_empty() && c_statut.is_none() { ACCOMPLIE.to_string() } else { s };
                if let Some(o) = fabriquer(genre, "", &s, &cell(c_date), &cell(c_temple)) {
                    sortie.push(LigneReleve { cle: cle.clone(), nom: nom.clone(), ordonnance: o });
                }
            }
        }
        let mut types: Vec<&'static str> = Vec::new();
        for c in cols.iter().flatten() {
            if let Colonne::Type(g, _) = c {
                if !types.contains(g) {
                    types.push(g);
                }
            }
        }
        for g in types {
            let sous = |s: Sous| cell(col(Colonne::Type(g, s)));
            if let Some(o) = fabriquer(g, &sous(Sous::Valeur), &sous(Sous::Statut), &sous(Sous::Date), &sous(Sous::Temple)) {
                sortie.push(LigneReleve { cle: cle.clone(), nom: nom.clone(), ordonnance: o });
            }
        }
    }
    Ok(sortie)
}

/// Lit un relevé copié depuis la page d'une personne : un intitulé d'ordonnance, puis
/// son statut, sa date et son temple sur les lignes suivantes, dans n'importe quel ordre.
pub fn lire_releve(texte: &str) -> Vec<Ordonnance> {
    let mut r: Vec<Ordonnance> = Vec::new();
    for brute in texte.lines() {
        let l = brute.trim();
        if l.is_empty() {
            continue;
        }
        if let Some(g) = genre_depuis(l) {
            if l.chars().count() <= 60 && date_gedcom(l).is_none() {
                r.push(Ordonnance { genre: g.to_string(), statut: statut_depuis(l).unwrap_or("").to_string(), ..Default::default() });
                continue;
            }
        }
        let Some(o) = r.last_mut() else { continue };
        if compact(l).contains("temple") {
            if o.temple.is_empty() {
                o.temple = l.to_string();
            }
            continue;
        }
        if let Some(s) = statut_depuis(l) {
            if o.statut.is_empty() {
                o.statut = s.to_string();
            }
        }
        if let Some(d) = date_gedcom(l) {
            if o.date.is_empty() {
                o.date = d;
            }
        }
    }
    for o in &mut r {
        if o.statut.is_empty() {
            o.statut = if o.date.is_empty() { STATUTS[0].to_string() } else { ACCOMPLIE.to_string() };
        }
    }
    r
}

/// Bilan d'un versement de relevé.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Bilan {
    pub lues: usize,
    pub personnes: usize,
    pub ajoutees: usize,
    pub mises_a_jour: usize,
    pub inchangees: usize,
    pub conflits: usize,
    /// Lignes sans personne correspondante : « clé (nom) ».
    pub inconnues: Vec<String>,
}

impl Bilan {
    fn compter(&mut self, e: Effet) {
        match e {
            Effet::Ajoutee => self.ajoutees += 1,
            Effet::MiseAJour => self.mises_a_jour += 1,
            Effet::Inchangee => self.inchangees += 1,
            Effet::Conflit => self.conflits += 1,
        }
    }

    pub fn modifie(&self) -> bool {
        self.ajoutees + self.mises_a_jour > 0
    }

    pub fn texte(&self) -> String {
        let mut t = format!(
            "{} ordonnance(s) lue(s), {} personne(s) reconnue(s).\n\n\
             Ajoutées : {}\nMises à jour : {}\nDéjà à jour : {}",
            self.lues, self.personnes, self.ajoutees, self.mises_a_jour, self.inchangees
        );
        if self.conflits > 0 {
            t += &format!("\nDate différente de celle déjà notée (laissée telle quelle) : {}", self.conflits);
        }
        if !self.inconnues.is_empty() {
            let n = self.inconnues.len();
            let exemples: Vec<&str> = self.inconnues.iter().take(10).map(String::as_str).collect();
            t += &format!(
                "\n\n{n} ligne(s) sans personne correspondante dans le GEDCOM (identifiant FamilySearch absent) :\n{}{}",
                exemples.join("\n"),
                if n > 10 { "\n…" } else { "" }
            );
        }
        t
    }
}

/// Verse un relevé en tableau dans le registre. Les personnes sont reconnues par leur
/// identifiant FamilySearch (`_FSFTID`), à défaut par leur identifiant GEDCOM ; jamais
/// par leur nom, que des homonymes rendraient incertain.
pub fn appliquer(registre: &mut Registre, arbre: &Arbre, lignes: &[LigneReleve]) -> Bilan {
    let index: HashMap<String, &str> =
        arbre.personnes.iter().filter(|p| !p.fsid.is_empty()).map(|p| (p.fsid.trim().to_uppercase(), p.id.as_str())).collect();
    let mut bilan = Bilan { lues: lignes.len(), ..Default::default() };
    let mut vues = HashSet::new();
    for l in lignes {
        let cle = l.cle.trim();
        let id = match index.get(&cle.to_uppercase()) {
            Some(id) => id.to_string(),
            None => {
                let x = if cle.starts_with('@') { cle.to_string() } else { format!("@{cle}@") };
                if arbre.personne(&x).is_none() {
                    let nom = if l.nom.is_empty() { String::new() } else { format!(" ({})", l.nom) };
                    let e = format!("{cle}{nom}");
                    if !bilan.inconnues.contains(&e) {
                        bilan.inconnues.push(e);
                    }
                    continue;
                }
                x
            }
        };
        vues.insert(id.clone());
        bilan.compter(fusionner(registre.entry(id).or_default(), l.ordonnance.clone()));
    }
    bilan.personnes = vues.len();
    bilan
}

/// Verse un relevé copié pour une seule personne.
pub fn appliquer_releve(registre: &mut Registre, personne: &str, liste: Vec<Ordonnance>) -> Bilan {
    let mut bilan = Bilan { lues: liste.len(), personnes: 1, ..Default::default() };
    let l = registre.entry(personne.to_string()).or_default();
    for o in liste {
        bilan.compter(fusionner(l, o));
    }
    bilan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gedcom::analyser;

    #[test]
    fn aller_retour() {
        let dossier = std::env::temp_dir().join(format!("genfam-ord-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        let ged = dossier.join("essai.ged");
        let mut r = Registre::new();
        r.entry("@I1@".into()).or_default().push(Ordonnance {
            genre: TYPES[0].into(),
            statut: STATUTS[3].into(),
            date: "1 JAN 2000".into(),
            temple: "Paris".into(),
            notes: String::new(),
        });
        enregistrer(&ged, &r).unwrap();
        assert_eq!(charger(&ged), r);
        r.clear();
        enregistrer(&ged, &r).unwrap();
        assert!(!chemin_pour(&ged).exists());
        std::fs::remove_dir_all(dossier).unwrap();
    }

    #[test]
    fn dates_et_libelles() {
        assert_eq!(date_gedcom("12 mars 1990").as_deref(), Some("12 MAR 1990"));
        assert_eq!(date_gedcom("March 12, 1990").as_deref(), Some("12 MAR 1990"));
        assert_eq!(date_gedcom("12/03/1990").as_deref(), Some("12 MAR 1990"));
        assert_eq!(date_gedcom("1990-03-12").as_deref(), Some("12 MAR 1990"));
        assert_eq!(date_gedcom("1er août 2001").as_deref(), Some("1 AUG 2001"));
        assert_eq!(date_gedcom("Décembre 1890").as_deref(), Some("DEC 1890"));
        assert_eq!(date_gedcom("Completed"), None);
        assert_eq!(genre_depuis("Sealing to Parents"), Some("Scellement aux parents"));
        assert_eq!(genre_depuis("Scellement au conjoint"), Some("Scellement au conjoint"));
        assert_eq!(genre_depuis("Endowment"), Some("Dotation"));
        assert_eq!(genre_depuis("Initiatory"), Some("Initiatoire"));
        assert_eq!(statut_depuis("Completed"), Some(ACCOMPLIE));
        assert_eq!(statut_depuis("Non disponible"), Some("À vérifier"));
        assert_eq!(statut_depuis("Prête"), Some("À faire"));
    }

    const ARBRE: &str = "0 @I1@ INDI\n1 NAME Jean /MARTIN/\n1 _FSFTID KWAA-AAA\n0 @I2@ INDI\n1 NAME Marie /DURAND/\n1 _FSFTID KWBB-BBB\n0 TRLR\n";

    #[test]
    fn import_csv_une_ligne_par_ordonnance() {
        let csv = "Person ID,Name,Ordinance,Status,Date,Temple\n\
                   KWAA-AAA,Jean MARTIN,Baptism,Completed,12 March 1990,Paris France Temple\n\
                   kwaa-aaa,Jean MARTIN,Confirmation,Completed,13 March 1990,\"Paris, France\"\n\
                   KWBB-BBB,Marie DURAND,Endowment,Completed,1 June 2001,Bern Switzerland Temple\n\
                   KWZZ-ZZZ,Inconnu ABSENT,Baptism,Completed,1 June 2001,\n";
        let lignes = lire_tableau(csv).unwrap();
        assert_eq!(lignes.len(), 4);
        assert_eq!(lignes[1].ordonnance.temple, "Paris, France");
        let a = analyser(ARBRE);
        let mut r = Registre::new();
        // Une ordonnance déjà notée « À faire » sans date est complétée, pas doublée.
        r.entry("@I1@".into()).or_default().push(Ordonnance { genre: "Baptême".into(), statut: "À faire".into(), ..Default::default() });
        let b = appliquer(&mut r, &a, &lignes);
        assert_eq!((b.lues, b.personnes, b.ajoutees, b.mises_a_jour, b.inchangees), (4, 2, 2, 1, 0));
        assert_eq!(b.inconnues, vec!["KWZZ-ZZZ (Inconnu ABSENT)".to_string()]);
        let jean = &r["@I1@"];
        assert_eq!(jean.len(), 2);
        assert_eq!((jean[0].statut.as_str(), jean[0].date.as_str(), jean[0].temple.as_str()), (ACCOMPLIE, "12 MAR 1990", "Paris France Temple"));
        assert_eq!(r["@I2@"][0].genre, "Dotation");
        // Le même import rejoué ne change plus rien.
        let b = appliquer(&mut r, &a, &lignes);
        assert_eq!((b.ajoutees, b.mises_a_jour, b.inchangees), (0, 0, 3));
        assert!(!b.modifie());
    }

    #[test]
    fn import_tableau_une_colonne_par_type() {
        let texte = "Liste des ordonnances accomplies\n\
                     Nom\tID FamilySearch\tBaptême\tConfirmation\tDotation\tScellement aux parents\n\
                     Jean MARTIN\tKWAA-AAA\t12/03/1990\t13/03/1990\t\tAccomplie\n\
                     Marie DURAND\tKWBB-BBB\t\t\t\t\n";
        let lignes = lire_tableau(texte).unwrap();
        assert_eq!(lignes.len(), 3);
        assert_eq!(lignes[2].ordonnance, Ordonnance { genre: "Scellement aux parents".into(), statut: ACCOMPLIE.into(), ..Default::default() });
        let mut r = Registre::new();
        let b = appliquer(&mut r, &analyser(ARBRE), &lignes);
        assert_eq!((b.personnes, b.ajoutees), (1, 3));
        // Date différente pour un baptême déjà noté : gardé tel quel, signalé.
        let autre = vec![LigneReleve {
            cle: "KWAA-AAA".into(),
            nom: String::new(),
            ordonnance: Ordonnance { genre: "Baptême".into(), statut: ACCOMPLIE.into(), date: "1 JAN 1991".into(), ..Default::default() },
        }];
        let b = appliquer(&mut r, &analyser(ARBRE), &autre);
        assert_eq!(b.conflits, 1);
        assert_eq!(r["@I1@"][0].date, "12 MAR 1990");
        assert!(lire_tableau("a;b;c\n1;2;3\n").is_err());
    }

    #[test]
    fn releve_colle() {
        let texte = "Ordonnances\nBaptism\nCompleted\n12 March 1990\nParis France Temple\n\
                     Confirmation\nCompleted\n13 March 1990\n\nScellement aux parents\nRéservée\n\n\
                     Dotation\nAccomplie le 1er juin 2001\nTemple de Berne (Suisse)\n";
        let l = lire_releve(texte);
        assert_eq!(l.len(), 4);
        assert_eq!(l[0], Ordonnance { genre: "Baptême".into(), statut: ACCOMPLIE.into(), date: "12 MAR 1990".into(), temple: "Paris France Temple".into(), notes: String::new() });
        assert_eq!((l[2].genre.as_str(), l[2].statut.as_str(), l[2].date.as_str()), ("Scellement aux parents", "Réservée", ""));
        assert_eq!((l[3].statut.as_str(), l[3].date.as_str(), l[3].temple.as_str()), (ACCOMPLIE, "1 JUN 2001", "Temple de Berne (Suisse)"));
        let mut r = Registre::new();
        let b = appliquer_releve(&mut r, "@I1@", l);
        assert_eq!(b.ajoutees, 4);
        // Un statut ne recule pas : « Accomplie » n'est pas ramené à « Réservée ».
        let mut liste = vec![Ordonnance { genre: "Baptême".into(), statut: ACCOMPLIE.into(), ..Default::default() }];
        let e = fusionner(&mut liste, Ordonnance { genre: "Baptême".into(), statut: "Réservée".into(), ..Default::default() });
        assert_eq!((e, liste[0].statut.as_str()), (Effet::Inchangee, ACCOMPLIE));
    }

    #[test]
    fn sauvegarde_du_registre() {
        let dossier = std::env::temp_dir().join(format!("genfam-ord-bak-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        let ged = dossier.join("essai.ged");
        let mut r = Registre::new();
        r.entry("@I1@".into()).or_default().push(Ordonnance { genre: TYPES[0].into(), statut: STATUTS[1].into(), ..Default::default() });
        enregistrer_avec_sauvegarde(&ged, &r).unwrap();
        let avant = std::fs::read(chemin_pour(&ged)).unwrap();
        r.get_mut("@I1@").unwrap()[0].statut = ACCOMPLIE.into();
        enregistrer_avec_sauvegarde(&ged, &r).unwrap();
        let mut bak = chemin_pour(&ged).into_os_string();
        bak.push(".bak");
        assert_eq!(std::fs::read(PathBuf::from(bak)).unwrap(), avant);
        assert_eq!(charger(&ged), r);
        std::fs::remove_dir_all(dossier).unwrap();
    }
}
