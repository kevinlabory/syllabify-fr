// SPDX-License-Identifier: GPL-3.0-or-later
//! Homographes non homophones : mots qui s'écrivent pareil mais se prononcent différemment
//! selon le contexte (déterminant / pronom qui précède).
//!
//! Exemples canoniques :
//! - `couvent` : "le couvent" (lieu, kuv‿ɑ̃) vs "elles couvent" (verbe, kuv)
//! - `est` : "il est" (verbe, ɛ) vs "vers l'est" (direction, ɛst)
//! - `fils` : "les fils" (fibres, fil) vs "le fils" (enfant, fis)
//! - `violent` : "un violent éclat" (adj, vjɔlɑ̃) vs "ils violent" (verbe, vjɔl)
//!
//! Port de `testeHomographeNonHomophone` de LireCouleur 6.

use crate::data::HOMOGRAPHES;

/// Si `word` est un homographe non homophone et que `previous_word` matche
/// un de ses contextes, retourne la liste de phonèmes à utiliser sous forme
/// `(code_phonème, lettres)`. Sinon retourne `None`.
///
/// La comparaison sur `previous_word` ignore la casse et les apostrophes
/// typographiques.
pub fn lookup(word: &str, previous_word: Option<&str>) -> Option<Vec<(String, String)>> {
    let prev = previous_word?.to_lowercase();
    // Normaliser l'apostrophe typographique
    let prev = prev.replace('\u{2019}', "'");

    for (key, variants) in HOMOGRAPHES {
        if *key != word {
            continue;
        }
        for v in *variants {
            if v.precedent.iter().any(|p| *p == prev) {
                return Some(
                    v.codage
                        .iter()
                        .map(|(p, l)| (p.to_string(), l.to_string()))
                        .collect(),
                );
            }
        }
        // Mot reconnu comme homographe mais pas de contexte matchant :
        // on laisse l'automate par défaut faire son travail.
        return None;
    }
    None
}

/// Liste les mots actuellement reconnus comme homographes non homophones.
#[allow(dead_code)]
pub fn known_homographs() -> impl Iterator<Item = &'static str> {
    HOMOGRAPHES.iter().map(|(k, _)| *k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn couvent_verbe() {
        // "elles couvent" : verbe couver 3pp
        let r = lookup("couvent", Some("elles"));
        assert!(r.is_some());
        let phons = r.unwrap();
        // Le dernier phonème doit être verb_3p (marqueur -ent muet)
        assert_eq!(phons.last().map(|p| p.0.as_str()), Some("verb_3p"));
    }

    #[test]
    fn couvent_nom() {
        // "le couvent" : nom (lieu)
        let r = lookup("couvent", Some("le"));
        assert!(r.is_some());
        let phons = r.unwrap();
        // Le -ent se prononce a~
        assert!(phons.iter().any(|p| p.0 == "a~"));
    }

    #[test]
    fn est_verbe_vs_nom() {
        let r_verbe = lookup("est", Some("il")).unwrap();
        assert_eq!(r_verbe.len(), 1);
        assert_eq!(r_verbe[0].1, "est");

        let r_nom = lookup("est", Some("l'")).unwrap();
        assert_eq!(r_nom.len(), 3); // e, s, t (le t est prononcé)
    }

    #[test]
    fn mot_non_homographe() {
        assert!(lookup("chat", Some("le")).is_none());
    }

    #[test]
    fn homographe_sans_contexte() {
        assert!(lookup("est", None).is_none());
    }
}
