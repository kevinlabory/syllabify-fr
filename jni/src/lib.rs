// SPDX-License-Identifier: GPL-3.0-or-later
use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jobjectArray, jstring};
use jni::JNIEnv;
use serde_json::{json, Value};
use syllabify_fr::letters::{match_letters, presets, render_letters_html, LetterRule, RenderMode};
use syllabify_fr::{phonemes, render_html, render_word_html, syllabify_text, syllables, TextChunk};

// --- JSON helpers via serde_json ---
//
// Échappement RFC 8259-conforme par construction (caractères de contrôle
// U+0000..U+001F sérialisés en `\uXXXX`, `\b`/`\f` en formes courtes).
// Remplace les helpers maison antérieurs qui ne couvraient que
// `" \ \n \r \t` (cf. audit #3 / hand-rolled JSON).

fn chunk_to_value(chunk: &TextChunk) -> Value {
    match chunk {
        TextChunk::Word(syls) => json!({ "kind": "word", "syllables": syls }),
        TextChunk::Raw(text) => json!({ "kind": "raw", "text": text }),
        _ => json!({ "kind": "unknown" }),
    }
}

fn chunks_to_json(chunks: &[TextChunk]) -> String {
    let values: Vec<Value> = chunks.iter().map(chunk_to_value).collect();
    // `to_string` ne peut échouer que sur `Map<NonString, _>` ou erreur d'IO —
    // aucun des deux cas n'est possible sur des `Value` construits ici.
    serde_json::to_string(&values).expect("serde_json::to_string never fails for owned Values")
}

fn phonemes_to_json(pairs: &[(String, String)]) -> String {
    let values: Vec<Value> = pairs
        .iter()
        .map(|(code, letters)| json!([code, letters]))
        .collect();
    serde_json::to_string(&values).expect("serde_json::to_string never fails for owned Values")
}

// --- JNI helpers ---

fn jni_input<'a>(env: &mut JNIEnv<'a>, s: &JString<'a>) -> Option<String> {
    env.get_string(s).ok().map(|js| js.into())
}

fn jni_output(env: &mut JNIEnv, s: String) -> jstring {
    env.new_string(s)
        .map(|js| js.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

// --- Public JNI API ---
// Class: com.dyscolor.syllabify.SyllabifyFr
// Naming convention: Java_{package_underscored}_{ClassName}_{methodName}

/// `SyllabifyFr.syllables(word)` → `String[]`
///
/// Returns the syllables of a single French word.
/// Example: `"chocolat"` → `["cho", "co", "lat"]`
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_syllables<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    word: JString<'local>,
) -> jobjectArray {
    let word = match jni_input(&mut env, &word) {
        Some(w) => w,
        None => return std::ptr::null_mut(),
    };
    let syls = syllables(&word);
    let string_class = match env.find_class("java/lang/String") {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };
    let arr: JObjectArray =
        match env.new_object_array(syls.len() as i32, string_class, JString::default()) {
            Ok(a) => a,
            Err(_) => return std::ptr::null_mut(),
        };
    for (i, syl) in syls.iter().enumerate() {
        let js = match env.new_string(syl) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        if env.set_object_array_element(&arr, i as i32, js).is_err() {
            return std::ptr::null_mut();
        }
    }
    arr.into_raw()
}

/// `SyllabifyFr.syllabifyText(text)` → JSON `String`
///
/// Returns a JSON array of chunks.
/// Example: `"le chat"` → `[{"kind":"word","syllables":["le"]},{"kind":"raw","text":" "},{"kind":"word","syllables":["chat"]}]`
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_syllabifyText<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    text: JString<'local>,
) -> jstring {
    let text = match jni_input(&mut env, &text) {
        Some(t) => t,
        None => return std::ptr::null_mut(),
    };
    let chunks = syllabify_text(&text);
    jni_output(&mut env, chunks_to_json(&chunks))
}

/// `SyllabifyFr.phonemes(word)` → JSON `String`
///
/// Returns a JSON array of `[code, letters]` pairs.
/// Example: `"chat"` → `[["s^","ch"],["a","a"],["#","t"]]`
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_phonemes<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    word: JString<'local>,
) -> jstring {
    let word = match jni_input(&mut env, &word) {
        Some(w) => w,
        None => return std::ptr::null_mut(),
    };
    let pairs = phonemes(&word);
    jni_output(&mut env, phonemes_to_json(&pairs))
}

/// `SyllabifyFr.renderWordHtml(word)` → HTML `String`
///
/// Returns HTML with `<span class="syl syl-a/b">` syllable spans for a single word.
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_renderWordHtml<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    word: JString<'local>,
) -> jstring {
    let word = match jni_input(&mut env, &word) {
        Some(w) => w,
        None => return std::ptr::null_mut(),
    };
    jni_output(&mut env, render_word_html(&word))
}

/// `SyllabifyFr.renderHtml(text)` → HTML `String`
///
/// Returns HTML with syllable spans and liaison markers for full text.
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_renderHtml<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    text: JString<'local>,
) -> jstring {
    let text = match jni_input(&mut env, &text) {
        Some(t) => t,
        None => return std::ptr::null_mut(),
    };
    jni_output(&mut env, render_html(&text))
}

fn preset_rules(name: &str) -> Vec<LetterRule> {
    match name {
        "bdpq" => presets::bdpq(),
        "mnu" => presets::mnu(),
        "pir-pri" | "pir_pri" => presets::pir_pri(),
        _ => Vec::new(),
    }
}

fn parse_mode(mode: &str) -> RenderMode {
    match mode {
        "classes" => RenderMode::Classes,
        _ => RenderMode::Inline,
    }
}

/// `SyllabifyFr.highlightLetters(word, preset, mode)` → HTML `String`
///
/// `preset` accepts `"bdpq"`, `"mnu"`, or `"pir-pri"`.
/// `mode` accepts `"inline"` (default) or `"classes"`.
/// On unknown preset the word is returned HTML-escaped without spans.
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_highlightLetters<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    word: JString<'local>,
    preset: JString<'local>,
    mode: JString<'local>,
) -> jstring {
    let word = match jni_input(&mut env, &word) {
        Some(w) => w,
        None => return std::ptr::null_mut(),
    };
    let preset = match jni_input(&mut env, &preset) {
        Some(p) => p,
        None => return std::ptr::null_mut(),
    };
    let mode = jni_input(&mut env, &mode).unwrap_or_else(|| "inline".to_string());
    let rules = preset_rules(&preset);
    let spans = match_letters(&word, &rules);
    jni_output(
        &mut env,
        render_letters_html(&word, &spans, &rules, parse_mode(&mode)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_to_json_word() {
        let chunks = vec![TextChunk::Word(vec![
            "cho".into(),
            "co".into(),
            "lat".into(),
        ])];
        assert_eq!(
            chunks_to_json(&chunks),
            r#"[{"kind":"word","syllables":["cho","co","lat"]}]"#
        );
    }

    #[test]
    fn chunks_to_json_raw() {
        let chunks = vec![TextChunk::Raw(" ".into())];
        assert_eq!(chunks_to_json(&chunks), r#"[{"kind":"raw","text":" "}]"#);
    }

    /// Audit #3 — caractères de contrôle correctement échappés (RFC 8259).
    /// Régression du bug pré-serde_json où U+0001..U+001F sortaient bruts.
    #[test]
    fn chunks_to_json_escapes_control_chars() {
        let control = "a\u{0001}\u{0008}\u{000C}\u{001F}b";
        let chunks = vec![TextChunk::Raw(control.into())];
        let out = chunks_to_json(&chunks);
        // Aucun caractère de contrôle brut U+0000..U+001F ne doit subsister.
        assert!(
            !out.chars().any(|c| (c as u32) < 0x20),
            "control char brut dans {out:?}"
        );
        // Reparse → JSON syntaxiquement valide ET contenu préservé.
        let parsed: serde_json::Value = serde_json::from_str(&out).expect("invalid JSON produced");
        assert_eq!(parsed[0]["text"], control);
    }
}
