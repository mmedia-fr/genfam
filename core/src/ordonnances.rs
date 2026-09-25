// SPDX-License-Identifier: GPL-3.0-or-later
//! Ordonnances : données locales, hors GEDCOM.
//!
//! La version Python les gardait en mémoire seulement — elles étaient perdues à la
//! fermeture. Elles sont désormais enregistrées à côté du GEDCOM, dans
//! `<fichier>.ordonnances.json`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const TYPES: [&str; 5] = [
    "Baptême",
    "Confirmation",
    "Dotation",
    "Scellement au conjoint",
    "Scellement aux parents",
];

pub const STATUTS: [&str; 4] = ["À vérifier", "À faire", "Réservée", "Accomplie"];

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
