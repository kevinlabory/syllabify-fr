// SPDX-License-Identifier: GPL-3.0-or-later
use jni::errors::{ErrorPolicy, Result as JniResult};
use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jobjectArray, jstring};
use jni::{jni_str, Env, EnvUnowned};
use serde_json::{json, Value};
use syllabify_fr::letters::{match_letters, presets, render_letters_html, LetterRule, RenderMode};
use syllabify_fr::{phonemes, render_html, render_word_html, syllabify_text, syllables, TextChunk};

// --- JSON helpers via serde_json (RFC 8259-conforme par construction) ---

fn chunk_to_value(chunk: &TextChunk) -> Value {
    match chunk {
        TextChunk::Word(syls) => json!({ "kind": "word", "syllables": syls }),
        TextChunk::Raw(text) => json!({ "kind": "raw", "text": text }),
        _ => json!({ "kind": "unknown" }),
    }
}

fn chunks_to_json(chunks: &[TextChunk]) -> String {
    let values: Vec<Value> = chunks.iter().map(chunk_to_value).collect();
    serde_json::to_string(&values).expect("serde_json::to_string never fails for owned Values")
}

fn phonemes_to_json(pairs: &[(String, String)]) -> String {
    let values: Vec<Value> = pairs
        .iter()
        .map(|(code, letters)| json!([code, letters]))
        .collect();
    serde_json::to_string(&values).expect("serde_json::to_string never fails for owned Values")
}

// --- ErrorPolicy : préserve le contrat « null sur erreur » de jni 0.21 ---
//
// jni 0.22 oblige à choisir une politique pour mapper les erreurs/panics vers
// la valeur de retour. `ThrowRuntimeExAndDefault` jetterait une RuntimeException
// côté Java, ce qui changerait le contrat observable. `SilentDefault` retombe
// silencieusement sur `T::default()` (jstring/jobjectArray => null_mut), reproduisant
// le comportement pré-0.22 où une UTF-8 invalide ou allocation Java échouée
// résultait en `return std::ptr::null_mut()`.

struct SilentDefault;

impl<T: Default, E: std::error::Error> ErrorPolicy<T, E> for SilentDefault {
    type Captures<'unowned_env_local: 'native_method, 'native_method> = ();

    fn on_error<'unowned_env_local: 'native_method, 'native_method>(
        _env: &mut Env<'unowned_env_local>,
        _cap: &mut Self::Captures<'unowned_env_local, 'native_method>,
        _err: E,
    ) -> JniResult<T> {
        Ok(T::default())
    }

    fn on_panic<'unowned_env_local: 'native_method, 'native_method>(
        _env: &mut Env<'unowned_env_local>,
        _cap: &mut Self::Captures<'unowned_env_local, 'native_method>,
        _payload: Box<dyn std::any::Any + Send + 'static>,
    ) -> JniResult<T> {
        Ok(T::default())
    }
}

// --- Public JNI API ---
// Class: com.dyscolor.syllabify.SyllabifyFr
// Naming convention: Java_{package_underscored}_{ClassName}_{methodName}

/// `SyllabifyFr.syllables(word)` → `String[]`
///
/// Returns the syllables of a single French word.
/// Example: `"chocolat"` → `["cho", "co", "lat"]`
#[no_mangle]
#[allow(deprecated)] // `find_class` / `new_object_array` / `set_object_array_element`
                     // — migration vers `JObjectArray::<T>::new` / `set_element`
                     // à programmer (refactor type-generic non-trivial, hors-scope
                     // du bump jni 0.21 → 0.22).
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_syllables<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    word: JString<'local>,
) -> jobjectArray {
    env.with_env(|env| -> JniResult<jobjectArray> {
        let word: String = word.try_to_string(env)?;
        let syls = syllables(&word);
        let string_class = env.find_class(jni_str!("java/lang/String"))?;
        let arr: JObjectArray<'local> =
            env.new_object_array(syls.len() as i32, string_class, JString::default())?;
        for (i, syl) in syls.iter().enumerate() {
            let js = env.new_string(syl)?;
            env.set_object_array_element(&arr, i, js)?;
        }
        Ok(arr.into_raw())
    })
    .resolve::<SilentDefault>()
}

/// `SyllabifyFr.syllabifyText(text)` → JSON `String`
///
/// Returns a JSON array of chunks.
/// Example: `"le chat"` → `[{"kind":"word","syllables":["le"]},{"kind":"raw","text":" "},{"kind":"word","syllables":["chat"]}]`
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_syllabifyText<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    text: JString<'local>,
) -> jstring {
    env.with_env(|env| -> JniResult<jstring> {
        let text: String = text.try_to_string(env)?;
        let chunks = syllabify_text(&text);
        Ok(env.new_string(chunks_to_json(&chunks))?.into_raw())
    })
    .resolve::<SilentDefault>()
}

/// `SyllabifyFr.phonemes(word)` → JSON `String`
///
/// Returns a JSON array of `[code, letters]` pairs.
/// Example: `"chat"` → `[["s^","ch"],["a","a"],["#","t"]]`
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_phonemes<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    word: JString<'local>,
) -> jstring {
    env.with_env(|env| -> JniResult<jstring> {
        let word: String = word.try_to_string(env)?;
        let pairs = phonemes(&word);
        Ok(env.new_string(phonemes_to_json(&pairs))?.into_raw())
    })
    .resolve::<SilentDefault>()
}

/// `SyllabifyFr.renderWordHtml(word)` → HTML `String`
///
/// Returns HTML with `<span class="syl syl-a/b">` syllable spans for a single word.
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_renderWordHtml<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    word: JString<'local>,
) -> jstring {
    env.with_env(|env| -> JniResult<jstring> {
        let word: String = word.try_to_string(env)?;
        Ok(env.new_string(render_word_html(&word))?.into_raw())
    })
    .resolve::<SilentDefault>()
}

/// `SyllabifyFr.renderHtml(text)` → HTML `String`
///
/// Returns HTML with syllable spans and liaison markers for full text.
#[no_mangle]
pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_renderHtml<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    text: JString<'local>,
) -> jstring {
    env.with_env(|env| -> JniResult<jstring> {
        let text: String = text.try_to_string(env)?;
        Ok(env.new_string(render_html(&text))?.into_raw())
    })
    .resolve::<SilentDefault>()
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
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    word: JString<'local>,
    preset: JString<'local>,
    mode: JString<'local>,
) -> jstring {
    env.with_env(|env| -> JniResult<jstring> {
        let word: String = word.try_to_string(env)?;
        let preset: String = preset.try_to_string(env)?;
        // `mode` est optionnel côté Java (peut être null). On retombe sur "inline"
        // si la conversion échoue (typiquement parce que `mode` est null).
        let mode_str = mode
            .try_to_string(env)
            .unwrap_or_else(|_| "inline".to_string());
        let rules = preset_rules(&preset);
        let spans = match_letters(&word, &rules);
        let html = render_letters_html(&word, &spans, &rules, parse_mode(&mode_str));
        Ok(env.new_string(html)?.into_raw())
    })
    .resolve::<SilentDefault>()
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
