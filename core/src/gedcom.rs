// SPDX-License-Identifier: GPL-3.0-or-later
//! Lecture et modification d'un fichier GEDCOM.
//!
//! Comme la version Python, les modifications travaillent **ligne à ligne** sur le
//! fichier : tout ce que le programme ne comprend pas (sources, médias, balises
//! propres à Ancestral Quest…) est conservé tel quel. Chaque écriture laisse une
//! copie `.bak` de l'état précédent, qui sert à annuler la dernière modification.

use crate::modele::{Arbre, Famille, Personne};
use std::fs;
use std::path::Path;

/// En-tête d'un fichier créé par le programme.
pub const ENTETE_NOUVEAU: &str = "0 HEAD\n1 SOUR GENFAM\n2 NAME GenFam\n2 VERS 8.0\n1 CHAR UTF-8\n1 GEDC\n2 VERS 5.5.1\n2 FORM LINEAGE-LINKED\n0 TRLR\n";

/// Une ligne GEDCOM décomposée : `niveau [@xref@] TAG [valeur]`.
#[derive(Debug, PartialEq, Eq)]
pub struct Ligne<'a> {
    pub niveau: u32,
    pub xref: Option<&'a str>,
    pub tag: &'a str,
    pub valeur: &'a str,
}

fn est_xref(mot: &str) -> bool {
    mot.len() > 2
        && mot.starts_with('@')
        && mot.ends_with('@')
        && mot[1..mot.len() - 1].chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub fn analyser_ligne(ligne: &str) -> Option<Ligne<'_>> {
    let ligne = ligne.trim_start_matches('\u{feff}').trim_end_matches(['\r', '\n']);
    let ligne = ligne.trim_start();
    let fin_niveau = ligne.find(char::is_whitespace)?;
    let niveau: u32 = ligne[..fin_niveau].parse().ok()?;
    let mut reste = ligne[fin_niveau..].trim_start();
    let mut xref = None;
    if let Some(fin) = reste.find(char::is_whitespace) {
        if est_xref(&reste[..fin]) {
            xref = Some(&reste[..fin]);
            reste = reste[fin..].trim_start();
        }
    }
    if reste.is_empty() {
        return None;
    }
    let (tag, valeur) = match reste.find(char::is_whitespace) {
        Some(fin) => (&reste[..fin], reste[fin..].trim()),
        None => (reste, ""),
    };
    Some(Ligne { niveau, xref, tag, valeur })
}

/// Décode le contenu d'un fichier : UTF-8 (avec ou sans BOM), à défaut Windows-1252.
pub fn decoder(octets: &[u8]) -> String {
    let sans_bom = octets.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(octets);
    match std::str::from_utf8(sans_bom) {
        Ok(t) => t.to_string(),
        Err(_) => sans_bom.iter().map(|&o| cp1252(o)).collect(),
    }
}

fn cp1252(o: u8) -> char {
    const HAUT: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž', '\u{8f}',
        '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}', 'ž', 'Ÿ',
    ];
    if (0x80..0xA0).contains(&o) { HAUT[(o - 0x80) as usize] } else { o as char }
}

/// Construit l'arbre à partir du texte d'un fichier GEDCOM (portage de parse_gedcom()).
pub fn analyser(texte: &str) -> Arbre {
    let mut personnes: Vec<Personne> = Vec::new();
    let mut familles: Vec<Famille> = Vec::new();
    #[derive(PartialEq)]
    enum Courant { Rien, Personne, Famille }
    let mut courant = Courant::Rien;
    let mut evenement = "";
    let mut tag1 = String::new();
    // Seul le premier BIRT et le premier DEAT d'une personne comptent : ce sont eux
    // que la modification d'une fiche réécrit.
    let (mut naissance_vue, mut deces_vu) = (false, false);

    for brute in texte.lines() {
        let Some(l) = analyser_ligne(brute) else { continue };
        if l.niveau == 0 {
            courant = Courant::Rien;
            evenement = "";
            tag1.clear();
            naissance_vue = false;
            deces_vu = false;
            match (l.xref, l.tag) {
                (Some(x), "INDI") => {
                    personnes.push(Personne::nouvelle(x));
                    courant = Courant::Personne;
                }
                (Some(x), "FAM") => {
                    familles.push(Famille { id: x.to_string(), ..Default::default() });
                    courant = Courant::Famille;
                }
                _ => {}
            }
            continue;
        }
        match courant {
            Courant::Personne => {
                let p = personnes.last_mut().unwrap();
                if l.niveau == 1 {
                    tag1 = l.tag.to_string();
                    evenement = match l.tag {
                        "BIRT" if !naissance_vue => {
                            naissance_vue = true;
                            "BIRT"
                        }
                        "DEAT" if !deces_vu => {
                            deces_vu = true;
                            "DEAT"
                        }
                        _ => "",
                    };
                    let v = l.valeur.to_string();
                    match l.tag {
                        // Seul le premier nom compte : les suivants sont des variantes.
                        "NAME" if p.nom_gedcom.is_empty() => p.nom_gedcom = v,
                        "SEX" if p.sexe.is_empty() => p.sexe = v,
                        "FAMS" if !v.is_empty() => p.familles_conjoint.push(v),
                        "FAMC" if !v.is_empty() => p.familles_enfant.push(v),
                        "NOTE" if !v.is_empty() => p.notes.push(v),
                        "_FSFTID" if !v.is_empty() => p.fsid = v,
                        _ => {}
                    }
                } else if l.niveau == 2 {
                    let v = l.valeur.to_string();
                    match (evenement, l.tag) {
                        ("BIRT", "DATE") => p.naissance = v,
                        ("BIRT", "PLAC") => p.lieu_naissance = v,
                        ("DEAT", "DATE") => p.deces = v,
                        ("DEAT", "PLAC") => p.lieu_deces = v,
                        _ if tag1 == "NAME" && l.tag == "GIVN" && p.prenoms.is_empty() => p.prenoms = v,
                        _ if tag1 == "NAME" && l.tag == "SURN" && p.nom.is_empty() => p.nom = v,
                        // Suite d'une note sur plusieurs lignes.
                        _ if tag1 == "NOTE" && (l.tag == "CONT" || l.tag == "CONC") => {
                            if let Some(n) = p.notes.last_mut() {
                                if l.tag == "CONT" { n.push('\n'); }
                                n.push_str(&v);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Courant::Famille => {
                let f = familles.last_mut().unwrap();
                if l.niveau == 1 {
                    tag1 = l.tag.to_string();
                    evenement = if l.tag == "MARR" { "MARR" } else { "" };
                    let v = l.valeur.to_string();
                    match l.tag {
                        "HUSB" => f.mari = Some(v),
                        "WIFE" => f.epouse = Some(v),
                        "CHIL" if !v.is_empty() => f.enfants.push(v),
                        _ => {}
                    }
                } else if l.niveau == 2 && evenement == "MARR" {
                    match l.tag {
                        "DATE" => f.mariage = l.valeur.to_string(),
                        "PLAC" => f.lieu_mariage = l.valeur.to_string(),
                        _ => {}
                    }
                }
            }
            Courant::Rien => {}
        }
    }
    Arbre::construire(personnes, familles)
}

/// Texte d'un fichier GEDCOM, tenu ligne à ligne pour être modifié sans rien perdre.
#[derive(Debug, Clone, Default)]
pub struct Document {
    pub lignes: Vec<String>,
}

impl Document {
    pub fn depuis_texte(texte: &str) -> Self {
        Self { lignes: texte.lines().map(|l| l.trim_start_matches('\u{feff}').to_string()).collect() }
    }

    pub fn lire(chemin: &Path) -> std::io::Result<Self> {
        Ok(Self::depuis_texte(&decoder(&fs::read(chemin)?)))
    }

    /// Texte à écrire : UTF-8 sans BOM, fins de ligne CRLF.
    pub fn texte(&self) -> String {
        let mut t = self.lignes.join("\r\n");
        t.push_str("\r\n");
        t
    }

    fn position_trlr(&self) -> usize {
        self.lignes.iter().position(|l| l.starts_with("0 TRLR")).unwrap_or(self.lignes.len())
    }

    /// Bornes [début, fin) de l'enregistrement de niveau 0 portant cet identifiant.
    pub fn bloc(&self, xref: &str, tag: &str) -> Option<(usize, usize)> {
        let debut = self.lignes.iter().position(|l| {
            l.starts_with('0')
                && l.contains(xref)
                && matches!(analyser_ligne(l), Some(Ligne { niveau: 0, xref: Some(x), tag: t, .. }) if x == xref && t == tag)
        })?;
        let fin = self.lignes[debut + 1..]
            .iter()
            .position(|l| l.starts_with("0 "))
            .map_or(self.lignes.len(), |i| debut + 1 + i);
        Some((debut, fin))
    }

    fn bloc_contient(&self, (a, b): (usize, usize), tag: &str, valeur: &str) -> bool {
        self.lignes[a + 1..b]
            .iter()
            .any(|l| matches!(analyser_ligne(l), Some(Ligne { niveau: 1, tag: t, valeur: v, .. }) if t == tag && v == valeur))
    }

    /// Ajoute `1 TAG valeur` en fin d'enregistrement, s'il n'y figure pas déjà.
    pub fn ajouter_reference(&mut self, xref: &str, tag_bloc: &str, tag: &str, valeur: &str) -> bool {
        let Some(bornes) = self.bloc(xref, tag_bloc) else { return false };
        if self.bloc_contient(bornes, tag, valeur) {
            return false;
        }
        self.lignes.insert(bornes.1, format!("1 {tag} {valeur}"));
        true
    }

    fn prochain_numero(&self, prefixe: char, tag: &str) -> u64 {
        let max = self
            .lignes
            .iter()
            .filter_map(|l| match analyser_ligne(l) {
                Some(Ligne { niveau: 0, xref: Some(x), tag: t, .. }) if t == tag => {
                    x.trim_matches('@').strip_prefix(prefixe).and_then(|n| n.parse::<u64>().ok())
                }
                _ => None,
            })
            .max()
            .unwrap_or(0);
        max + 1
    }

    pub fn prochain_id_personne(&self) -> String {
        let mut n = self.prochain_numero('I', "INDI");
        while self.bloc(&format!("@I{n}@"), "INDI").is_some() {
            n += 1;
        }
        format!("@I{n}@")
    }

    fn prochain_id_famille(&self) -> String {
        let mut n = self.prochain_numero('F', "FAM");
        while self.bloc(&format!("@F{n}@"), "FAM").is_some() {
            n += 1;
        }
        format!("@F{n}@")
    }

    /// Enregistrement INDI d'une nouvelle personne.
    fn bloc_personne(p: &Personne) -> Vec<String> {
        let mut b = vec![format!("0 {} INDI", p.id)];
        if !p.nom_gedcom.is_empty() {
            b.push(format!("1 NAME {}", p.nom_gedcom));
        }
        if !p.prenoms.is_empty() {
            b.push(format!("2 GIVN {}", p.prenoms));
        }
        if !p.nom.is_empty() {
            b.push(format!("2 SURN {}", p.nom));
        }
        if p.sexe == "M" || p.sexe == "F" {
            b.push(format!("1 SEX {}", p.sexe));
        }
        for (tag, date, lieu) in [("BIRT", &p.naissance, &p.lieu_naissance), ("DEAT", &p.deces, &p.lieu_deces)] {
            if !date.is_empty() || !lieu.is_empty() {
                b.push(format!("1 {tag}"));
                if !date.is_empty() {
                    b.push(format!("2 DATE {date}"));
                }
                if !lieu.is_empty() {
                    b.push(format!("2 PLAC {lieu}"));
                }
            }
        }
        if !p.fsid.is_empty() {
            b.push(format!("1 _FSFTID {}", p.fsid));
        }
        b
    }

    /// Insère une personne avant `0 TRLR`, puis la relie à son père et/ou sa mère.
    pub fn ajouter_personne(&mut self, p: &Personne, pere: Option<&str>, mere: Option<&str>) {
        let pos = self.position_trlr();
        let bloc = Self::bloc_personne(p);
        self.lignes.splice(pos..pos, bloc);
        if pere.is_some() || mere.is_some() {
            self.lier_parents(&p.id, pere, mere);
        }
    }

    /// Rattache un enfant à ses parents : famille existante qui les porte déjà
    /// (même règle que la version Python), sinon famille nouvelle. Rend son identifiant.
    pub fn lier_parents(&mut self, enfant: &str, pere: Option<&str>, mere: Option<&str>) -> Option<String> {
        if pere.is_none() && mere.is_none() {
            return None;
        }
        let mut choisie = None;
        for (i, l) in self.lignes.iter().enumerate() {
            if !l.starts_with('0') {
                continue;
            }
            let Some(Ligne { niveau: 0, xref: Some(x), tag: "FAM", .. }) = analyser_ligne(l) else { continue };
            // Fin de l'enregistrement trouvée sur place : un appel à bloc() par famille
            // rendrait la recherche quadratique sur un fichier de plusieurs milliers de familles.
            let fin = self.lignes[i + 1..].iter().position(|l| l.starts_with("0 ")).map_or(self.lignes.len(), |k| i + 1 + k);
            let bornes = (i, fin);
            let a_pere = pere.is_some_and(|p| self.bloc_contient(bornes, "HUSB", p));
            let a_mere = mere.is_some_and(|m| self.bloc_contient(bornes, "WIFE", m));
            let convient = match (pere, mere) {
                (Some(_), Some(_)) => a_pere && a_mere,
                (Some(_), None) => a_pere,
                (None, Some(_)) => a_mere,
                (None, None) => false,
            };
            if convient {
                choisie = Some(x.to_string());
                break;
            }
        }
        let fam = match choisie {
            Some(f) => {
                self.ajouter_reference(&f, "FAM", "CHIL", enfant);
                f
            }
            None => {
                let f = self.prochain_id_famille();
                let mut b = vec![format!("0 {f} FAM")];
                if let Some(p) = pere {
                    b.push(format!("1 HUSB {p}"));
                }
                if let Some(m) = mere {
                    b.push(format!("1 WIFE {m}"));
                }
                b.push(format!("1 CHIL {enfant}"));
                let pos = self.position_trlr();
                self.lignes.splice(pos..pos, b);
                f
            }
        };
        self.ajouter_reference(enfant, "INDI", "FAMC", &fam);
        for parent in [pere, mere].into_iter().flatten() {
            self.ajouter_reference(parent, "INDI", "FAMS", &fam);
        }
        Some(fam)
    }

    /// Famille nouvelle réunissant le parent connu (ou les deux) et un enfant, sans
    /// reprendre une famille existante : un parent déjà marié ailleurs ne doit pas
    /// recevoir d'office le conjoint de cette autre union. Rend son identifiant.
    pub fn creer_famille(&mut self, pere: Option<&str>, mere: Option<&str>, enfant: &str) -> String {
        let f = self.prochain_id_famille();
        let mut b = vec![format!("0 {f} FAM")];
        if let Some(p) = pere {
            b.push(format!("1 HUSB {p}"));
        }
        if let Some(m) = mere {
            b.push(format!("1 WIFE {m}"));
        }
        b.push(format!("1 CHIL {enfant}"));
        let pos = self.position_trlr();
        self.lignes.splice(pos..pos, b);
        self.ajouter_reference(enfant, "INDI", "FAMC", &f);
        for parent in [pere, mere].into_iter().flatten() {
            self.ajouter_reference(parent, "INDI", "FAMS", &f);
        }
        f
    }

    /// Ajoute des lignes telles quelles en fin d'enregistrement (citations de sources).
    pub fn ajouter_lignes(&mut self, xref: &str, tag_bloc: &str, lignes: &[String]) -> bool {
        let Some((_, fin)) = self.bloc(xref, tag_bloc) else { return false };
        self.lignes.splice(fin..fin, lignes.iter().cloned());
        true
    }

    /// Supprime une personne, ses renvois, et les familles devenues vides.
    pub fn supprimer_personne(&mut self, pid: &str) {
        let mut sortie = Vec::with_capacity(self.lignes.len());
        let mut dans_bloc = false;
        for l in &self.lignes {
            let a = analyser_ligne(l);
            if let Some(Ligne { niveau: 0, xref, tag, .. }) = &a {
                dans_bloc = *xref == Some(pid) && *tag == "INDI";
            }
            if dans_bloc {
                continue;
            }
            if let Some(Ligne { niveau: 1, tag, valeur, .. }) = &a {
                if *valeur == pid && matches!(*tag, "FAMS" | "FAMC" | "HUSB" | "WIFE" | "CHIL") {
                    continue;
                }
            }
            sortie.push(l.clone());
        }
        // Une famille sans HUSB, WIFE ni CHIL n'est pas recopiée.
        let mut nettoye = Vec::with_capacity(sortie.len());
        let mut i = 0;
        while i < sortie.len() {
            if let Some(Ligne { niveau: 0, xref: Some(_), tag: "FAM", .. }) = analyser_ligne(&sortie[i]) {
                let fin = sortie[i + 1..].iter().position(|l| l.starts_with("0 ")).map_or(sortie.len(), |k| i + 1 + k);
                let membre = sortie[i + 1..fin].iter().any(|l| {
                    matches!(analyser_ligne(l), Some(Ligne { niveau: 1, tag: "HUSB" | "WIFE" | "CHIL", valeur, .. }) if est_xref(valeur))
                });
                if membre {
                    nettoye.extend_from_slice(&sortie[i..fin]);
                }
                i = fin;
                continue;
            }
            nettoye.push(sortie[i].clone());
            i += 1;
        }
        self.lignes = nettoye;
    }

    /// Retire de l'enregistrement chaque ligne `1 TAG valeur`, avec ses lignes
    /// subordonnées (`2 PEDI`, `2 _FREL`…). Rend le nombre de liens retirés.
    pub fn retirer_reference(&mut self, xref: &str, tag_bloc: &str, tag: &str, valeur: &str) -> usize {
        let Some((a, b)) = self.bloc(xref, tag_bloc) else { return 0 };
        let mut garde = Vec::with_capacity(b - a);
        let mut retires = 0;
        let mut i = a;
        while i < b {
            let l = &self.lignes[i];
            if i > a && matches!(analyser_ligne(l), Some(Ligne { niveau: 1, tag: t, valeur: v, .. }) if t == tag && v == valeur) {
                retires += 1;
                i = fin_sous_bloc(&self.lignes[..b], i);
                continue;
            }
            garde.push(l.clone());
            i += 1;
        }
        if retires > 0 {
            self.lignes.splice(a..b, garde);
        }
        retires
    }

    /// Supprime l'enregistrement d'une famille et tous les renvois `FAMS` / `FAMC` vers elle.
    pub fn supprimer_famille(&mut self, fid: &str) -> bool {
        let Some((a, b)) = self.bloc(fid, "FAM") else { return false };
        self.lignes.drain(a..b);
        let mut sortie = Vec::with_capacity(self.lignes.len());
        let mut sauter = false;
        for l in std::mem::take(&mut self.lignes) {
            if sauter {
                if niveau_de(&l).is_some_and(|n| n > 1) {
                    continue;
                }
                sauter = false;
            }
            if l.contains(fid)
                && matches!(analyser_ligne(&l), Some(Ligne { niveau: 1, tag: "FAMS" | "FAMC", valeur, .. }) if valeur == fid)
            {
                sauter = true;
                continue;
            }
            sortie.push(l);
        }
        self.lignes = sortie;
        true
    }

    /// Union nouvelle, sans enfant, entre deux conjoints. Rend l'identifiant de la famille.
    pub fn creer_union(&mut self, mari: Option<&str>, epouse: Option<&str>) -> String {
        let f = self.prochain_id_famille();
        let mut b = vec![format!("0 {f} FAM")];
        if let Some(p) = mari {
            b.push(format!("1 HUSB {p}"));
        }
        if let Some(m) = epouse {
            b.push(format!("1 WIFE {m}"));
        }
        let pos = self.position_trlr();
        self.lignes.splice(pos..pos, b);
        for c in [mari, epouse].into_iter().flatten() {
            self.ajouter_reference(c, "INDI", "FAMS", &f);
        }
        f
    }

    /// Place un conjoint (`HUSB` ou `WIFE`) dans une famille existante, en tête de
    /// l'enregistrement, et pose le renvoi `FAMS` sur sa fiche.
    pub fn inserer_conjoint(&mut self, fid: &str, balise: &str, pid: &str) -> bool {
        let Some((a, b)) = self.bloc(fid, "FAM") else { return false };
        if self.bloc_contient((a, b), balise, pid) {
            return false;
        }
        let mut pos = a + 1;
        if balise == "WIFE" && pos < b && matches!(analyser_ligne(&self.lignes[pos]), Some(Ligne { niveau: 1, tag: "HUSB", .. })) {
            pos = fin_sous_bloc(&self.lignes[..b], pos);
        }
        self.lignes.insert(pos, format!("1 {balise} {pid}"));
        self.ajouter_reference(pid, "INDI", "FAMS", fid);
        true
    }

    /// Membres d'une famille tels qu'ils figurent dans le fichier.
    pub fn membres(&self, fid: &str) -> Option<Membres> {
        let (a, b) = self.bloc(fid, "FAM")?;
        let mut m = Membres::default();
        for l in &self.lignes[a + 1..b] {
            let Some(Ligne { niveau: 1, tag, valeur, .. }) = analyser_ligne(l) else { continue };
            match tag {
                "HUSB" if est_xref(valeur) => m.mari = Some(valeur.to_string()),
                "WIFE" if est_xref(valeur) => m.epouse = Some(valeur.to_string()),
                "CHIL" if est_xref(valeur) => m.enfants.push(valeur.to_string()),
                "HUSB" | "WIFE" | "CHIL" | "CHAN" | "_UID" | "RIN" | "_UPD" | "REFN" => {}
                _ => m.autres_donnees = true,
            }
        }
        Some(m)
    }

    /// Réécrit l'identité d'une personne (nom, sexe, naissance, décès) sans toucher au
    /// reste de sa fiche. Seuls les champs qui diffèrent de `actuel` sont réécrits.
    /// Rend vrai si le fichier a changé.
    pub fn modifier_identite(&mut self, id: &str, actuel: &Identite, nouveau: &Identite) -> Result<bool, String> {
        let (a, b) = self.bloc(id, "INDI").ok_or("Personne introuvable dans le fichier GEDCOM.")?;
        let mut bloc: Vec<String> = self.lignes[a..b].to_vec();
        if nouveau.prenoms != actuel.prenoms || nouveau.nom != actuel.nom {
            modifier_nom(&mut bloc, &nouveau.prenoms, &nouveau.nom);
        }
        if nouveau.sexe != actuel.sexe {
            modifier_sexe(&mut bloc, &nouveau.sexe);
        }
        for (tag, date, lieu, date0, lieu0) in [
            ("BIRT", &nouveau.naissance, &nouveau.lieu_naissance, &actuel.naissance, &actuel.lieu_naissance),
            ("DEAT", &nouveau.deces, &nouveau.lieu_deces, &actuel.deces, &actuel.lieu_deces),
        ] {
            let d = (date != date0).then_some(date.as_str());
            let l = (lieu != lieu0).then_some(lieu.as_str());
            if d.is_some() || l.is_some() {
                modifier_evenement(&mut bloc, tag, d, l);
            }
        }
        if bloc[..] == self.lignes[a..b] {
            return Ok(false);
        }
        self.lignes.splice(a..b, bloc);
        Ok(true)
    }
}

/// Membres d'une famille relevés dans le fichier.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Membres {
    pub mari: Option<String>,
    pub epouse: Option<String>,
    pub enfants: Vec<String>,
    /// Vrai si la famille porte autre chose que ses membres (mariage, source, note…).
    pub autres_donnees: bool,
}

/// Identité d'une personne telle que la présente la fiche, et telle qu'elle se modifie.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Identite {
    pub prenoms: String,
    pub nom: String,
    pub sexe: String,
    pub naissance: String,
    pub lieu_naissance: String,
    pub deces: String,
    pub lieu_deces: String,
}

impl Identite {
    /// Valeurs présentées dans la fiche. Un nom sans barres GEDCOM (« Jean Dupont »)
    /// se présente en entier dans les prénoms plutôt que de paraître vide.
    pub fn de(p: &Personne) -> Self {
        let (mut prenoms, nom) = (p.prenoms_affiches(), p.nom_famille());
        if prenoms.is_empty() && nom.is_empty() {
            prenoms = p.nom_gedcom.trim().to_string();
        }
        Self {
            prenoms,
            nom,
            sexe: p.sexe.trim().to_string(),
            naissance: p.naissance.trim().to_string(),
            lieu_naissance: p.lieu_naissance.trim().to_string(),
            deces: p.deces.trim().to_string(),
            lieu_deces: p.lieu_deces.trim().to_string(),
        }
    }

    /// Contrôle et nettoie une saisie : espaces superflus, sexe M, F ou U.
    pub fn nettoyee(&self) -> Result<Self, String> {
        let net = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
        let sexe = net(&self.sexe).to_uppercase();
        let sexe = match sexe.as_str() {
            "" | "M" | "F" | "U" => sexe,
            "H" | "MASCULIN" | "HOMME" => "M".into(),
            "FÉMININ" | "FEMININ" | "FEMME" => "F".into(),
            "INCONNU" | "?" => "U".into(),
            _ => return Err("Le sexe s'indique par M, F ou U (inconnu).".into()),
        };
        let r = Self {
            prenoms: net(&self.prenoms),
            nom: net(&self.nom).replace('/', ""),
            sexe,
            naissance: net(&self.naissance),
            lieu_naissance: net(&self.lieu_naissance),
            deces: net(&self.deces),
            lieu_deces: net(&self.lieu_deces),
        };
        if r.prenoms.is_empty() && r.nom.is_empty() {
            return Err("Indiquez au moins un prénom ou un nom.".into());
        }
        Ok(r)
    }

    /// Valeur de `1 NAME` : « Prénoms /NOM/ ».
    pub fn nom_gedcom(&self) -> String {
        if self.nom.is_empty() { self.prenoms.clone() } else { format!("{} /{}/", self.prenoms, self.nom).trim().to_string() }
    }

    /// Personne nouvelle portant cette identité.
    pub fn personne(&self, id: &str) -> Personne {
        let mut p = Personne::nouvelle(id);
        p.nom_gedcom = self.nom_gedcom();
        p.prenoms = self.prenoms.clone();
        p.nom = self.nom.clone();
        p.sexe = self.sexe.clone();
        p.naissance = self.naissance.clone();
        p.lieu_naissance = self.lieu_naissance.clone();
        p.deces = self.deces.clone();
        p.lieu_deces = self.lieu_deces.clone();
        p
    }
}

/// Niveau d'une ligne GEDCOM, lu sur son premier mot.
fn niveau_de(l: &str) -> Option<u32> {
    l.trim_start().split_whitespace().next()?.parse().ok()
}

fn valeur_de(l: &str) -> String {
    analyser_ligne(l).map(|x| x.valeur.to_string()).unwrap_or_default()
}

/// Fin (exclue) de la structure ouverte par la ligne `i` : ses lignes subordonnées.
fn fin_sous_bloc(lignes: &[String], i: usize) -> usize {
    let n = niveau_de(&lignes[i]).unwrap_or(0);
    let mut j = i + 1;
    while j < lignes.len() && niveau_de(&lignes[j]).is_some_and(|m| m > n) {
        j += 1;
    }
    j
}

/// Première ligne de niveau 1 portant ce tag dans un enregistrement (index 0 = en-tête).
fn trouver1(bloc: &[String], tag: &str) -> Option<usize> {
    (1..bloc.len()).find(|&i| matches!(analyser_ligne(&bloc[i]), Some(Ligne { niveau: 1, tag: t, .. }) if t == tag))
}

/// Première sous-ligne de niveau 2 portant ce tag dans la structure ouverte en `i`.
fn trouver2(bloc: &[String], i: usize, tag: &str) -> Option<usize> {
    (i + 1..fin_sous_bloc(bloc, i)).find(|&j| matches!(analyser_ligne(&bloc[j]), Some(Ligne { niveau: 2, tag: t, .. }) if t == tag))
}

fn retirer_structure(bloc: &mut Vec<String>, i: usize) {
    let fin = fin_sous_bloc(bloc, i);
    bloc.drain(i..fin);
}

fn modifier_nom(bloc: &mut Vec<String>, prenoms: &str, nom: &str) {
    let base = if nom.is_empty() { prenoms.to_string() } else { format!("{prenoms} /{nom}/").trim().to_string() };
    let Some(i) = trouver1(bloc, "NAME") else {
        let mut l = vec![format!("1 NAME {base}")];
        if !prenoms.is_empty() {
            l.push(format!("2 GIVN {prenoms}"));
        }
        if !nom.is_empty() {
            l.push(format!("2 SURN {nom}"));
        }
        bloc.splice(1..1, l);
        return;
    };
    // Ce qui suit le nom de famille (« Jr », « dit Martin ») est conservé.
    let ancien = valeur_de(&bloc[i]);
    let suffixe = ancien.splitn(3, '/').nth(2).map(str::trim).unwrap_or("").to_string();
    let valeur = if suffixe.is_empty() || nom.is_empty() { base } else { format!("{base} {suffixe}") };
    bloc[i] = format!("1 NAME {valeur}").trim_end().to_string();
    for (tag, v) in [("GIVN", prenoms), ("SURN", nom)] {
        if let Some(j) = trouver2(bloc, i, tag) {
            if v.is_empty() {
                retirer_structure(bloc, j);
            } else {
                bloc[j] = format!("2 {tag} {v}");
            }
        }
    }
}

fn modifier_sexe(bloc: &mut Vec<String>, sexe: &str) {
    match trouver1(bloc, "SEX") {
        Some(i) if sexe.is_empty() => retirer_structure(bloc, i),
        Some(i) => bloc[i] = format!("1 SEX {sexe}"),
        None if sexe.is_empty() => {}
        None => {
            let pos = trouver1(bloc, "NAME").map_or(1, |i| fin_sous_bloc(bloc, i));
            bloc.insert(pos, format!("1 SEX {sexe}"));
        }
    }
}

/// Date et lieu d'un événement (`BIRT`, `DEAT`) : `None` laisse la valeur en place,
/// une chaîne vide la retire. Un événement qui n'a plus rien à dire est retiré.
fn modifier_evenement(bloc: &mut Vec<String>, tag: &str, date: Option<&str>, lieu: Option<&str>) {
    let Some(i) = trouver1(bloc, tag) else {
        let (d, l) = (date.unwrap_or(""), lieu.unwrap_or(""));
        if d.is_empty() && l.is_empty() {
            return;
        }
        let mut lignes = vec![format!("1 {tag}")];
        if !d.is_empty() {
            lignes.push(format!("2 DATE {d}"));
        }
        if !l.is_empty() {
            lignes.push(format!("2 PLAC {l}"));
        }
        // La naissance avant le décès, le décès juste après la naissance ; à défaut,
        // l'un et l'autre après le nom et le sexe.
        let apres_identite = ["NAME", "SEX"].iter().filter_map(|t| trouver1(bloc, t)).map(|i| fin_sous_bloc(bloc, i)).max().unwrap_or(1);
        let pos = if tag == "BIRT" {
            trouver1(bloc, "DEAT").unwrap_or(apres_identite)
        } else {
            trouver1(bloc, "BIRT").map_or(apres_identite, |i| fin_sous_bloc(bloc, i))
        };
        bloc.splice(pos..pos, lignes);
        return;
    };
    for (sous, valeur) in [("DATE", date), ("PLAC", lieu)] {
        let Some(v) = valeur else { continue };
        match trouver2(bloc, i, sous) {
            Some(j) if v.is_empty() => retirer_structure(bloc, j),
            Some(j) => bloc[j] = format!("2 {sous} {v}"),
            None if v.is_empty() => {}
            None => {
                let pos = if sous == "PLAC" { trouver2(bloc, i, "DATE").map_or(i + 1, |d| fin_sous_bloc(bloc, d)) } else { i + 1 };
                bloc.insert(pos, format!("2 {sous} {v}"));
            }
        }
    }
    if fin_sous_bloc(bloc, i) == i + 1 && valeur_de(&bloc[i]).is_empty() {
        bloc.remove(i);
    }
}

/// Écrit le document après avoir copié l'état précédent du fichier dans `<fichier>.bak`.
pub fn ecrire_avec_sauvegarde(chemin: &Path, doc: &Document) -> std::io::Result<()> {
    if chemin.is_file() {
        fs::copy(chemin, sauvegarde(chemin))?;
    }
    fs::write(chemin, doc.texte())
}

pub fn sauvegarde(chemin: &Path) -> std::path::PathBuf {
    let mut s = chemin.as_os_str().to_owned();
    s.push(".bak");
    s.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXEMPLE: &str = include_str!("../tests/donnees/exemple.ged");

    #[test]
    fn ligne_decomposee() {
        assert_eq!(
            analyser_ligne("0 @I1@ INDI"),
            Some(Ligne { niveau: 0, xref: Some("@I1@"), tag: "INDI", valeur: "" })
        );
        assert_eq!(
            analyser_ligne("1 NAME Jean  /DUPONT/ "),
            Some(Ligne { niveau: 1, xref: None, tag: "NAME", valeur: "Jean  /DUPONT/" })
        );
        assert_eq!(analyser_ligne("\u{feff}0 HEAD").unwrap().tag, "HEAD");
        assert!(analyser_ligne("").is_none());
    }

    #[test]
    fn decodage_cp1252() {
        assert_eq!(decoder(&[0xEF, 0xBB, 0xBF, b'a']), "a");
        assert_eq!(decoder(&[b'd', 0xE9, b'c', 0xE8, b's', 0x80]), "décès€");
    }

    #[test]
    fn lecture_exemple() {
        let a = analyser(EXEMPLE);
        assert_eq!(a.personnes.len(), 7);
        assert_eq!(a.familles.len(), 3);
        let p = a.personne("@I1@").unwrap();
        assert_eq!(p.nom_affiche(), "Paul MARTIN");
        assert_eq!(p.nom_liste(), "MARTIN Paul");
        assert_eq!(p.fsid, "LZ1A-B2C");
        assert_eq!(p.naissance, "12 MAR 1950");
        assert_eq!(p.mariage, "5 JUN 1975");
        assert_eq!(a.parents(p).len(), 2);
        assert_eq!(a.enfants(a.personne("@I2@").unwrap()).len(), 2);
        assert_eq!(a.fratrie(p)[0].id, "@I4@");
        assert_eq!(a.personne("@I7@").unwrap().notes, vec!["Ligne un\nLigne deux".to_string()]);
    }

    #[test]
    fn ahnentafel_et_ascendants() {
        let a = analyser(EXEMPLE);
        let n = a.ahnentafel("@I1@");
        assert_eq!(n["@I1@"], vec![1]);
        assert_eq!(n["@I2@"], vec![2]);
        assert_eq!(n["@I3@"], vec![3]);
        assert_eq!(n["@I5@"], vec![4]);
        assert!(!n.contains_key("@I4@"));
        let asc = a.ascendants(a.personne("@I1@").unwrap());
        assert_eq!(asc.iter().map(|(g, p)| (*g, p.id.as_str())).collect::<Vec<_>>(),
                   vec![(1, "@I2@"), (2, "@I5@"), (1, "@I3@")]);
        let desc = a.descendants(a.personne("@I5@").unwrap());
        assert_eq!(desc.len(), 3);
    }

    #[test]
    fn ajout_puis_suppression() {
        let mut d = Document::depuis_texte(EXEMPLE);
        let id = d.prochain_id_personne();
        assert_eq!(id, "@I8@");
        let mut p = Personne::nouvelle(&id);
        p.nom_gedcom = "Lucie /MARTIN/".into();
        p.prenoms = "Lucie".into();
        p.nom = "MARTIN".into();
        p.sexe = "F".into();
        d.ajouter_personne(&p, Some("@I2@"), Some("@I3@"));
        let a = analyser(&d.texte());
        let lucie = a.personne("@I8@").unwrap();
        assert_eq!(lucie.familles_enfant, vec!["@F1@".to_string()]);
        assert_eq!(a.fratrie(lucie).len(), 2);
        assert!(d.lignes.last().unwrap().starts_with("0 TRLR"));

        // Parents sans famille commune : une famille nouvelle est créée.
        let id2 = d.prochain_id_personne();
        let mut q = Personne::nouvelle(&id2);
        q.nom_gedcom = "Enfant /X/".into();
        d.ajouter_personne(&q, Some("@I4@"), None);
        let a = analyser(&d.texte());
        let fam = &a.personne(&id2).unwrap().familles_enfant[0];
        assert_eq!(fam, "@F4@");
        assert!(a.personne("@I4@").unwrap().familles_conjoint.contains(fam));

        d.supprimer_personne(&id2);
        let a = analyser(&d.texte());
        assert!(a.personne(&id2).is_none());
        assert!(a.famille("@F4@").unwrap().enfants.is_empty());
        // Le dernier membre parti, la famille disparaît.
        d.supprimer_personne("@I4@");
        let a = analyser(&d.texte());
        assert!(a.famille("@F4@").is_none(), "famille vidée non supprimée");
        assert_eq!(a.famille("@F1@").unwrap().enfants, vec!["@I1@".to_string(), "@I8@".to_string()]);
        // Les balises inconnues sont conservées.
        assert!(d.texte().contains("1 _UID 1234"));
    }
}
