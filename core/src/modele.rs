//! Modèle généalogique : personnes, familles, et les requêtes qui les relient.
//!
//! Portage de GENFAMILLE.pyw (Claude Boisseau, V7.6) : les règles de parenté,
//! d'ascendance et de numérotation Ahnentafel sont reprises à l'identique.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Personne {
    pub id: String,
    /// Valeur brute de `1 NAME`, barres GEDCOM comprises (« Jean /DUPONT/ »).
    pub nom_gedcom: String,
    pub prenoms: String,
    pub nom: String,
    pub sexe: String,
    pub naissance: String,
    pub lieu_naissance: String,
    pub mariage: String,
    pub lieu_mariage: String,
    pub deces: String,
    pub lieu_deces: String,
    /// Identifiant de l'arbre FamilySearch (`1 _FSFTID`), posé par Ancestral Quest
    /// ou par l'import FamilySearch de ce programme.
    pub fsid: String,
    pub familles_enfant: Vec<String>,
    pub familles_conjoint: Vec<String>,
    pub notes: Vec<String>,
}

impl Personne {
    pub fn nouvelle(id: &str) -> Self {
        Self { id: id.to_string(), ..Default::default() }
    }

    /// Prénoms, puis nom : ce que l'on lit sur une fiche. Les barres GEDCOM sont ôtées.
    pub fn nom_affiche(&self) -> String {
        if !self.nom_gedcom.trim().is_empty() {
            let brut = self.nom_gedcom.replace('/', " ");
            return brut.split_whitespace().collect::<Vec<_>>().join(" ");
        }
        let assemble = format!("{} {}", self.prenoms, self.nom).trim().to_string();
        if assemble.is_empty() { self.id.clone() } else { assemble }
    }

    /// NOM puis prénoms : l'ordre des listes.
    pub fn nom_liste(&self) -> String {
        let brut = self.nom_gedcom.trim();
        if let Some((avant, apres)) = brut.split_once('/') {
            let nom = apres.split('/').next().unwrap_or("").trim();
            return format!("{} {}", nom, avant.trim()).trim().to_string();
        }
        if !self.nom.trim().is_empty() {
            return format!("{} {}", self.nom.trim(), self.prenoms.trim()).trim().to_string();
        }
        self.nom_affiche()
    }

    /// Nom de famille : `SURN`, à défaut la partie entre barres du `NAME`.
    pub fn nom_famille(&self) -> String {
        if !self.nom.trim().is_empty() {
            return self.nom.trim().to_string();
        }
        match self.nom_gedcom.trim().split_once('/') {
            Some((_, apres)) => apres.split('/').next().unwrap_or("").trim().to_string(),
            None => String::new(),
        }
    }

    /// Prénoms : `GIVN`, à défaut la partie qui précède les barres du `NAME`.
    pub fn prenoms_affiches(&self) -> String {
        if !self.prenoms.trim().is_empty() {
            return self.prenoms.trim().to_string();
        }
        match self.nom_gedcom.trim().split_once('/') {
            Some((avant, _)) => avant.trim().to_string(),
            None => String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Famille {
    pub id: String,
    pub mari: Option<String>,
    pub epouse: Option<String>,
    pub enfants: Vec<String>,
    pub mariage: String,
    pub lieu_mariage: String,
}

/// Mode de tri de la liste des personnes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    Alpha,
    Numerotation,
    Ahnentafel,
}

impl Tri {
    pub fn depuis(texte: &str) -> Self {
        match texte {
            "Numérotation" | "Numerotation" => Tri::Numerotation,
            "Ahnentafel" => Tri::Ahnentafel,
            _ => Tri::Alpha,
        }
    }
}

/// Arbre chargé d'un fichier GEDCOM. Les personnes gardent l'ordre du fichier :
/// la première devient le N° 1 Ahnentafel par défaut, comme dans la version Python.
#[derive(Debug, Clone, Default)]
pub struct Arbre {
    pub personnes: Vec<Personne>,
    pub familles: Vec<Famille>,
    index_p: HashMap<String, usize>,
    index_f: HashMap<String, usize>,
}

impl Arbre {
    pub fn construire(personnes: Vec<Personne>, familles: Vec<Famille>) -> Self {
        let mut a = Arbre { personnes, familles, ..Default::default() };
        a.index_p = a.personnes.iter().enumerate().map(|(i, p)| (p.id.clone(), i)).collect();
        a.index_f = a.familles.iter().enumerate().map(|(i, f)| (f.id.clone(), i)).collect();
        a.relier();
        a
    }

    /// Complète les renvois FAMS / FAMC à partir des familles, et reporte la date
    /// de mariage sur les conjoints — ce que fait parse_gedcom() en fin de lecture.
    fn relier(&mut self) {
        let familles = self.familles.clone();
        for f in &familles {
            for pid in [&f.mari, &f.epouse].into_iter().flatten() {
                if let Some(&i) = self.index_p.get(pid) {
                    let p = &mut self.personnes[i];
                    if !p.familles_conjoint.contains(&f.id) {
                        p.familles_conjoint.push(f.id.clone());
                    }
                    if !f.mariage.is_empty() && p.mariage.is_empty() {
                        p.mariage = f.mariage.clone();
                    }
                    if !f.lieu_mariage.is_empty() && p.lieu_mariage.is_empty() {
                        p.lieu_mariage = f.lieu_mariage.clone();
                    }
                }
            }
            for cid in &f.enfants {
                if let Some(&i) = self.index_p.get(cid) {
                    let p = &mut self.personnes[i];
                    if !p.familles_enfant.contains(&f.id) {
                        p.familles_enfant.push(f.id.clone());
                    }
                }
            }
        }
    }

    pub fn personne(&self, id: &str) -> Option<&Personne> {
        self.index_p.get(id).map(|&i| &self.personnes[i])
    }

    pub fn famille(&self, id: &str) -> Option<&Famille> {
        self.index_f.get(id).map(|&i| &self.familles[i])
    }

    pub fn par_fsid(&self, fsid: &str) -> Option<&Personne> {
        self.personnes.iter().find(|p| !p.fsid.is_empty() && p.fsid.eq_ignore_ascii_case(fsid))
    }

    pub fn parents(&self, p: &Personne) -> Vec<&Personne> {
        let mut r = Vec::new();
        for fid in &p.familles_enfant {
            if let Some(f) = self.famille(fid) {
                if let Some(x) = f.mari.as_deref().and_then(|id| self.personne(id)) {
                    r.push(x);
                }
                if let Some(x) = f.epouse.as_deref().and_then(|id| self.personne(id)) {
                    r.push(x);
                }
            }
        }
        r
    }

    pub fn conjoints(&self, p: &Personne) -> Vec<&Personne> {
        let mut r = Vec::new();
        for fid in &p.familles_conjoint {
            if let Some(f) = self.famille(fid) {
                let autre = if f.mari.as_deref() == Some(p.id.as_str()) { &f.epouse } else { &f.mari };
                if let Some(x) = autre.as_deref().and_then(|id| self.personne(id)) {
                    r.push(x);
                }
            }
        }
        r
    }

    pub fn enfants(&self, p: &Personne) -> Vec<&Personne> {
        let mut r = Vec::new();
        for fid in &p.familles_conjoint {
            if let Some(f) = self.famille(fid) {
                for cid in &f.enfants {
                    if let Some(x) = self.personne(cid) {
                        r.push(x);
                    }
                }
            }
        }
        r
    }

    pub fn fratrie(&self, p: &Personne) -> Vec<&Personne> {
        let mut r = Vec::new();
        for fid in &p.familles_enfant {
            if let Some(f) = self.famille(fid) {
                for cid in &f.enfants {
                    if cid != &p.id {
                        if let Some(x) = self.personne(cid) {
                            r.push(x);
                        }
                    }
                }
            }
        }
        r
    }

    /// Tous les ascendants, avec leur génération (1 = parents), en profondeur d'abord.
    pub fn ascendants(&self, p: &Personne) -> Vec<(usize, &Personne)> {
        let mut r = Vec::new();
        let mut vus = HashSet::new();
        self.ascendants_rec(p, 1, &mut vus, &mut r);
        r
    }

    fn ascendants_rec<'a>(&'a self, p: &Personne, g: usize, vus: &mut HashSet<String>, r: &mut Vec<(usize, &'a Personne)>) {
        for parent in self.parents(p) {
            if !vus.insert(parent.id.clone()) {
                continue;
            }
            r.push((g, parent));
            self.ascendants_rec(parent, g + 1, vus, r);
        }
    }

    /// Tous les descendants, avec leur génération (1 = enfants), en profondeur d'abord.
    pub fn descendants(&self, p: &Personne) -> Vec<(usize, &Personne)> {
        let mut r = Vec::new();
        let mut vus = HashSet::new();
        self.descendants_rec(p, 1, &mut vus, &mut r);
        r
    }

    fn descendants_rec<'a>(&'a self, p: &Personne, g: usize, vus: &mut HashSet<String>, r: &mut Vec<(usize, &'a Personne)>) {
        for enfant in self.enfants(p) {
            if !vus.insert(enfant.id.clone()) {
                continue;
            }
            r.push((g, enfant));
            self.descendants_rec(enfant, g + 1, vus, r);
        }
    }

    /// Père et mère d'une personne : HUSB / WIFE de sa première famille d'origine,
    /// puis le sexe, puis l'ordre du fichier — les trois recours de la version Python.
    pub fn pere_mere(&self, p: &Personne) -> (Option<&Personne>, Option<&Personne>) {
        let parents = self.parents(p);
        let mut pere = None;
        let mut mere = None;
        for fid in &p.familles_enfant {
            if let Some(f) = self.famille(fid) {
                if let Some(x) = f.mari.as_deref().and_then(|id| self.personne(id)) {
                    pere = Some(x);
                }
                if let Some(x) = f.epouse.as_deref().and_then(|id| self.personne(id)) {
                    mere = Some(x);
                }
                if pere.is_some() || mere.is_some() {
                    break;
                }
            }
        }
        if pere.is_none() {
            pere = parents.iter().copied().find(|x| x.sexe == "M");
        }
        if mere.is_none() {
            mere = parents.iter().copied().find(|x| x.sexe == "F");
        }
        if pere.is_none() {
            pere = parents.first().copied();
        }
        if mere.is_none() {
            mere = parents.iter().copied().find(|x| pere.map_or(true, |pp| pp.id != x.id));
        }
        (pere, mere)
    }

    /// Numérotation Ahnentafel (Sosa-Stradonitz) depuis `racine` : 1, père 2, mère 3…
    /// Un ancêtre reçoit plusieurs numéros en cas d'implexe ; une boucle est coupée.
    pub fn ahnentafel(&self, racine: &str) -> HashMap<String, Vec<u64>> {
        let mut numeros: HashMap<String, Vec<u64>> = HashMap::new();
        if let Some(r) = self.personne(racine) {
            let mut chemin = HashSet::new();
            self.ahnentafel_rec(r, 1, &mut chemin, &mut numeros);
        }
        for v in numeros.values_mut() {
            v.sort_unstable();
            v.dedup();
        }
        numeros
    }

    fn ahnentafel_rec(&self, p: &Personne, n: u64, chemin: &mut HashSet<String>, num: &mut HashMap<String, Vec<u64>>) {
        if chemin.contains(&p.id) {
            return;
        }
        num.entry(p.id.clone()).or_default().push(n);
        // Au-delà de 62 générations, le numéro ne tient plus sur 64 bits.
        if n >= (1u64 << 62) {
            return;
        }
        chemin.insert(p.id.clone());
        let (pere, mere) = self.pere_mere(p);
        if let Some(x) = pere {
            self.ahnentafel_rec(x, n * 2, chemin, num);
        }
        if let Some(x) = mere {
            self.ahnentafel_rec(x, n * 2 + 1, chemin, num);
        }
        chemin.remove(&p.id);
    }

    /// Recherche (nom, identifiant, lieux de naissance et de décès) et tri.
    pub fn rechercher(&self, requete: &str, tri: Tri, numeros: &HashMap<String, Vec<u64>>) -> Vec<&Personne> {
        let q = requete.trim().to_lowercase();
        let mut r: Vec<&Personne> = self
            .personnes
            .iter()
            .filter(|p| {
                q.is_empty()
                    || p.nom_affiche().to_lowercase().contains(&q)
                    || p.nom_gedcom.to_lowercase().contains(&q)
                    || p.id.to_lowercase().contains(&q)
                    || p.lieu_naissance.to_lowercase().contains(&q)
                    || p.lieu_deces.to_lowercase().contains(&q)
            })
            .collect();
        match tri {
            Tri::Alpha => r.sort_by_key(|p| p.nom_liste().to_lowercase()),
            Tri::Numerotation => r.sort_by_key(|p| cle_numerique(&p.id)),
            Tri::Ahnentafel => r.sort_by_key(|p| {
                let n = numeros.get(&p.id).and_then(|v| v.first()).copied().unwrap_or(u64::MAX);
                (n, p.nom_liste().to_lowercase())
            }),
        }
        r
    }
}

/// Clé de tri d'un identifiant GEDCOM : son nombre final, puis le texte.
fn cle_numerique(id: &str) -> (u64, String) {
    let coeur = id.trim_matches('@');
    let debut = coeur.len() - coeur.chars().rev().take_while(|c| c.is_ascii_digit()).count();
    (coeur[debut..].parse().unwrap_or(u64::MAX), id.to_lowercase())
}

/// Numéros Ahnentafel d'une personne, tels qu'affichés : « 4 / 12 ».
pub fn libelle_numeros(numeros: &HashMap<String, Vec<u64>>, id: &str) -> String {
    numeros
        .get(id)
        .map(|v| v.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" / "))
        .unwrap_or_default()
}
