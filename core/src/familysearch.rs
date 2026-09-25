// SPDX-License-Identifier: GPL-3.0-or-later
//! Liaison FamilySearch : connexion OAuth 2.0 (Authorization Code + PKCE) et import
//! de l'ascendance d'une personne dans le GEDCOM ouvert.
//!
//! La connexion reprend le schéma de la version Python : le navigateur ouvre la page
//! FamilySearch, qui renvoie le code sur un petit serveur local. Seule différence,
//! **le port est fixe** : FamilySearch n'accepte que l'URI de rappel déclarée pour
//! l'App Key, et la version Python essayait cinquante ports successifs.

use crate::gedcom::Document;
use crate::modele::{Arbre, Personne};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

/// Port du serveur de rappel. L'URI à déclarer chez FamilySearch est `URI_RAPPEL`.
pub const PORT_RAPPEL: u16 = 57938;
pub const URI_RAPPEL: &str = "http://127.0.0.1:57938/familysearch-auth";
/// FamilySearch plafonne l'ascendance à 8 générations par requête.
pub const GENERATIONS_MAX: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environnement {
    /// Données de test, accessible à toute App Key dès sa création.
    Integration,
    /// Copie ancienne de la production, sur demande.
    Beta,
    /// Données réelles, après la revue de compatibilité de FamilySearch.
    Production,
}

impl Environnement {
    pub fn depuis(t: &str) -> Self {
        match t {
            "production" => Environnement::Production,
            "beta" => Environnement::Beta,
            _ => Environnement::Integration,
        }
    }

    fn ident(self) -> &'static str {
        match self {
            Environnement::Integration => "https://identint.familysearch.org",
            Environnement::Beta => "https://identbeta.familysearch.org",
            Environnement::Production => "https://ident.familysearch.org",
        }
    }

    fn api(self) -> &'static str {
        match self {
            Environnement::Integration => "https://api-integ.familysearch.org",
            Environnement::Beta => "https://apibeta.familysearch.org",
            Environnement::Production => "https://api.familysearch.org",
        }
    }

    pub fn libelle(self) -> &'static str {
        match self {
            Environnement::Integration => "intégration (données de test)",
            Environnement::Beta => "bêta",
            Environnement::Production => "production",
        }
    }
}

/// Encodage « pourcent » d'un composant d'URL (RFC 3986, caractères non réservés).
pub fn encoder(s: &str) -> String {
    let mut r = String::with_capacity(s.len());
    for o in s.bytes() {
        if o.is_ascii_alphanumeric() || b"-._~".contains(&o) {
            r.push(o as char);
        } else {
            r.push_str(&format!("%{o:02X}"));
        }
    }
    r
}

fn decoder(s: &str) -> String {
    fn hex(c: u8) -> Option<u8> {
        (c as char).to_digit(16).map(|d| d as u8)
    }
    let o = s.as_bytes();
    let mut r = Vec::with_capacity(o.len());
    let mut i = 0;
    while i < o.len() {
        match o[i] {
            b'%' if i + 2 < o.len() => match (hex(o[i + 1]), hex(o[i + 2])) {
                (Some(a), Some(b)) => {
                    r.push(a * 16 + b);
                    i += 3;
                }
                _ => {
                    r.push(b'%');
                    i += 1;
                }
            },
            b'+' => {
                r.push(b' ');
                i += 1;
            }
            c => {
                r.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&r).into_owned()
}

fn aleatoire(n: usize) -> String {
    let mut o = vec![0u8; n];
    getrandom::getrandom(&mut o).expect("générateur aléatoire du système indisponible");
    URL_SAFE_NO_PAD.encode(o)
}

/// Paramètres d'une connexion en cours.
pub struct Connexion {
    pub env: Environnement,
    pub cle: String,
    pub verificateur: String,
    pub etat: String,
    pub url: String,
}

impl Connexion {
    pub fn preparer(env: Environnement, cle: &str) -> Self {
        let verificateur = aleatoire(64);
        let defi = URL_SAFE_NO_PAD.encode(Sha256::digest(verificateur.as_bytes()));
        let etat = aleatoire(24);
        let url = format!(
            "{}/cis-web/oauth2/v3/authorization?response_type=code&client_id={}&redirect_uri={}&state={}&scope=openid&code_challenge={}&code_challenge_method=S256",
            env.ident(), encoder(cle), encoder(URI_RAPPEL), encoder(&etat), encoder(&defi)
        );
        Self { env, cle: cle.to_string(), verificateur, etat, url }
    }
}

/// Ouvre le port de rappel. À faire **avant** d'ouvrir le navigateur.
pub fn ecouter() -> Result<TcpListener, String> {
    TcpListener::bind(("127.0.0.1", PORT_RAPPEL)).map_err(|e| {
        format!("Le port local {PORT_RAPPEL} est indisponible ({e}). Une autre connexion est peut-être déjà en cours.")
    })
}

const PAGE_RETOUR: &str = "<!doctype html><meta charset=\"utf-8\"><title>FamilySearch</title>\
<body style=\"font-family:Segoe UI,sans-serif;padding:40px\"><h2>FamilySearch</h2>\
<p>Vous pouvez fermer cette page et revenir au programme de généalogie.</p></body>";

/// Attend le retour du navigateur et rend le code d'autorisation.
pub fn attendre_code(ecoute: TcpListener, etat: &str, delai: Duration) -> Result<String, String> {
    ecoute.set_nonblocking(true).map_err(|e| e.to_string())?;
    let limite = Instant::now() + delai;
    loop {
        match ecoute.accept() {
            Ok((mut flux, _)) => {
                flux.set_nonblocking(false).ok();
                flux.set_read_timeout(Some(Duration::from_secs(10))).ok();
                let mut ligne = String::new();
                BufReader::new(&flux).read_line(&mut ligne).map_err(|e| e.to_string())?;
                let chemin = ligne.split_whitespace().nth(1).unwrap_or("");
                // Le navigateur demande aussi /favicon.ico : on l'ignore.
                if !chemin.starts_with("/familysearch-auth") {
                    let _ = flux.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                    continue;
                }
                let _ = flux.write_all(
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{PAGE_RETOUR}", PAGE_RETOUR.len()).as_bytes(),
                );
                let requete = chemin.split_once('?').map(|x| x.1).unwrap_or("");
                let params: HashMap<String, String> = requete
                    .split('&')
                    .filter_map(|kv| kv.split_once('='))
                    .map(|(k, v)| (k.to_string(), decoder(v)))
                    .collect();
                if let Some(e) = params.get("error") {
                    let detail = params.get("error_description").cloned().unwrap_or_default();
                    return Err(format!("FamilySearch a refusé la connexion : {e} {detail}").trim().to_string());
                }
                if params.get("state").map(String::as_str) != Some(etat) {
                    return Err("Erreur de sécurité : état OAuth invalide.".into());
                }
                return params.get("code").cloned().ok_or_else(|| "Aucun code reçu de FamilySearch.".to_string());
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() > limite {
                    return Err("Délai dépassé : la connexion FamilySearch n'a pas abouti.".into());
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(60)).user_agent(concat!("GenFam/", env!("CARGO_PKG_VERSION"))).build()
}

fn erreur_http(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(401, _) => "Session FamilySearch expirée ou refusée : reconnectez-vous.".into(),
        ureq::Error::Status(403, _) => "FamilySearch refuse l'accès (App Key non autorisée sur cet environnement).".into(),
        ureq::Error::Status(code, r) => {
            let corps = r.into_string().unwrap_or_default();
            format!("FamilySearch a répondu {code} : {}", corps.chars().take(300).collect::<String>())
        }
        ureq::Error::Transport(t) => format!("Réseau : {t}"),
    }
}

/// Échange le code contre un jeton d'accès.
pub fn echanger(c: &Connexion, code: &str) -> Result<String, String> {
    let r: Value = agent()
        .post(&format!("{}/cis-web/oauth2/v3/token", c.env.ident()))
        .set("Accept", "application/json")
        .send_form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &c.cle),
            ("redirect_uri", URI_RAPPEL),
            ("code_verifier", &c.verificateur),
        ])
        .map_err(erreur_http)?
        .into_json()
        .map_err(|e| e.to_string())?;
    r["access_token"].as_str().map(str::to_string).ok_or_else(|| "Réponse sans jeton d'accès.".into())
}

fn obtenir(env: Environnement, jeton: &str, chemin: &str) -> Result<Value, String> {
    agent()
        .get(&format!("{}{chemin}", env.api()))
        .set("Authorization", &format!("Bearer {jeton}"))
        .set("Accept", "application/x-fs-v1+json")
        .call()
        .map_err(erreur_http)?
        .into_json()
        .map_err(|e| e.to_string())
}

/// Nom et identifiant d'arbre de l'utilisateur connecté.
pub fn utilisateur(env: Environnement, jeton: &str) -> Result<(String, String), String> {
    let v = obtenir(env, jeton, "/platform/users/current")?;
    let u = &v["users"][0];
    let nom = u["displayName"].as_str().or(u["contactName"].as_str()).unwrap_or("").to_string();
    Ok((nom, u["personId"].as_str().unwrap_or("").to_string()))
}

/// Personne telle que rendue par FamilySearch, avec son numéro d'ascendance.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PersonneFs {
    pub numero: u64,
    pub fsid: String,
    pub prenoms: String,
    pub nom: String,
    pub sexe: String,
    pub naissance: String,
    pub lieu_naissance: String,
    pub deces: String,
    pub lieu_deces: String,
}

/// Lit une personne GEDCOM X (format `application/x-fs-v1+json`).
pub fn lire_personne(v: &Value) -> Option<PersonneFs> {
    let d = &v["display"];
    let numero = d["ascendancyNumber"].as_str()?.parse().ok()?;
    let mut p = PersonneFs { numero, fsid: v["id"].as_str()?.to_string(), ..Default::default() };
    if let Some(parts) = v["names"][0]["nameForms"][0]["parts"].as_array() {
        for part in parts {
            let valeur = part["value"].as_str().unwrap_or("").to_string();
            match part["type"].as_str().unwrap_or("") {
                "http://gedcomx.org/Given" => p.prenoms = valeur,
                "http://gedcomx.org/Surname" => p.nom = valeur,
                _ => {}
            }
        }
    }
    if p.prenoms.is_empty() && p.nom.is_empty() {
        // À défaut de parties, le dernier mot du nom affiché est pris pour le nom.
        let nom = d["name"].as_str().unwrap_or("").trim().to_string();
        match nom.rsplit_once(' ') {
            Some((a, b)) => { p.prenoms = a.to_string(); p.nom = b.to_string(); }
            None => p.nom = nom,
        }
    }
    p.sexe = match d["gender"].as_str().or(v["gender"]["type"].as_str()).unwrap_or("") {
        "Male" | "http://gedcomx.org/Male" => "M".into(),
        "Female" | "http://gedcomx.org/Female" => "F".into(),
        _ => String::new(),
    };
    let texte = |k: &str| d[k].as_str().unwrap_or("").to_string();
    p.naissance = texte("birthDate");
    p.lieu_naissance = texte("birthPlace");
    p.deces = texte("deathDate");
    p.lieu_deces = texte("deathPlace");
    Some(p)
}

/// Ascendance d'une personne FamilySearch sur `generations` générations (1 à 8).
pub fn ascendance(env: Environnement, jeton: &str, fsid: &str, generations: u32) -> Result<Vec<PersonneFs>, String> {
    let g = generations.clamp(1, GENERATIONS_MAX);
    let v = obtenir(env, jeton, &format!("/platform/tree/ancestry?person={}&generations={g}&personDetails=", encoder(fsid)))?;
    let mut r: Vec<PersonneFs> = v["persons"].as_array().map(|a| a.iter().filter_map(lire_personne).collect()).unwrap_or_default();
    r.sort_by_key(|p| p.numero);
    Ok(r)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Bilan {
    /// Personnes créées dans le GEDCOM.
    pub ajoutees: usize,
    /// Personnes déjà présentes, reconnues par leur identifiant FamilySearch.
    pub reconnues: usize,
    /// Liens parent → enfant créés.
    pub liens: usize,
    /// Liens écartés : l'enfant a déjà d'autres parents dans le GEDCOM.
    pub ecartes: usize,
}

/// Verse une ascendance FamilySearch dans le document. Le N° 1 est `racine`, la
/// personne locale d'où part l'import ; rien de ce qui existe n'est écrasé.
pub fn verser(doc: &mut Document, arbre: &Arbre, racine: &str, liste: &[PersonneFs]) -> Bilan {
    let mut bilan = Bilan::default();
    let mut locaux: HashMap<u64, String> = HashMap::new();
    let mut crees: HashMap<String, String> = HashMap::new();
    for fp in liste {
        if fp.numero == 1 {
            locaux.insert(1, racine.to_string());
            if arbre.personne(racine).is_some_and(|p| p.fsid.is_empty()) {
                doc.ajouter_reference(racine, "INDI", "_FSFTID", &fp.fsid);
            }
            continue;
        }
        if let Some(p) = arbre.par_fsid(&fp.fsid) {
            locaux.insert(fp.numero, p.id.clone());
            bilan.reconnues += 1;
            continue;
        }
        if let Some(id) = crees.get(&fp.fsid) {
            // Implexe : le même ancêtre à deux numéros.
            locaux.insert(fp.numero, id.clone());
            continue;
        }
        let id = doc.prochain_id_personne();
        let mut p = Personne::nouvelle(&id);
        p.prenoms = fp.prenoms.clone();
        p.nom = fp.nom.clone();
        p.nom_gedcom = if fp.nom.is_empty() { fp.prenoms.clone() } else { format!("{} /{}/", fp.prenoms, fp.nom).trim().to_string() };
        p.sexe = fp.sexe.clone();
        p.naissance = fp.naissance.clone();
        p.lieu_naissance = fp.lieu_naissance.clone();
        p.deces = fp.deces.clone();
        p.lieu_deces = fp.lieu_deces.clone();
        p.fsid = fp.fsid.clone();
        doc.ajouter_personne(&p, None, None);
        crees.insert(fp.fsid.clone(), id.clone());
        locaux.insert(fp.numero, id);
        bilan.ajoutees += 1;
    }
    let mut numeros: Vec<u64> = locaux.keys().copied().collect();
    numeros.sort_unstable();
    for n in numeros {
        let (Some(pere), Some(mere)) = (n.checked_mul(2), n.checked_mul(2).and_then(|x| x.checked_add(1))) else { continue };
        let pere = locaux.get(&pere).map(String::as_str);
        let mere = locaux.get(&mere).map(String::as_str);
        if pere.is_none() && mere.is_none() {
            continue;
        }
        let enfant = &locaux[&n];
        if let Some(existant) = arbre.personne(enfant) {
            let actuels: Vec<&str> = arbre.parents(existant).iter().map(|p| p.id.as_str()).collect();
            if !actuels.is_empty() {
                let voulus: Vec<&str> = [pere, mere].into_iter().flatten().collect();
                if !voulus.iter().all(|v| actuels.contains(v)) {
                    bilan.ecartes += 1;
                }
                continue;
            }
        }
        doc.lier_parents(enfant, pere, mere);
        bilan.liens += 1;
    }
    bilan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gedcom::analyser;
    use serde_json::json;

    #[test]
    fn pkce_et_url() {
        let c = Connexion::preparer(Environnement::Integration, "a-b c");
        assert!(c.url.starts_with("https://identint.familysearch.org/cis-web/oauth2/v3/authorization?"));
        assert!(c.url.contains("client_id=a-b%20c"));
        assert!(c.url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A57938%2Ffamilysearch-auth"));
        assert!(c.verificateur.len() >= 43 && c.verificateur.len() <= 128);
        assert_eq!(decoder("a%20b+c%C3%A9"), "a b cé");
    }

    #[test]
    fn rappel_local() {
        let ecoute = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = ecoute.local_addr().unwrap().port();
        let fil = std::thread::spawn(move || attendre_code(ecoute, "ETAT", Duration::from_secs(10)));
        let mut f = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        f.write_all(b"GET /familysearch-auth?code=XYZ%2B1&state=ETAT HTTP/1.1\r\nHost: x\r\n\r\n").unwrap();
        assert_eq!(fil.join().unwrap(), Ok("XYZ+1".to_string()));
    }

    fn fs(n: u64, id: &str, prenom: &str, nom: &str, genre: &str) -> Value {
        json!({"id": id, "display": {"name": format!("{prenom} {nom}"), "gender": genre, "ascendancyNumber": n.to_string(),
               "birthDate": "1900"},
               "names": [{"nameForms": [{"parts": [
                   {"type": "http://gedcomx.org/Given", "value": prenom},
                   {"type": "http://gedcomx.org/Surname", "value": nom}]}]}]})
    }

    #[test]
    fn versement_ascendance() {
        let texte = include_str!("../tests/donnees/exemple.ged");
        let arbre = analyser(texte);
        let mut doc = Document::depuis_texte(texte);
        // Claire (@I6@) n'a pas de parents : FamilySearch en fournit deux, plus un grand-père.
        let liste: Vec<PersonneFs> = [
            fs(1, "C-1", "Claire", "PETIT", "Female"),
            fs(2, "P-2", "Henri", "PETIT", "Male"),
            fs(3, "M-3", "Rose", "BLANC", "Female"),
            fs(4, "G-4", "Émile", "PETIT", "Male"),
        ].iter().filter_map(lire_personne).collect();
        assert_eq!(liste[3].prenoms, "Émile");
        let b = verser(&mut doc, &arbre, "@I6@", &liste);
        assert_eq!(b, Bilan { ajoutees: 3, reconnues: 0, liens: 2, ecartes: 0 });
        let a = analyser(&doc.texte());
        let claire = a.personne("@I6@").unwrap();
        assert_eq!(claire.fsid, "C-1");
        let parents: Vec<String> = a.parents(claire).iter().map(|p| p.nom_affiche()).collect();
        assert_eq!(parents, vec!["Henri PETIT", "Rose BLANC"]);
        let henri = a.par_fsid("P-2").unwrap();
        assert_eq!(a.parents(henri)[0].nom_affiche(), "Émile PETIT");
        assert_eq!(henri.naissance, "1900");

        // Second import identique : tout est reconnu, rien n'est dupliqué.
        let arbre2 = a.clone();
        let mut doc2 = doc.clone();
        let b2 = verser(&mut doc2, &arbre2, "@I6@", &liste);
        assert_eq!(b2, Bilan { ajoutees: 0, reconnues: 3, liens: 0, ecartes: 0 });

        // Paul a déjà des parents : un père différent venu de FamilySearch est écarté.
        let mut doc3 = Document::depuis_texte(texte);
        let l3: Vec<PersonneFs> = [fs(1, "LZ1A-B2C", "Paul", "MARTIN", "Male"), fs(2, "X-9", "Autre", "PERE", "Male")]
            .iter().filter_map(lire_personne).collect();
        let b3 = verser(&mut doc3, &arbre, "@I1@", &l3);
        assert_eq!(b3, Bilan { ajoutees: 1, reconnues: 0, liens: 0, ecartes: 1 });
    }
}
