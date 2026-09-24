//! Mise en page du pedigree : cartes, traits et libellés de génération.
//!
//! Le calcul est celui de show_pedigree() de la version Python : ascendants au-dessus,
//! personne centrale et premier conjoint au milieu, descendants au-dessous, chaque
//! génération centrée. L'interface ne fait que dessiner ce qui est rendu ici.

use crate::modele::{Arbre, Personne};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

fn niveaux<'a>(arbre: &'a Arbre, p: &'a Personne, n: usize, montant: bool) -> Vec<Vec<&'a Personne>> {
    let mut r = vec![vec![p]];
    for niveau in 1..=n {
        let mut suivant = Vec::new();
        let mut vus = HashSet::new();
        for q in &r[niveau - 1] {
            let proches = if montant { arbre.parents(q) } else { arbre.enfants(q) };
            for x in proches {
                if vus.insert(x.id.clone()) {
                    suivant.push(x);
                }
            }
        }
        r.push(suivant);
    }
    r
}

/// Rend la mise en page en JSON : `largeur`, `hauteur`, `cartes`, `traits`,
/// `etiquettes`, et `centre` (point sur lequel recentrer la vue).
pub fn mise_en_page(arbre: &Arbre, id: &str, haut: usize, bas: usize, zoom: f64) -> Value {
    let Some(personne) = arbre.personne(id) else { return json!({}) };
    let haut = haut.min(20);
    let bas = bas.min(20);
    let s = zoom.clamp(0.3, 2.0);
    let asc = niveaux(arbre, personne, haut, true);
    let desc = niveaux(arbre, personne, bas, false);
    let conjoints = arbre.conjoints(personne);
    let conjoint = conjoints.first().copied();
    let ids_conjoints: HashSet<&str> = conjoints.iter().map(|c| c.id.as_str()).collect();

    let (w, h, gx, gy) = (260.0 * s, 205.0 * s, 80.0 * s, 95.0 * s);
    let marge = 90.0 * s;
    let max_nb = asc.iter().chain(desc.iter()).map(Vec::len).chain([if conjoint.is_some() { 2 } else { 1 }]).max().unwrap_or(1) as f64;
    let largeur = (1900.0 * s).max(max_nb * w + (max_nb - 1.0).max(0.0) * gx + 380.0 * s);
    let hauteur = (1200.0 * s).max((haut + bas + 1) as f64 * h + (haut + bas + 2) as f64 * gy + 150.0 * s);
    let cx = largeur / 2.0;
    let y_racine = marge + haut as f64 * (h + gy);

    let mut cartes = Vec::new();
    let mut centres: HashMap<String, (f64, f64)> = HashMap::new();
    let mut placer = |gens: &[&Personne], y: f64, categorie: &str, cartes: &mut Vec<Value>| {
        if gens.is_empty() {
            return;
        }
        let n = gens.len() as f64;
        let x0 = cx - (n * w + (n - 1.0) * gx) / 2.0;
        for (i, q) in gens.iter().enumerate() {
            let x = x0 + i as f64 * (w + gx);
            let cat = if q.id == personne.id {
                "centrale"
            } else if ids_conjoints.contains(q.id.as_str()) {
                "conjoint"
            } else {
                categorie
            };
            cartes.push(json!({
                "id": q.id, "x": x, "y": y, "categorie": cat,
                "nom": q.nom_affiche(),
                "naissance": q.naissance, "lieuNaissance": q.lieu_naissance,
                "mariage": q.mariage, "lieuMariage": q.lieu_mariage,
                "deces": q.deces, "lieuDeces": q.lieu_deces,
            }));
            centres.insert(q.id.clone(), (x + w / 2.0, y));
        }
    };
    for niveau in (1..=haut).rev() {
        placer(&asc[niveau], marge + (haut - niveau) as f64 * (h + gy), "ascendant", &mut cartes);
    }
    let mut milieu = vec![personne];
    if let Some(c) = conjoint {
        milieu.push(c);
    }
    placer(&milieu, y_racine, "racine", &mut cartes);
    for niveau in 1..=bas {
        placer(&desc[niveau], y_racine + niveau as f64 * (h + gy), "descendant", &mut cartes);
    }

    let (lw, plw) = ((2.0 * s).max(2.0), (2.5 * s).max(2.0));
    let mut traits = Vec::new();
    let relier = |haut_: (f64, f64), bas_: (f64, f64), traits: &mut Vec<Value>| {
        let (px, py) = haut_;
        let (ex, ey) = bas_;
        let y_branche = py + h + gy / 2.0;
        traits.push(json!([px, py + h, px, y_branche, plw, "#566873"]));
        traits.push(json!([px, y_branche, ex, y_branche, lw, "#566873"]));
        traits.push(json!([ex, y_branche, ex, ey, lw, "#566873"]));
    };
    for niveau in 0..bas {
        let suivants: HashSet<&str> = desc[niveau + 1].iter().map(|q| q.id.as_str()).collect();
        for parent in &desc[niveau] {
            let Some(&pc) = centres.get(&parent.id) else { continue };
            for enfant in arbre.enfants(parent) {
                if let (true, Some(&ec)) = (suivants.contains(enfant.id.as_str()), centres.get(&enfant.id)) {
                    relier(pc, ec, &mut traits);
                }
            }
        }
    }
    for niveau in 0..haut {
        let suivants: HashSet<&str> = asc[niveau + 1].iter().map(|q| q.id.as_str()).collect();
        for enfant in &asc[niveau] {
            let Some(&ec) = centres.get(&enfant.id) else { continue };
            for parent in arbre.parents(enfant) {
                if let (true, Some(&pc)) = (suivants.contains(parent.id.as_str()), centres.get(&parent.id)) {
                    relier(pc, ec, &mut traits);
                }
            }
        }
    }
    if let Some(c) = conjoint {
        if let (Some(&(px, py)), Some(&(sx, sy))) = (centres.get(&personne.id), centres.get(&c.id)) {
            traits.push(json!([px + w / 2.0, py + h / 2.0, sx - w / 2.0, sy + h / 2.0, lw, "#8B7650"]));
        }
    }

    let mut etiquettes = Vec::new();
    for niveau in (1..=haut).rev() {
        if !asc[niveau].is_empty() {
            let nom = match niveau { 1 => "Parents", 2 => "Grands-parents", _ => "Ascendants" };
            etiquettes.push(json!({"x": 20.0 * s, "y": marge + (haut - niveau) as f64 * (h + gy) + h / 2.0,
                                   "texte": format!("GÉNÉRATION −{niveau}\n{nom}")}));
        }
    }
    etiquettes.push(json!({"x": 20.0 * s, "y": y_racine + h / 2.0, "texte": "GÉNÉRATION 0\nPersonne centrale"}));
    for niveau in 1..=bas {
        if !desc[niveau].is_empty() {
            let nom = match niveau { 1 => "Enfants", 2 => "Petits-enfants", 3 => "Arrière-petits-enfants", _ => "Descendants" };
            etiquettes.push(json!({"x": 20.0 * s, "y": y_racine + niveau as f64 * (h + gy) + h / 2.0,
                                   "texte": format!("GÉNÉRATION +{niveau}\n{nom}")}));
        }
    }

    json!({
        "largeur": largeur, "hauteur": hauteur, "carteL": w, "carteH": h, "echelle": s,
        "centre": {"x": cx, "y": y_racine + h / 2.0},
        "nom": personne.nom_affiche(),
        "cartes": cartes, "traits": traits, "etiquettes": etiquettes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gedcom::analyser;

    #[test]
    fn pedigree_exemple() {
        let a = analyser(include_str!("../tests/donnees/exemple.ged"));
        let v = mise_en_page(&a, "@I1@", 3, 3, 1.0);
        let cartes = v["cartes"].as_array().unwrap();
        // Louis ; Jacques et Marie ; Paul et Claire.
        assert_eq!(cartes.len(), 5);
        assert!(cartes.iter().any(|c| c["id"] == "@I6@" && c["categorie"] == "conjoint"));
        assert!(cartes.iter().any(|c| c["id"] == "@I1@" && c["categorie"] == "centrale"));
        // Deux liens parent → enfant (3 traits chacun) + Louis → Jacques, et l'union.
        assert_eq!(v["traits"].as_array().unwrap().len(), 3 * 3 + 1);
    }
}
