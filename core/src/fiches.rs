// SPDX-License-Identifier: GPL-3.0-or-later
//! Présentations d'une personne : fiche complète, famille, ascendants, descendants.
//!
//! Chacune se rend deux fois : en HTML pour le panneau d'information (sous-ensemble
//! HTML 4 que Qt sait afficher, liens cliquables vers les proches), et en texte brut
//! pour l'export `.txt`, en colonnes comme dans la version Python.

use crate::modele::{Arbre, Personne};
use crate::ordonnances::Ordonnance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vue {
    Fiche,
    Famille,
    Ascendants,
    Descendants,
}

impl Vue {
    pub fn depuis(t: &str) -> Self {
        match t {
            "famille" => Vue::Famille,
            "ascendants" => Vue::Ascendants,
            "descendants" => Vue::Descendants,
            _ => Vue::Fiche,
        }
    }
}

pub fn echapper(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn ou(v: &str, defaut: &str) -> String {
    if v.trim().is_empty() { defaut.to_string() } else { v.to_string() }
}

/// Coupe ou complète un texte à `n` caractères (colonnes de l'export texte).
fn colonne(t: &str, n: usize) -> String {
    let nb = t.chars().count();
    if nb >= n { format!("{t} ") } else { format!("{t}{}", " ".repeat(n - nb)) }
}

const STYLE: &str = "<style>\
 body{font-family:'Segoe UI',sans-serif;color:#202020;}\
 h1{color:#173A54;font-size:18pt;margin:0;}\
 .sous{color:#5A6872;}\
 h2{color:#234E70;background:#EEF3F7;font-size:11pt;margin-top:10px;margin-bottom:4px;padding:3px;}\
 td.l{color:#5A6872;font-weight:bold;padding-right:14px;}\
 a{color:#1769AA;}\
 .vide{color:#777777;font-style:italic;}\
 th{background:#E9EEF2;text-align:left;padding:4px 10px 4px 4px;}\
 td{padding:3px 10px 3px 4px;}\
</style>";

pub fn rendre(arbre: &Arbre, id: &str, vue: Vue, ordonnances: &[Ordonnance]) -> (String, String) {
    let Some(p) = arbre.personne(id) else { return (String::new(), String::new()) };
    match vue {
        Vue::Fiche => fiche(arbre, p, ordonnances),
        Vue::Famille => famille(arbre, p),
        Vue::Ascendants => generations(&format!("Ascendants de {}", p.nom_affiche()), &arbre.ascendants(p), "Aucun ascendant renseigné."),
        Vue::Descendants => generations(&format!("Descendants de {}", p.nom_affiche()), &arbre.descendants(p), "Aucun descendant renseigné."),
    }
}

fn lien(p: &Personne) -> String {
    format!("<a href=\"{}\">{}</a> <span class=\"sous\">[{}]</span>", echapper(&p.id), echapper(&p.nom_affiche()), echapper(&p.id))
}

fn fiche(arbre: &Arbre, p: &Personne, ordonnances: &[Ordonnance]) -> (String, String) {
    let mut h = String::from(STYLE);
    let mut t = String::new();
    let dates: Vec<&str> = [p.naissance.as_str(), p.deces.as_str()].into_iter().filter(|d| !d.is_empty()).collect();
    let ligne_dates = if dates.is_empty() { "Dates non renseignées".to_string() } else { dates.join(" — ") };
    h += &format!("<h1>{}</h1><p class=\"sous\">{}</p>", echapper(&p.nom_affiche()), echapper(&ligne_dates));
    t += &format!("{}\n{}\n\n", p.nom_affiche(), ligne_dates);

    let section = |titre: &str, champs: &[(&str, String)], h: &mut String, t: &mut String| {
        *h += &format!("<h2>{titre}</h2><table>");
        *t += &format!("{titre}\n");
        for (l, v) in champs {
            *h += &format!("<tr><td class=\"l\">{l}</td><td>{}</td></tr>", echapper(v));
            *t += &format!("{}{v}\n", colonne(l, 18));
        }
        *h += "</table>";
        *t += "\n";
    };
    let mut infos = vec![
        ("Identifiant", p.id.clone()),
        ("Nom", ou(&p.nom_famille(), "Non renseigné")),
        ("Prénoms", ou(&p.prenoms_affiches(), "Non renseignés")),
        ("Sexe", ou(&p.sexe, "Non renseigné")),
    ];
    if !p.fsid.is_empty() {
        infos.push(("FamilySearch", p.fsid.clone()));
    }
    section("INFORMATIONS PERSONNELLES", &infos, &mut h, &mut t);
    section(
        "ÉVÉNEMENTS",
        &[
            ("Naissance", ou(&p.naissance, "Non renseignée")),
            ("Lieu de naissance", ou(&p.lieu_naissance, "Non renseigné")),
            ("Mariage", ou(&p.mariage, "Non renseigné")),
            ("Lieu de mariage", ou(&p.lieu_mariage, "Non renseigné")),
            ("Décès", ou(&p.deces, "Non renseigné")),
            ("Lieu de décès", ou(&p.lieu_deces, "Non renseigné")),
        ],
        &mut h,
        &mut t,
    );
    for (titre, gens, vide) in [
        ("PARENTS", arbre.parents(p), "Parents non renseignés"),
        ("CONJOINT(S)", arbre.conjoints(p), "Conjoint non renseigné"),
        ("ENFANTS", arbre.enfants(p), "Aucun enfant renseigné"),
        ("FRATRIE", arbre.fratrie(p), "Fratrie non renseignée"),
    ] {
        h += &format!("<h2>{titre}</h2>");
        t += &format!("{titre}\n");
        if gens.is_empty() {
            h += &format!("<p class=\"vide\">{vide}</p>");
            t += &format!("  {vide}\n");
        } else {
            for g in gens {
                h += &format!("<p>&nbsp;&nbsp;{}</p>", lien(g));
                t += &format!("  {} [{}]\n", g.nom_affiche(), g.id);
            }
        }
        t += "\n";
    }
    h += "<h2>ORDONNANCES</h2>";
    t += "ORDONNANCES\n";
    if ordonnances.is_empty() {
        h += "<p class=\"vide\">Aucune ordonnance enregistrée</p>";
        t += "  Aucune ordonnance enregistrée\n";
    } else {
        for o in ordonnances {
            let mut d = vec![o.statut.clone()];
            d.extend([o.date.clone(), o.temple.clone()].into_iter().filter(|x| !x.is_empty()));
            let l = format!("{} — {}", o.genre, d.join(" — "));
            h += &format!("<p>&nbsp;&nbsp;{}</p>", echapper(&l));
            t += &format!("  {l}\n");
        }
    }
    if !p.notes.is_empty() {
        h += "<h2>NOTES</h2>";
        t += "\nNOTES\n";
        for n in &p.notes {
            h += &format!("<p>{}</p>", echapper(n).replace('\n', "<br>"));
            t += &format!("  {n}\n");
        }
    }
    (h, t)
}

fn famille(arbre: &Arbre, p: &Personne) -> (String, String) {
    let mut h = format!("{STYLE}<h1>Famille de {}</h1><p class=\"sous\">Présentation des relations familiales</p>", echapper(&p.nom_affiche()));
    let mut t = format!("Famille de {}\nPrésentation des relations familiales\n\n", p.nom_affiche());
    for (titre, relation, gens) in [
        ("PARENTS", "Parent", arbre.parents(p)),
        ("CONJOINT(S)", "Conjoint", arbre.conjoints(p)),
        ("ENFANTS", "Enfant", arbre.enfants(p)),
        ("FRATRIE", "Frère / sœur", arbre.fratrie(p)),
    ] {
        h += &format!("<h2>{titre}</h2>");
        t += &format!("{titre}\n{}{}ID\n", colonne("Relation", 18), colonne("Nom", 42));
        if gens.is_empty() {
            h += "<p class=\"vide\">Aucune information renseignée.</p>";
            t += "Aucune information renseignée.\n\n";
            continue;
        }
        h += "<table width=\"100%\" cellspacing=\"0\"><tr><th>Relation</th><th>Nom</th><th>ID</th></tr>";
        for (i, g) in gens.iter().enumerate() {
            let fond = if i % 2 == 0 { "#FFFFFF" } else { "#F7F9FA" };
            h += &format!(
                "<tr bgcolor=\"{fond}\"><td>{relation}</td><td><a href=\"{}\">{}</a></td><td>{}</td></tr>",
                echapper(&g.id), echapper(&g.nom_affiche()), echapper(&g.id)
            );
            t += &format!("{}{}{}\n", colonne(relation, 18), colonne(&g.nom_affiche(), 42), g.id);
        }
        h += "</table>";
        t += "\n";
    }
    (h, t)
}

fn generations(titre: &str, liste: &[(usize, &Personne)], vide: &str) -> (String, String) {
    let mut h = format!("{STYLE}<h1>{}</h1>", echapper(titre));
    let mut t = format!("{titre}\n{}{}ID\n", colonne("Génération", 14), colonne("Nom", 42));
    if liste.is_empty() {
        h += &format!("<p class=\"vide\">{vide}</p>");
        t += &format!("{vide}\n");
        return (h, t);
    }
    h += "<table width=\"100%\" cellspacing=\"0\"><tr><th>Génération</th><th>Nom</th><th>ID</th></tr>";
    for (i, (g, p)) in liste.iter().enumerate() {
        let fond = if i % 2 == 0 { "#FFFFFF" } else { "#F7F9FA" };
        h += &format!(
            "<tr bgcolor=\"{fond}\"><td>{g}</td><td><a href=\"{}\">{}</a></td><td>{}</td></tr>",
            echapper(&p.id), echapper(&p.nom_affiche()), echapper(&p.id)
        );
        t += &format!("{}{}{}\n", colonne(&g.to_string(), 14), colonne(&p.nom_affiche(), 42), p.id);
    }
    h += "</table>";
    (h, t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gedcom::analyser;

    #[test]
    fn fiche_et_export() {
        let a = analyser(include_str!("../tests/donnees/exemple.ged"));
        let (h, t) = rendre(&a, "@I1@", Vue::Fiche, &[]);
        assert!(h.contains("<a href=\"@I2@\">Jacques MARTIN</a>"));
        assert!(t.contains("Prénoms           Paul"));
        assert!(t.contains("FamilySearch      LZ1A-B2C"));
        let (_, t) = rendre(&a, "@I1@", Vue::Ascendants, &[]);
        assert!(t.lines().nth(2).unwrap().starts_with("1             Jacques MARTIN"));
        let (h, _) = rendre(&a, "@I5@", Vue::Famille, &[]);
        assert!(h.contains("Enfant"));
    }
}
