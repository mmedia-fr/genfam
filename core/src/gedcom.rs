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

    for brute in texte.lines() {
        let Some(l) = analyser_ligne(brute) else { continue };
        if l.niveau == 0 {
            courant = Courant::Rien;
            evenement = "";
            tag1.clear();
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
                    evenement = match l.tag { "BIRT" => "BIRT", "DEAT" => "DEAT", _ => "" };
                    let v = l.valeur.to_string();
                    match l.tag {
                        // Seul le premier nom compte : les suivants sont des variantes.
                        "NAME" if p.nom_gedcom.is_empty() => p.nom_gedcom = v,
                        "SEX" => p.sexe = v,
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
    fn bloc(&self, xref: &str, tag: &str) -> Option<(usize, usize)> {
        let debut = self.lignes.iter().position(|l| {
            matches!(analyser_ligne(l), Some(Ligne { niveau: 0, xref: Some(x), tag: t, .. }) if x == xref && t == tag)
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
            let Some(Ligne { niveau: 0, xref: Some(x), tag: "FAM", .. }) = analyser_ligne(l) else { continue };
            let bornes = self.bloc(x, "FAM").unwrap_or((i, i + 1));
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
