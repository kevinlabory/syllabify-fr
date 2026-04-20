// SPDX-License-Identifier: GPL-3.0-or-later
//! Décodeur : post-traitements phonologiques et assemblage syllabique.
//! Port de `decoder.py`.

use crate::cleaner::clean;
use crate::data::MOTS_OSSE;
use crate::parser::parse;
use crate::phoneme::{classify, PhonClass};

/// Un phonème décoré avec la chaîne de lettres du mot d'origine qui le produit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPhoneme {
    pub code: String,
    pub letters: String,
}

/// Mode de syllabification : orale ou écrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyllableMode {
    /// Syllabes écrites : la dernière syllabe muette reste détachée (ex: `é-co-le`)
    Written,
    /// Syllabes orales : la dernière syllabe muette est fusionnée avec la précédente (ex: `é-cole`)
    Oral,
}

/// Mode d'assemblage syllabique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssembleMode {
    /// **Mode historique — non aligné avec LireCouleur 6 v6.**
    ///
    /// En mode phonologique, les consonnes doubles restent dans la même syllabe
    /// (ex : `homme` → `ho-mme`). Ce mode n'est plus maintenu en conformité avec
    /// LC6 depuis la migration v5→v6 ; il peut diverger sur certains mots.
    ///
    /// Utiliser [`AssembleMode::Std`] pour une conformité maximale avec LC6.
    #[deprecated(
        since = "0.4.0",
        note = "non aligné avec LireCouleur 6 v6 ; préférer AssembleMode::Std"
    )]
    Lc,
    /// Mode pédagogique (défaut LC6) : les consonnes doubles sont réparties entre deux syllabes
    /// (ex : `homme` → `hom-me`, `pomme` → `pom-me`).
    Std,
}

/// Un élément intermédiaire de l'assemblage : (classe, indices de phonèmes concernés).
#[derive(Debug, Clone)]
struct SylPh {
    class: PhonClass,
    indices: Vec<usize>,
}

/// Retourne tous les indices des phonèmes dont le code est parmi `values` dans `codes[..=limit]`.
fn indices_of(codes: &[String], values: &[&str], limit: usize) -> Vec<usize> {
    let mut out = Vec::new();
    for (i, c) in codes.iter().enumerate() {
        if i > limit {
            break;
        }
        if values.contains(&c.as_str()) {
            out.push(i);
        }
    }
    out
}

/// Post-traitement `eu` : détermine si `x` est ouvert ou fermé.
/// Équivalent `__post_process_e`.
pub fn post_process_e(pp: &mut Vec<DecodedPhoneme>) {
    if pp.len() <= 1 {
        return;
    }
    let codes: Vec<String> = pp.iter().map(|p| p.code.clone()).collect();
    if !codes.iter().any(|c| c == "x") {
        return;
    }

    // Indice du dernier phonème prononcé (skippe les '#' finaux)
    let mut nb_ph = codes.len() - 1;
    while nb_ph >= 1 && codes[nb_ph] == "#" {
        nb_ph -= 1;
    }

    let i_x = indices_of(&codes, &["x"], nb_ph);
    if i_x.is_empty() {
        return;
    }
    let i_ph = *i_x.last().unwrap();

    // Pas dans les 3 derniers phonèmes prononcés : on ne peut rien décider
    if i_ph + 2 < nb_ph {
        return;
    }

    if i_ph == nb_ph {
        // Dernier phonème prononcé = 'eu' → fermé
        pp[i_ph].code = "x^".to_string();
        return;
    }

    let consonnes_eu_ferme = ["z", "z_s", "t"];
    if consonnes_eu_ferme.contains(&codes[i_ph + 1].as_str())
        && codes[nb_ph] == "q_caduc"
    {
        pp[i_ph].code = "x^".to_string();
    }
}

/// Post-traitement `o` : ouvert ou fermé.
pub fn post_process_o(pp: &mut Vec<DecodedPhoneme>) {
    if pp.len() <= 1 {
        return;
    }
    let codes: Vec<String> = pp.iter().map(|p| p.code.clone()).collect();
    if !codes.iter().any(|c| c == "o") {
        return;
    }

    let consonnes_syllabe_fermee = [
        "p", "k", "b", "d", "g", "f", "f_ph", "s^", "l", "r", "m", "n",
    ];

    let mut nb_ph = codes.len() - 1;
    while nb_ph > 0 && codes[nb_ph] == "#" {
        nb_ph -= 1;
    }

    let i_o = indices_of(&codes, &["o"], nb_ph);

    // Reconstituer le mot sans les phonèmes muets de fin
    let mot: String = pp[..=nb_ph].iter().map(|p| p.letters.as_str()).collect();

    if MOTS_OSSE.binary_search(&mot.as_str()).is_ok() {
        if let Some(&last_o) = i_o.last() {
            pp[last_o].code = "o_ouvert".to_string();
        }
        return;
    }

    let consonnes = ["p", "t", "k", "b", "d", "g", "f", "f_ph", "s", "s^",
                     "v", "z", "z^", "l", "r", "m", "n",
                     "k_qu", "z^_g", "g_u", "s_c", "s_t", "z_s", "ks", "gz"];

    for &i_ph in &i_o {
        if i_ph == nb_ph {
            return; // syllabe tonique ouverte en fin de mot : o fermé (sortie fonction)
        }
        if pp[i_ph].letters != "ô" {
            let next = codes.get(i_ph + 1).map(String::as_str).unwrap_or("");
            let next2 = codes.get(i_ph + 2).map(String::as_str).unwrap_or("");

            if i_ph + 2 == nb_ph
                && consonnes_syllabe_fermee.contains(&next)
                && next2 == "q_caduc"
            {
                pp[i_ph].code = "o_ouvert".to_string();
            } else if ["r", "z^_g", "v"].contains(&next) {
                pp[i_ph].code = "o_ouvert".to_string();
            } else if i_ph + 2 < nb_ph
                && consonnes.contains(&next)
                && consonnes.contains(&next2)
            {
                pp[i_ph].code = "o_ouvert".to_string();
            }
        }
    }
}

/// Post-traitement `w` : associe `u + voyelle` en phonème composé `w_X`.
pub fn post_process_w(pp: &mut Vec<DecodedPhoneme>) {
    if pp.len() <= 1 {
        return;
    }

    // v6 : on ne fusionne plus `u + voyelle → w_voyelle`, on garde juste `wa` tel quel.
    // Le `wa` produit directement par l'automate reste inchangé (c'est une voyelle).
    // Cette fonction devient quasi no-op en v6 mais est conservée pour compat API.
    let _ = pp;
}

/// Post-traitement yod (v6) : remplace `i + voyelle` par `j` simple (plus de fusion `j_V`).
pub fn post_process_yod(pp: &mut Vec<DecodedPhoneme>, _mode: SyllableMode) {
    if pp.len() <= 1 {
        return;
    }
    let phon_suivant = [
        "a", "a~", "e", "e^", "e_comp", "e^_comp", "o", "o_comp", "o~", "e~",
        "x", "x^", "u",
    ];

    for i in 0..pp.len() - 1 {
        if pp[i].code == "i" && phon_suivant.contains(&pp[i + 1].code.as_str()) {
            pp[i].code = "j".to_string();
        }
    }
}

/// Assemblage des phonèmes en syllabes.
/// Retourne la liste des syllabes (chaque syllabe = indices des phonèmes la composant)
/// et la liste de phonèmes (potentiellement modifiée en mode STD avec duplication).
pub fn assemble_syllables(
    phonemes: &[DecodedPhoneme],
    assemble_mode: AssembleMode,
    syl_mode: SyllableMode,
) -> (Vec<Vec<usize>>, Vec<DecodedPhoneme>) {
    let nb_phon = phonemes.len();
    if nb_phon < 2 {
        return (vec![(0..nb_phon).collect()], phonemes.to_vec());
    }

    // 1. Mode STD : dupliquer uniquement les phonèmes CONSONNES aux lettres doublées.
    // LC6 ne dédouble pas les semi-voyelles (j, w simples) : "fille" avec j(ll)
    // reste j(ll) et non pas j(l)+j(l), d'où la segmentation fi|lle et pas fil|le.
    let mut nphonemes: Vec<DecodedPhoneme> = Vec::with_capacity(nb_phon);
    if assemble_mode == AssembleMode::Std {
        for ph in phonemes {
            let c = classify(&ph.code);
            let is_semi_consonne = ph.code.starts_with("j_")
                || ph.code.starts_with("w_")
                || ph.code.starts_with("y_");
            let eligible = c == PhonClass::Consonant || is_semi_consonne;
            if eligible && ph.letters.chars().count() > 1 {
                let chars: Vec<char> = ph.letters.chars().collect();
                let n = chars.len();
                if chars[n - 1] == chars[n - 2] {
                    let prefix: String = chars[..n - 1].iter().collect();
                    let last: String = chars[n - 1..].iter().collect();
                    nphonemes.push(DecodedPhoneme { code: ph.code.clone(), letters: prefix });
                    nphonemes.push(DecodedPhoneme { code: ph.code.clone(), letters: last });
                } else {
                    nphonemes.push(ph.clone());
                }
            } else {
                nphonemes.push(ph.clone());
            }
        }
    } else {
        nphonemes = phonemes.to_vec();
    }

    let nb_phon = nphonemes.len();

    // 2. Construire la liste sylph (classe + indices)
    // Comportement LC6 : un phonème "vide" (code == "") provenant d'un caractère
    // non reconnu par l'automate (ex: '-') est COMPLÈTEMENT ignoré — il n'apparaît
    // ni dans sylph ni dans les syllabes reconstituées. C'est ce qui permet
    // "grand-père" → ["grand", "pè", "re"] (sans le tiret).
    let mut sylph: Vec<SylPh> = Vec::with_capacity(nb_phon);
    for (i, ph) in nphonemes.iter().enumerate() {
        if ph.code.is_empty() {
            // Phonème non décodé : on ne l'ajoute pas à sylph, donc
            // aucune syllabe ne le référencera.
            continue;
        }
        let class = if ph.code.starts_with("j_")
            || ph.code.starts_with("w_")
            || ph.code.starts_with("y_")
        {
            PhonClass::Vowel
        } else {
            classify(&ph.code)
        };
        sylph.push(SylPh { class, indices: vec![i] });
    }

    // 3. Mixer les doubles consonnes type bl, br, tr, cr, chr, pl...
    let attaque_premiere = ["b", "k", "p", "t", "g", "d", "f", "v"];
    let mut i = 0;
    while i + 1 < sylph.len() {
        if sylph[i].class == PhonClass::Consonant && sylph[i + 1].class == PhonClass::Consonant {
            let phon0 = &nphonemes[sylph[i].indices[0]].code;
            let phon1 = &nphonemes[sylph[i + 1].indices[0]].code;
            if (phon1 == "l" || phon1 == "r") && attaque_premiere.contains(&phon0.as_str()) {
                let indices1 = sylph[i + 1].indices.clone();
                sylph[i].indices.extend(indices1);
                sylph.remove(i + 1);
                // ne pas incrémenter
                continue;
            }
        }
        i += 1;
    }

    // 4. Mixer les doubles voyelles [y]+[i], [u]+[i|e~|o~]
    let mut i = 0;
    while i + 1 < sylph.len() {
        if sylph[i].class == PhonClass::Vowel && sylph[i + 1].class == PhonClass::Vowel {
            let phon1 = nphonemes[sylph[i].indices[0]].code.clone();
            let phon2 = nphonemes[sylph[i + 1].indices[0]].code.clone();
            let merge = (phon1 == "y" && phon2 == "i")
                || (phon1 == "u" && (phon2 == "i" || phon2 == "e~" || phon2 == "o~"));
            if merge {
                let indices1 = sylph[i + 1].indices.clone();
                sylph[i].indices.extend(indices1);
                sylph.remove(i + 1);
                continue;
            }
        }
        i += 1;
    }

    // 5. Accrocher les lettres muettes ('#') à ce qui précède
    let mut i = 0;
    while i + 1 < sylph.len() {
        if sylph[i + 1].class == PhonClass::Silent {
            let indices1 = sylph[i + 1].indices.clone();
            sylph[i].indices.extend(indices1);
            sylph.remove(i + 1);
            continue;
        }
        i += 1;
    }

    // 6. Assembler les syllabes : attaque (non-voyelles) + noyau (voyelle)
    let mut sylls: Vec<Vec<usize>> = Vec::new();
    let nb_sylph = sylph.len();
    let mut i = 0;
    let mut j = 0usize;
    while i < nb_sylph {
        j = i;
        // Tout ce qui n'est pas voyelle va dans l'attaque
        while i < nb_sylph && sylph[i].class != PhonClass::Vowel {
            i += 1;
        }
        // Inclure la voyelle
        if i < nb_sylph && sylph[i].class == PhonClass::Vowel {
            i += 1;
            let mut cur_syl: Vec<usize> = Vec::new();
            for k in j..i {
                cur_syl.extend(sylph[k].indices.iter().copied());
            }
            j = i;
            sylls.push(cur_syl);
        }

        // Ce bloc DOIT être au même niveau que le if ci-dessus (conforme au Python).
        // Si la consonne qui suit est elle-même suivie d'une autre consonne,
        // on la rattache à la syllabe courante (coda).
        if i + 1 < nb_sylph {
            let phon_i_idx = *sylph[i].indices.last().unwrap();
            let phon_i1_idx = sylph[i + 1].indices[0];
            let last_letter = nphonemes[phon_i_idx].letters.chars().last().unwrap_or(' ');
            let first_letter = nphonemes[phon_i1_idx].letters.chars().next().unwrap_or(' ');
            let consonnes = "bcdfghjklmnpqrstvwxzç";
            if consonnes.contains(last_letter) && consonnes.contains(first_letter) {
                if let Some(last) = sylls.last_mut() {
                    last.extend(sylph[i].indices.iter().copied());
                }
                i += 1;
                j = i;
            }
        }
    }

    if sylls.is_empty() {
        return (vec![(0..nb_phon).collect()], nphonemes);
    }

    // 7. Ajouter à la dernière syllabe TOUT ce qui reste à partir de j (et uniquement à partir de j).
    // Important : pas un HashSet sur tous les non-consommés, sinon on ré-agrège des phonèmes
    // délibérément laissés entre deux syllabes (ex: le 'r' central de "frère").
    for k in j..nb_sylph {
        sylls.last_mut().unwrap().extend(sylph[k].indices.iter().copied());
    }

    // 8. Mode oral : fusionner la dernière syllabe qui finit en q_caduc
    if syl_mode == SyllableMode::Oral && sylls.len() > 1 {
        let last = sylls.last().unwrap();
        let mut k = last.len() as isize - 1;
        while k > 0 {
            let code = &nphonemes[last[k as usize]].code;
            if code != "#" && code != "verb_3p" {
                break;
            }
            k -= 1;
        }
        if k >= 0 && nphonemes[last[k as usize]].code.ends_with("q_caduc") {
            let last_syl = sylls.pop().unwrap();
            sylls.last_mut().unwrap().extend(last_syl);
        }
    }

    (sylls, nphonemes)
}

/// Extrait les phonèmes d'un mot unique (après nettoyage).
pub fn extract_phonemes_word(word: &str, novice_reader: bool, mode: SyllableMode) -> Vec<DecodedPhoneme> {
    // Le parser travaille en minuscules (l'automate est défini en minuscules).
    // On préserve la casse originale dans les `letters` de sortie.
    let lower: String = word.chars().flat_map(|c| c.to_lowercase()).collect();
    let raw_phons = parse(&lower);
    let chars_orig: Vec<char> = word.chars().collect();
    // On suppose que lower préserve le nombre de caractères (vrai pour le français ;
    // si une lettre lowercasait en plusieurs chars, on replierait sur chars_orig seulement).
    let chars_lower: Vec<char> = lower.chars().collect();
    let mut cursor = 0usize;
    let mut out: Vec<DecodedPhoneme> = Vec::with_capacity(raw_phons.len());
    for ph in raw_phons {
        let end = (cursor + ph.step).min(chars_lower.len());
        // Lettres originales (avec casse) alignées par index char
        let orig_end = end.min(chars_orig.len());
        let letters: String = chars_orig[cursor.min(chars_orig.len())..orig_end].iter().collect();
        out.push(DecodedPhoneme { code: ph.code, letters });
        cursor = end;
    }

    post_process_e(&mut out);
    if !novice_reader {
        post_process_w(&mut out);
        post_process_yod(&mut out, mode);
        post_process_o(&mut out);
    }
    out
}

/// API principale texte : équivalent `extract_syllables` pour un texte arbitraire.
/// Retourne une liste d'éléments : soit un vecteur de syllabes (pour chaque mot),
/// soit une chaîne de texte brut (ponctuation, espaces, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextChunk {
    Word(Vec<String>),
    Raw(String),
}

pub fn extract_syllables(
    text: &str,
    novice_reader: bool,
    assemble_mode: AssembleMode,
    syl_mode: SyllableMode,
) -> Vec<TextChunk> {
    let ultext = clean(text, ' ');
    let words: Vec<&str> = ultext.split_whitespace().collect();
    let mut out: Vec<TextChunk> = Vec::new();
    let mut p_text = 0usize;

    let text_chars: Vec<char> = text.chars().collect();
    let ultext_chars: Vec<char> = ultext.chars().collect();

    // Mot précédent (lowercase, apostrophe typo normalisée) pour les homographes.
    let mut previous_word: Option<String> = None;

    for word in &words {
        let wlen = word.chars().count();
        let word_chars: Vec<char> = word.chars().collect();
        let pp_text = match find_subseq(&ultext_chars, &word_chars, p_text) {
            Some(p) => p,
            None => continue,
        };

        if pp_text > p_text {
            let raw: String = text_chars[p_text..pp_text].iter().collect();
            out.push(TextChunk::Raw(raw));
        }

        let original_word: String = text_chars[pp_text..pp_text + wlen].iter().collect();
        let lower_word = original_word.to_lowercase();

        // Désambiguïsation des homographes (v6)
        let phonemes: Vec<DecodedPhoneme> = match crate::homographs::lookup(
            &lower_word,
            previous_word.as_deref(),
        ) {
            Some(coded) => coded
                .into_iter()
                .map(|(code, letters)| DecodedPhoneme { code, letters })
                .collect(),
            None => extract_phonemes_word(&original_word, novice_reader, syl_mode),
        };
        let (sylls, nphons) = assemble_syllables(&phonemes, assemble_mode, syl_mode);

        let sylls_strings: Vec<String> = sylls
            .iter()
            .map(|syl| syl.iter().map(|&i| nphons[i].letters.clone()).collect::<String>())
            .collect();

        out.push(TextChunk::Word(sylls_strings));
        p_text = pp_text + wlen;

        // Conserver ce mot (normalisé) comme contexte pour le suivant.
        previous_word = Some(lower_word.replace('\u{2019}', "'"));
    }

    if p_text < text_chars.len() {
        let raw: String = text_chars[p_text..].iter().collect();
        out.push(TextChunk::Raw(raw));
    }

    out
}

fn find_subseq(haystack: &[char], needle: &[char], start: usize) -> Option<usize> {
    if needle.is_empty() || start >= haystack.len() {
        return None;
    }
    let nlen = needle.len();
    if start + nlen > haystack.len() {
        return None;
    }
    for i in start..=haystack.len().saturating_sub(nlen) {
        if haystack[i..i + nlen] == needle[..] {
            return Some(i);
        }
    }
    None
}
