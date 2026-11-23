// SPDX-License-Identifier: GPL-3.0-or-later
//! Modifications de la fiche d'une personne : identité, parents, conjoints, enfants.
//!
//! Chaque opération travaille sur le texte GEDCOM (`Document`) et ne touche qu'aux
//! lignes qu'elle doit écrire : tout le reste du fichier — sources, notes, balises
//! propres à un autre logiciel — est conservé. Les liens se posent toujours des deux
//! côtés : `HUSB` / `WIFE` / `CHIL` dans la famille, `FAMS` / `FAMC` dans la fiche de
//! la personne. Aucune opération n'écrit sur le disque : l'appelant enregistre le
//! document, avec sa copie `.bak`, si elle a réussi.

use crate::gedcom::{analyser, Document, Identite};
use crate::modele::{Arbre, Famille, Personne};

/// Lien d'un proche avec la personne de la fiche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Pere,
    Mere,
    /// Père ou mère, selon le sexe ou la place occupée dans la famille.
    Parent,
    Conjoint,
    Enfant,
}

impl Relation {
    pub fn depuis(t: &str) -> Option<Self> {
        match t {
            "pere" => Some(Self::Pere),
            "mere" => Some(Self::Mere),
            "parent" => Some(Self::Parent),
            "conjoint" => Some(Self::Conjoint),
            "enfant" => Some(Self::Enfant),
            _ => None,
        }
    }
}

/// Union d'une personne, telle que la propose le choix d'union.
#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    pub famille: String,
    pub conjoint: Option<String>,
    pub enfants: usize,
    pub mariage: String,
}

/// Arbre relu depuis le document en cours de modification.
pub fn arbre_de(doc: &Document) -> Arbre {
    analyser(&doc.lignes.join("\n"))
}

fn introuvable(id: &str) -> String {
    format!("Personne introuvable dans le fichier GEDCOM : {id}.")
}

/// Famille d'origine d'une personne : la première où figure un parent, à défaut la
/// première où elle est inscrite comme enfant. C'est celle que lit `pere_mere()`.
pub fn famille_origine<'a>(arbre: &'a Arbre, p: &Personne) -> Option<&'a Famille> {
    let familles: Vec<&Famille> = p.familles_enfant.iter().filter_map(|f| arbre.famille(f)).collect();
    familles.iter().copied().find(|f| f.mari.is_some() || f.epouse.is_some()).or_else(|| familles.first().copied())
}

/// La place `x`, sauf si elle est tenue par `id`.
fn sauf<'a>(x: &'a Option<String>, id: &str) -> Option<&'a str> {
    x.as_deref().filter(|x| *x != id)
}

fn couple(f: Option<&Famille>) -> (Option<&str>, Option<&str>) {
    f.map_or((None, None), |f| (f.mari.as_deref(), f.epouse.as_deref()))
}

/// Unions d'une personne, dans l'ordre de sa fiche.
pub fn unions(arbre: &Arbre, p: &Personne) -> Vec<Union> {
    p.familles_conjoint
        .iter()
        .filter_map(|fid| arbre.famille(fid))
        .map(|f| {
            let autre = if f.mari.as_deref() == Some(p.id.as_str()) { &f.epouse } else { &f.mari };
            Union { famille: f.id.clone(), conjoint: autre.clone(), enfants: f.enfants.len(), mariage: f.mariage.clone() }
        })
        .collect()
}

fn nom(arbre: &Arbre, id: &str) -> String {
    arbre.personne(id).map_or_else(|| id.to_string(), Personne::nom_affiche)
}

fn libelle_couple(arbre: &Arbre, pere: Option<&str>, mere: Option<&str>) -> String {
    match (pere, mere) {
        (Some(p), Some(m)) => format!("{} et {}", nom(arbre, p), nom(arbre, m)),
        (Some(x), None) | (None, Some(x)) => nom(arbre, x),
        (None, None) => "aucun parent".into(),
    }
}

/// Un parent ne peut descendre de son enfant : le lien fermerait une boucle.
fn verifier_boucle(arbre: &Arbre, parent: &Personne, enfant: &Personne) -> Result<(), String> {
    if parent.id == enfant.id {
        return Err("Une personne ne peut être son propre parent.".into());
    }
    if arbre.descendants(enfant).iter().any(|(_, d)| d.id == parent.id) {
        return Err(format!(
            "{} descend de {} : ce lien fermerait une boucle dans l'arbre.",
            parent.nom_affiche(),
            enfant.nom_affiche()
        ));
    }
    Ok(())
}

/// Le sexe enregistré doit convenir à la place de père ou de mère.
fn verifier_sexe(p: &Personne, pere: bool) -> Result<(), String> {
    match (pere, p.sexe.as_str()) {
        (true, "F") => Err(format!("{} est de sexe féminin : elle ne peut être le père.", p.nom_affiche())),
        (false, "M") => Err(format!("{} est de sexe masculin : il ne peut être la mère.", p.nom_affiche())),
        _ => Ok(()),
    }
}

/// Retire l'enfant d'une famille, des deux côtés.
fn detacher(doc: &mut Document, fid: &str, enfant: &str) {
    doc.retirer_reference(fid, "FAM", "CHIL", enfant);
    doc.retirer_reference(enfant, "INDI", "FAMC", fid);
}

/// Supprime une famille devenue vide de sens : plus d'enfant, au plus un conjoint, et
/// rien d'autre à conserver (ni mariage, ni source, ni note).
fn nettoyer(doc: &mut Document, fid: &str) {
    if let Some(m) = doc.membres(fid) {
        let conjoints = usize::from(m.mari.is_some()) + usize::from(m.epouse.is_some());
        if m.enfants.is_empty() && conjoints <= 1 && !m.autres_donnees {
            doc.supprimer_famille(fid);
        }
    }
}

/// Rattache un enfant au couple (`pere`, `mere`), en le retirant de sa famille d'origine.
///
/// Dans l'ordre : la famille `cible` si elle est donnée, sinon une famille existante du
/// même couple ; à défaut, la famille d'origine si l'enfant y est seul et qu'il ne s'agit
/// que d'en remplir une place libre ; à défaut, une famille nouvelle. Une union existante
/// n'est jamais modifiée pour un seul enfant : ses autres enfants et son mariage restent
/// tels quels. La famille quittée est supprimée si elle n'a plus de sens.
fn rattacher(
    doc: &mut Document,
    arbre: &Arbre,
    enfant: &str,
    origine: Option<&Famille>,
    cible: Option<&Famille>,
    pere: Option<&str>,
    mere: Option<&str>,
) {
    if couple(origine) == (pere, mere) && cible.is_none_or(|c| origine.is_some_and(|o| o.id == c.id)) {
        return;
    }
    let quitter = |doc: &mut Document| {
        if let Some(o) = origine {
            detacher(doc, &o.id, enfant);
            nettoyer(doc, &o.id);
        }
    };
    if pere.is_none() && mere.is_none() {
        quitter(doc);
        return;
    }
    let meme = cible.or_else(|| {
        arbre.familles.iter().find(|f| {
            f.mari.as_deref() == pere && f.epouse.as_deref() == mere && origine.is_none_or(|o| o.id != f.id)
        })
    });
    if let Some(t) = meme {
        if !t.enfants.iter().any(|c| c == enfant) {
            doc.ajouter_reference(&t.id, "FAM", "CHIL", enfant);
        }
        doc.ajouter_reference(enfant, "INDI", "FAMC", &t.id);
        quitter(doc);
        return;
    }
    if let Some(o) = origine {
        let seul = o.enfants.iter().all(|c| c == enfant);
        let complete = (o.mari.is_none() || o.mari.as_deref() == pere) && (o.epouse.is_none() || o.epouse.as_deref() == mere);
        if seul && complete {
            if let (None, Some(p)) = (&o.mari, pere) {
                doc.inserer_conjoint(&o.id, "HUSB", p);
            }
            if let (None, Some(m)) = (&o.epouse, mere) {
                doc.inserer_conjoint(&o.id, "WIFE", m);
            }
            return;
        }
    }
    doc.creer_famille(pere, mere, enfant);
    quitter(doc);
}

/// Crée une personne isolée. Rend son identifiant.
pub fn creer_personne(doc: &mut Document, identite: &Identite) -> Result<String, String> {
    let identite = identite.nettoyee()?;
    let id = doc.prochain_id_personne();
    doc.ajouter_personne(&identite.personne(&id), None, None);
    Ok(id)
}

/// Réécrit l'identité d'une personne. Rend vrai si quelque chose a changé.
pub fn modifier_identite(doc: &mut Document, id: &str, identite: &Identite) -> Result<bool, String> {
    let nouveau = identite.nettoyee()?;
    let arbre = arbre_de(doc);
    let p = arbre.personne(id).ok_or_else(|| introuvable(id))?;
    doc.modifier_identite(id, &Identite::de(p), &nouveau)
}

/// Donne à une personne ce père et cette mère (`None` : pas de parent à cette place).
pub fn definir_parents(doc: &mut Document, enfant: &str, pere: Option<&str>, mere: Option<&str>) -> Result<String, String> {
    let arbre = arbre_de(doc);
    let e = arbre.personne(enfant).ok_or_else(|| introuvable(enfant))?;
    if pere.is_some() && pere == mere {
        return Err("Le père et la mère doivent être deux personnes différentes.".into());
    }
    for (id, est_pere) in [(pere, true), (mere, false)] {
        if let Some(id) = id {
            let p = arbre.personne(id).ok_or_else(|| introuvable(id))?;
            verifier_sexe(p, est_pere)?;
            verifier_boucle(&arbre, p, e)?;
        }
    }
    let origine = famille_origine(&arbre, e);
    if couple(origine) == (pere, mere) {
        return Ok("Aucun changement : ce sont déjà ses parents.".into());
    }
    rattacher(doc, &arbre, enfant, origine, None, pere, mere);
    Ok(match (pere, mere) {
        (None, None) => format!("{} n'a plus de parents enregistrés.", e.nom_affiche()),
        _ => format!("Parents de {} : {}.", e.nom_affiche(), libelle_couple(&arbre, pere, mere)),
    })
}

/// Relie une personne existante (`autre`) à la personne de la fiche. `famille` désigne
/// l'union choisie pour un enfant ou un conjoint (`None` : union nouvelle).
pub fn lier(doc: &mut Document, personne: &str, relation: Relation, autre: &str, famille: Option<&str>) -> Result<String, String> {
    let arbre = arbre_de(doc);
    let p = arbre.personne(personne).ok_or_else(|| introuvable(personne))?;
    let a = arbre.personne(autre).ok_or_else(|| introuvable(autre))?;
    if p.id == a.id {
        return Err("Une personne ne peut être reliée à elle-même.".into());
    }
    match relation {
        Relation::Pere | Relation::Mere | Relation::Parent => {
            let est_pere = match relation {
                Relation::Pere => true,
                Relation::Mere => false,
                _ => match a.sexe.as_str() {
                    "M" => true,
                    "F" => false,
                    _ => return Err("Indiquez s'il s'agit du père ou de la mère.".into()),
                },
            };
            verifier_sexe(a, est_pere)?;
            verifier_boucle(&arbre, a, p)?;
            let origine = famille_origine(&arbre, p);
            let (pere0, mere0) = couple(origine);
            let (place, article) = if est_pere { (pere0, "le père") } else { (mere0, "la mère") };
            if let Some(x) = place {
                return Err(if x == a.id {
                    format!("{} est déjà {article} de {}.", a.nom_affiche(), p.nom_affiche())
                } else {
                    format!(
                        "{} a déjà {article} : {}.\n\nPour le remplacer, utilisez « Modifier les parents ».",
                        p.nom_affiche(),
                        nom(&arbre, x)
                    )
                });
            }
            let (pere, mere) = if est_pere { (Some(a.id.as_str()), mere0) } else { (pere0, Some(a.id.as_str())) };
            rattacher(doc, &arbre, &p.id, origine, None, pere, mere);
            Ok(format!("{} est désormais {article} de {}.", a.nom_affiche(), p.nom_affiche()))
        }
        Relation::Conjoint => {
            if arbre.conjoints(p).iter().any(|c| c.id == a.id) {
                return Err(format!("{} et {} sont déjà unis.", p.nom_affiche(), a.nom_affiche()));
            }
            match famille {
                Some(fid) => {
                    let f = arbre.famille(fid).ok_or("Union introuvable dans le fichier GEDCOM.")?;
                    let balise = match (f.mari.as_deref(), f.epouse.as_deref()) {
                        (Some(m), None) if m == p.id => "WIFE",
                        (None, Some(e)) if e == p.id => "HUSB",
                        _ => return Err("Cette union a déjà ses deux conjoints, ou n'est pas une union de cette personne.".into()),
                    };
                    doc.inserer_conjoint(fid, balise, &a.id);
                    let n = f.enfants.len();
                    let suite = if n == 0 {
                        String::new()
                    } else {
                        format!("\n\nCette union compte {n} enfant(s) : {} en devient aussi le parent.", a.nom_affiche())
                    };
                    Ok(format!("{} et {} sont désormais unis (union {fid}).{suite}", p.nom_affiche(), a.nom_affiche()))
                }
                None => {
                    let p_epouse = p.sexe == "F" || (p.sexe != "M" && a.sexe == "M");
                    let (mari, epouse) = if p_epouse { (a, p) } else { (p, a) };
                    let f = doc.creer_union(Some(&mari.id), Some(&epouse.id));
                    Ok(format!("{} et {} sont désormais unis (union {f}).", p.nom_affiche(), a.nom_affiche()))
                }
            }
        }
        Relation::Enfant => {
            verifier_boucle(&arbre, p, a)?;
            let cible = match famille {
                Some(fid) => {
                    let f = arbre.famille(fid).ok_or("Union introuvable dans le fichier GEDCOM.")?;
                    if f.mari.as_deref() != Some(p.id.as_str()) && f.epouse.as_deref() != Some(p.id.as_str()) {
                        return Err("Cette union n'est pas une union de cette personne.".into());
                    }
                    if f.enfants.iter().any(|c| c == &a.id) {
                        return Err(format!("{} est déjà enfant de cette union.", a.nom_affiche()));
                    }
                    Some(f)
                }
                None => None,
            };
            let (pere, mere) = match cible {
                Some(f) => (f.mari.as_deref(), f.epouse.as_deref()),
                None if p.sexe == "F" => (None, Some(p.id.as_str())),
                None => (Some(p.id.as_str()), None),
            };
            let origine = famille_origine(&arbre, a);
            let (pere0, mere0) = couple(origine);
            if pere0.is_some_and(|x| Some(x) != pere) || mere0.is_some_and(|x| Some(x) != mere) {
                return Err(format!(
                    "{} a déjà des parents : {}.\n\nPour les changer, ouvrez sa fiche et utilisez « Modifier les parents ».",
                    a.nom_affiche(),
                    libelle_couple(&arbre, pere0, mere0)
                ));
            }
            if couple(origine) == (pere, mere) && cible.is_none() {
                return Err(format!("{} est déjà enfant de {}.", a.nom_affiche(), p.nom_affiche()));
            }
            rattacher(doc, &arbre, &a.id, origine, cible, pere, mere);
            Ok(format!("{} est désormais enfant de {}.", a.nom_affiche(), libelle_couple(&arbre, pere, mere)))
        }
    }
}

/// Retire le lien entre la personne de la fiche et un proche. Aucune personne n'est
/// supprimée : seuls les renvois entre elles et les familles vidées disparaissent.
pub fn retirer_lien(doc: &mut Document, personne: &str, relation: Relation, autre: &str) -> Result<String, String> {
    let arbre = arbre_de(doc);
    let p = arbre.personne(personne).ok_or_else(|| introuvable(personne))?;
    let a = arbre.personne(autre).ok_or_else(|| introuvable(autre))?;
    match relation {
        Relation::Pere | Relation::Mere | Relation::Parent | Relation::Enfant => {
            let (enfant, parent) = if relation == Relation::Enfant { (a, p) } else { (p, a) };
            let f = enfant
                .familles_enfant
                .iter()
                .filter_map(|fid| arbre.famille(fid))
                .find(|f| f.mari.as_deref() == Some(parent.id.as_str()) || f.epouse.as_deref() == Some(parent.id.as_str()))
                .ok_or_else(|| format!("{} n'est pas un parent de {}.", parent.nom_affiche(), enfant.nom_affiche()))?;
            let (pere, mere) = (sauf(&f.mari, &parent.id), sauf(&f.epouse, &parent.id));
            rattacher(doc, &arbre, &enfant.id, Some(f), None, pere, mere);
            let reste = match (pere, mere) {
                (None, None) => String::new(),
                _ => format!(" Il ou elle reste l'enfant de {}.", libelle_couple(&arbre, pere, mere)),
            };
            Ok(format!("{} n'est plus parent de {}.{reste}", parent.nom_affiche(), enfant.nom_affiche()))
        }
        Relation::Conjoint => {
            let familles: Vec<&Famille> = p
                .familles_conjoint
                .iter()
                .filter_map(|fid| arbre.famille(fid))
                .filter(|f| {
                    let membres = [f.mari.as_deref(), f.epouse.as_deref()];
                    membres.contains(&Some(p.id.as_str())) && membres.contains(&Some(a.id.as_str()))
                })
                .collect();
            if familles.is_empty() {
                return Err(format!("{} et {} ne sont pas unis.", p.nom_affiche(), a.nom_affiche()));
            }
            let mut lignes = Vec::new();
            for f in familles {
                if f.enfants.is_empty() {
                    doc.supprimer_famille(&f.id);
                    lignes.push(format!("L'union {} a été supprimée.", f.id));
                } else {
                    let balise = if f.mari.as_deref() == Some(a.id.as_str()) { "HUSB" } else { "WIFE" };
                    doc.retirer_reference(&f.id, "FAM", balise, &a.id);
                    doc.retirer_reference(&a.id, "INDI", "FAMS", &f.id);
                    lignes.push(format!(
                        "Les {} enfant(s) de l'union {} restent rattachés à {}.",
                        f.enfants.len(),
                        f.id,
                        p.nom_affiche()
                    ));
                }
            }
            Ok(format!("{} n'est plus le conjoint de {}.\n\n{}", a.nom_affiche(), p.nom_affiche(), lignes.join("\n")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gedcom::{analyser_ligne, ecrire_avec_sauvegarde, sauvegarde, Ligne};

    const EXEMPLE: &str = include_str!("../tests/donnees/exemple.ged");

    /// Contrôle de cohérence : chaque renvoi existe des deux côtés.
    fn coherent(doc: &Document) {
        let a = arbre_de(doc);
        // Renvois écrits dans les fiches des personnes.
        let mut courant = String::new();
        for l in &doc.lignes {
            let Some(x) = analyser_ligne(l) else { continue };
            if x.niveau == 0 {
                courant = x.xref.unwrap_or("").to_string();
                continue;
            }
            if let Ligne { niveau: 1, tag: tag @ ("FAMS" | "FAMC"), valeur, .. } = x {
                let f = a.famille(valeur).unwrap_or_else(|| panic!("{courant} renvoie à {valeur}, absente"));
                if tag == "FAMS" {
                    let c = Some(courant.as_str());
                    assert!(f.mari.as_deref() == c || f.epouse.as_deref() == c, "{courant} FAMS {valeur} sans HUSB/WIFE");
                } else {
                    assert!(f.enfants.contains(&courant), "{courant} FAMC {valeur} sans CHIL");
                }
            }
        }
        // Renvois écrits dans les familles.
        for f in &a.familles {
            let fiche = |id: &str, tag: &str| {
                let (d, fin) = doc.bloc(id, "INDI").unwrap_or_else(|| panic!("{} renvoie à {id}, absente", f.id));
                doc.lignes[d + 1..fin].iter().any(|l| l == &format!("1 {tag} {}", f.id))
            };
            for c in [&f.mari, &f.epouse].into_iter().flatten() {
                assert!(fiche(c, "FAMS"), "{} : {c} sans FAMS", f.id);
            }
            for c in &f.enfants {
                assert!(fiche(c, "FAMC"), "{} : {c} sans FAMC", f.id);
            }
            assert!(f.mari.is_some() || f.epouse.is_some() || !f.enfants.is_empty(), "{} vide", f.id);
        }
    }

    fn identite(prenoms: &str, nom: &str, sexe: &str) -> Identite {
        Identite { prenoms: prenoms.into(), nom: nom.into(), sexe: sexe.into(), ..Default::default() }
    }

    fn ids(v: Vec<&Personne>) -> Vec<String> {
        v.into_iter().map(|p| p.id.clone()).collect()
    }

    #[test]
    fn creation_puis_lien_de_conjoint() {
        let mut d = Document::depuis_texte(EXEMPLE);
        let id = creer_personne(&mut d, &identite("Hélène", "ROUX", "F")).unwrap();
        assert_eq!(id, "@I8@");
        // Louis (@I5@) n'a pas d'épouse : une union nouvelle est créée.
        lier(&mut d, "@I5@", Relation::Conjoint, &id, None).unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        assert_eq!(ids(a.conjoints(a.personne("@I5@").unwrap())), vec![id.clone()]);
        // L'enfant de la famille existante de Louis n'a pas reçu de mère d'office.
        assert_eq!(a.pere_mere(a.personne("@I2@").unwrap()).1.map(|p| p.id.clone()), None);
        // Un second lien identique est refusé.
        assert!(lier(&mut d, "@I5@", Relation::Conjoint, &id, None).is_err());
        // Ce qui n'est pas compris est conservé.
        assert!(d.texte().contains("1 _UID 1234"));
    }

    #[test]
    fn conjoint_dans_une_union_sans_conjoint() {
        let mut d = Document::depuis_texte(EXEMPLE);
        let id = creer_personne(&mut d, &identite("Rose", "BLANC", "F")).unwrap();
        // @F2@ : Louis et son fils Jacques, sans épouse. La compléter donne une mère à Jacques.
        let m = lier(&mut d, "@I5@", Relation::Conjoint, &id, Some("@F2@")).unwrap();
        assert!(m.contains("1 enfant"), "{m}");
        coherent(&d);
        let a = arbre_de(&d);
        assert_eq!(a.famille("@F2@").unwrap().epouse.as_deref(), Some(id.as_str()));
        assert_eq!(a.pere_mere(a.personne("@I2@").unwrap()).1.unwrap().id, id);
        // Une union qui a déjà ses deux conjoints ne se complète pas.
        let autre = creer_personne(&mut d, &identite("Léa", "NOIR", "F")).unwrap();
        assert!(lier(&mut d, "@I2@", Relation::Conjoint, &autre, Some("@F1@")).is_err());
    }

    #[test]
    fn enfant_avec_choix_de_l_union() {
        let mut d = Document::depuis_texte(EXEMPLE);
        // Paul (@I1@) et Claire (@F3@) : un enfant nouveau dans leur union.
        let e = creer_personne(&mut d, &identite("Luc", "MARTIN", "M")).unwrap();
        lier(&mut d, "@I1@", Relation::Enfant, &e, Some("@F3@")).unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        let (pere, mere) = a.pere_mere(a.personne(&e).unwrap());
        assert_eq!((pere.unwrap().id.as_str(), mere.unwrap().id.as_str()), ("@I1@", "@I6@"));
        // Déjà enfant : refus.
        assert!(lier(&mut d, "@I6@", Relation::Enfant, &e, Some("@F3@")).is_err());
        // Un enfant qui a d'autres parents n'est pas déplacé en silence.
        let err = lier(&mut d, "@I1@", Relation::Enfant, "@I4@", Some("@F3@")).unwrap_err();
        assert!(err.contains("a déjà des parents"), "{err}");
        // Union nouvelle, sans conjoint : Anne (@I4@, F) devient mère seule.
        let f = creer_personne(&mut d, &identite("Zoé", "MARTIN", "F")).unwrap();
        lier(&mut d, "@I4@", Relation::Enfant, &f, None).unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        let origine = famille_origine(&a, a.personne(&f).unwrap());
        assert_eq!(couple(origine), (None, Some("@I4@")));
    }

    #[test]
    fn enfant_qui_n_avait_qu_un_parent_rejoint_l_union() {
        let mut d = Document::depuis_texte(EXEMPLE);
        let e = creer_personne(&mut d, &identite("Max", "MARTIN", "M")).unwrap();
        definir_parents(&mut d, &e, Some("@I1@"), None).unwrap();
        coherent(&d);
        let fam_pere_seul = arbre_de(&d).personne(&e).unwrap().familles_enfant[0].clone();
        // Ajouté ensuite comme enfant de l'union de Paul et Claire : il la rejoint, et la
        // famille « père seul » qui ne sert plus disparaît.
        lier(&mut d, "@I6@", Relation::Enfant, &e, Some("@F3@")).unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        assert_eq!(a.personne(&e).unwrap().familles_enfant, vec!["@F3@".to_string()]);
        assert!(a.famille(&fam_pere_seul).is_none());
    }

    #[test]
    fn modification_des_parents() {
        let mut d = Document::depuis_texte(EXEMPLE);
        // Anne (@I4@) a pour parents Jacques et Marie (@F1@), avec son frère Paul.
        let m2 = creer_personne(&mut d, &identite("Jeanne", "COSTE", "F")).unwrap();
        definir_parents(&mut d, "@I4@", Some("@I2@"), Some(&m2)).unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        let anne = a.personne("@I4@").unwrap();
        let (pere, mere) = a.pere_mere(anne);
        assert_eq!((pere.unwrap().id.as_str(), mere.unwrap().id.as_str()), ("@I2@", m2.as_str()));
        // L'union de Jacques et Marie et l'autre enfant sont intacts.
        let f1 = a.famille("@F1@").unwrap();
        assert_eq!((f1.mari.as_deref(), f1.epouse.as_deref()), (Some("@I2@"), Some("@I3@")));
        assert_eq!(f1.enfants, vec!["@I1@".to_string()]);
        // Retour aux parents d'origine : la famille existante est reprise, l'autre disparaît.
        let fam_neuve = anne.familles_enfant[0].clone();
        definir_parents(&mut d, "@I4@", Some("@I2@"), Some("@I3@")).unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        assert_eq!(a.personne("@I4@").unwrap().familles_enfant, vec!["@F1@".to_string()]);
        // Jacques et Jeanne restent unis : la famille nouvelle, vidée de ses enfants, garde ses deux conjoints.
        assert!(a.famille(&fam_neuve).is_some());
        // Contrôles : même personne, sexe, boucle.
        assert!(definir_parents(&mut d, "@I4@", Some("@I2@"), Some("@I2@")).is_err());
        assert!(definir_parents(&mut d, "@I4@", Some("@I3@"), None).is_err(), "une femme comme père");
        assert!(definir_parents(&mut d, "@I5@", Some("@I1@"), None).is_err(), "boucle");
        assert_eq!(definir_parents(&mut d, "@I4@", Some("@I2@"), Some("@I3@")).unwrap(), "Aucun changement : ce sont déjà ses parents.");
    }

    #[test]
    fn association_d_un_parent() {
        let mut d = Document::depuis_texte(EXEMPLE);
        // Jacques (@I2@) n'a qu'un père, seul enfant de @F2@ : la mère complète cette famille.
        let m = creer_personne(&mut d, &identite("Berthe", "VIDAL", "F")).unwrap();
        lier(&mut d, "@I2@", Relation::Mere, &m, None).unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        assert_eq!(a.famille("@F2@").unwrap().epouse.as_deref(), Some(m.as_str()));
        // Un second père est refusé : il faut passer par « Modifier les parents ».
        let p = creer_personne(&mut d, &identite("Henri", "VIDAL", "M")).unwrap();
        let err = lier(&mut d, "@I2@", Relation::Pere, &p, None).unwrap_err();
        assert!(err.contains("Modifier les parents"), "{err}");
        // « Sans parents » (@I7@) reçoit un père : famille nouvelle.
        lier(&mut d, "@I7@", Relation::Parent, &p, None).unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        assert_eq!(a.pere_mere(a.personne("@I7@").unwrap()).0.unwrap().id, p);
    }

    #[test]
    fn retrait_des_liens() {
        let mut d = Document::depuis_texte(EXEMPLE);
        // Retrait de la mère de Paul : il reste l'enfant de Jacques, Anne garde ses deux parents.
        let m = retirer_lien(&mut d, "@I1@", Relation::Parent, "@I3@").unwrap();
        assert!(m.contains("Jacques MARTIN"), "{m}");
        coherent(&d);
        let a = arbre_de(&d);
        let (pere, mere) = a.pere_mere(a.personne("@I1@").unwrap());
        assert_eq!(pere.unwrap().id, "@I2@");
        assert!(mere.is_none());
        assert_eq!(a.famille("@F1@").unwrap().enfants, vec!["@I4@".to_string()]);
        // Retrait de l'enfant depuis la fiche du parent : Louis et Jacques (@F2@, seul enfant).
        retirer_lien(&mut d, "@I5@", Relation::Enfant, "@I2@").unwrap();
        coherent(&d);
        let a = arbre_de(&d);
        assert!(a.famille("@F2@").is_none(), "famille d'un parent seul sans enfant conservée");
        assert!(a.parents(a.personne("@I2@").unwrap()).is_empty());
        assert!(a.personne("@I5@").unwrap().familles_conjoint.is_empty());
        // Retrait d'un conjoint sans enfant : l'union disparaît, avec son mariage.
        let m = retirer_lien(&mut d, "@I1@", Relation::Conjoint, "@I6@").unwrap();
        assert!(m.contains("supprimée"), "{m}");
        coherent(&d);
        let a = arbre_de(&d);
        assert!(a.famille("@F3@").is_none());
        assert!(a.conjoints(a.personne("@I1@").unwrap()).is_empty());
        // Aucune personne n'a été supprimée, et ce qui n'est pas compris est conservé.
        assert_eq!(a.personnes.len(), 7);
        assert!(d.texte().contains("1 _UID 1234"));
        // Lien inexistant : refus.
        assert!(retirer_lien(&mut d, "@I1@", Relation::Conjoint, "@I6@").is_err());
        assert!(retirer_lien(&mut d, "@I7@", Relation::Parent, "@I5@").is_err());
    }

    #[test]
    fn retrait_d_un_conjoint_avec_enfants() {
        let mut d = Document::depuis_texte(EXEMPLE);
        let m = retirer_lien(&mut d, "@I2@", Relation::Conjoint, "@I3@").unwrap();
        assert!(m.contains("2 enfant(s)"), "{m}");
        coherent(&d);
        let a = arbre_de(&d);
        let f1 = a.famille("@F1@").unwrap();
        assert_eq!((f1.mari.as_deref(), f1.epouse.as_deref()), (Some("@I2@"), None));
        assert_eq!(f1.enfants.len(), 2);
        assert!(a.personne("@I3@").unwrap().familles_conjoint.is_empty());
    }

    #[test]
    fn sous_lignes_des_liens_retirees_avec_eux() {
        let texte = EXEMPLE.replace("1 CHIL @I4@\n", "1 CHIL @I4@\n2 _FREL Natural\n2 _MREL Natural\n").replace(
            "1 FAMC @F1@\n0 @I5@",
            "1 FAMC @F1@\n2 PEDI birth\n0 @I5@",
        );
        let mut d = Document::depuis_texte(&texte);
        assert!(d.texte().contains("2 PEDI birth"));
        retirer_lien(&mut d, "@I4@", Relation::Parent, "@I2@").unwrap();
        retirer_lien(&mut d, "@I4@", Relation::Parent, "@I3@").unwrap();
        coherent(&d);
        let t = d.texte();
        assert!(!t.contains("_FREL") && !t.contains("_MREL") && !t.contains("PEDI"), "{t}");
        assert!(arbre_de(&d).parents(arbre_de(&d).personne("@I4@").unwrap()).is_empty());
    }

    #[test]
    fn modification_de_l_identite() {
        let mut d = Document::depuis_texte(EXEMPLE);
        let avant = d.lignes.clone();
        let a = arbre_de(&d);
        let paul = Identite::de(a.personne("@I1@").unwrap());
        // Rien de changé : rien d'écrit.
        assert!(!modifier_identite(&mut d, "@I1@", &paul).unwrap());
        assert_eq!(d.lignes, avant);

        let mut n = paul.clone();
        n.prenoms = "Paul Henri".into();
        n.deces = "3 FEB 2020".into();
        n.lieu_deces = "Agen".into();
        n.lieu_naissance = String::new();
        assert!(modifier_identite(&mut d, "@I1@", &n).unwrap());
        let a = arbre_de(&d);
        let p = a.personne("@I1@").unwrap();
        assert_eq!(p.nom_affiche(), "Paul Henri MARTIN");
        assert_eq!(p.prenoms, "Paul Henri");
        assert_eq!((p.deces.as_str(), p.lieu_deces.as_str()), ("3 FEB 2020", "Agen"));
        assert_eq!((p.naissance.as_str(), p.lieu_naissance.as_str()), ("12 MAR 1950", ""));
        // Seules les lignes d'identité ont bougé ; liens et balises inconnues sont en place.
        let (debut, fin) = d.bloc("@I1@", "INDI").unwrap();
        let bloc = d.lignes[debut..fin].join("\n");
        assert_eq!(
            bloc,
            "0 @I1@ INDI\n1 NAME Paul Henri /MARTIN/\n2 GIVN Paul Henri\n2 SURN MARTIN\n1 SEX M\n1 BIRT\n2 DATE 12 MAR 1950\n\
             1 DEAT\n2 DATE 3 FEB 2020\n2 PLAC Agen\n1 _FSFTID LZ1A-B2C\n1 _UID 1234\n1 FAMC @F1@\n1 FAMS @F3@"
        );
        assert_eq!(d.lignes.len(), avant.len() + 3 - 1);
        coherent(&d);

        // Effacer date et lieu retire l'événement devenu vide.
        let mut m = Identite::de(a.personne("@I3@").unwrap());
        m.deces.clear();
        m.lieu_deces.clear();
        modifier_identite(&mut d, "@I3@", &m).unwrap();
        let (debut, fin) = d.bloc("@I3@", "INDI").unwrap();
        assert!(!d.lignes[debut..fin].iter().any(|l| l.contains("DEAT")));
        // Un nom vide est refusé.
        assert!(modifier_identite(&mut d, "@I3@", &identite("", "", "F")).is_err());
    }

    #[test]
    fn nom_sans_barres_et_suffixe() {
        let texte = "0 HEAD\n0 @I1@ INDI\n1 NAME Jean Dupont\n0 @I2@ INDI\n1 NAME Pierre /DURAND/ Jr\n2 NPFX Dr\n1 SEX M\n0 TRLR\n";
        let mut d = Document::depuis_texte(texte);
        let a = arbre_de(&d);
        let jean = Identite::de(a.personne("@I1@").unwrap());
        assert_eq!((jean.prenoms.as_str(), jean.nom.as_str()), ("Jean Dupont", ""));
        assert!(!modifier_identite(&mut d, "@I1@", &jean).unwrap());
        let mut n = jean.clone();
        n.prenoms = "Jean".into();
        n.nom = "DUPONT".into();
        modifier_identite(&mut d, "@I1@", &n).unwrap();
        assert!(d.texte().contains("1 NAME Jean /DUPONT/\r\n"));
        let mut p = Identite::de(a.personne("@I2@").unwrap());
        p.nom = "DURANT".into();
        modifier_identite(&mut d, "@I2@", &p).unwrap();
        assert!(d.texte().contains("1 NAME Pierre /DURANT/ Jr\r\n2 NPFX Dr\r\n"));
    }

    #[test]
    fn sauvegarde_bak_avant_ecriture() {
        let dossier = std::env::temp_dir().join(format!("genfam-edition-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        let ged = dossier.join("essai.ged");
        std::fs::write(&ged, EXEMPLE).unwrap();
        let mut d = Document::lire(&ged).unwrap();
        let id = creer_personne(&mut d, &identite("Eve", "ROCHE", "F")).unwrap();
        lier(&mut d, "@I7@", Relation::Mere, &id, None).unwrap();
        ecrire_avec_sauvegarde(&ged, &d).unwrap();
        // La copie .bak est l'état d'avant, à l'octet près ; le fichier porte le lien.
        assert_eq!(std::fs::read_to_string(sauvegarde(&ged)).unwrap(), EXEMPLE);
        let relu = Document::lire(&ged).unwrap();
        let a = arbre_de(&relu);
        assert_eq!(a.pere_mere(a.personne("@I7@").unwrap()).1.unwrap().id, id);
        std::fs::remove_dir_all(dossier).unwrap();
    }

    #[test]
    fn unions_d_une_personne() {
        let a = arbre_de(&Document::depuis_texte(EXEMPLE));
        let u = unions(&a, a.personne("@I1@").unwrap());
        assert_eq!(u, vec![Union { famille: "@F3@".into(), conjoint: Some("@I6@".into()), enfants: 0, mariage: "5 JUN 1975".into() }]);
        let u = unions(&a, a.personne("@I5@").unwrap());
        assert_eq!((u[0].conjoint.clone(), u[0].enfants), (None, 1));
    }
}
