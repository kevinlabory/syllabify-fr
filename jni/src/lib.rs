// SPDX-License-Identifier: GPL-3.0-or-later
use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jobjectArray, jstring};
use jni::JNIEnv;
use syllabify_fr::letters::{match_letters, presets, render_letters_html, LetterRule, RenderMode};
use syllabify_fr::{phonemes, render_html, render_word_html, syllabify_text, syllables, TextChunk};

// --- JSON helpers (same approach as the FFI crate, no serde dependency) ---

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", json_escape(s))
}

fn json_chunk(chunk: &TextChunk) -> String {
    match chunk {
        TextChunk::Word(syls) => {
            let items: Vec<String> = syls.iter().map(|s| json_str(s)).collect();
            format!("{{\"kind\":\"word\",\"syllables\":[{}]}}", items.join(","))
        }
        TextChunk::Raw(text) => {
            format!("{{\"kind\":\"raw\",\"text\":{}}}", json_str(text))
        }
        _ => "{\"kind\":\"unknown\"}".to_string(),
    }
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
    let items: Vec<String> = chunks.iter().map(json_chunk).collect();
    jni_output(&mut env, format!("[{}]", items.join(",")))
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
    let items: Vec<String> = pairs
        .iter()
        .map(|(code, letters)| format!("[{},{}]", json_str(code), json_str(letters)))
        .collect();
    jni_output(&mut env, format!("[{}]", items.join(",")))
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
    fn json_escape_special_chars() {
        assert_eq!(json_str("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_str("a\\b"), "\"a\\\\b\"");
    }

    #[test]
    fn json_chunk_word() {
        let chunk = TextChunk::Word(vec!["cho".into(), "co".into(), "lat".into()]);
        assert_eq!(
            json_chunk(&chunk),
            r#"{"kind":"word","syllables":["cho","co","lat"]}"#
        );
    }

    #[test]
    fn json_chunk_raw() {
        let chunk = TextChunk::Raw(" ".into());
        assert_eq!(json_chunk(&chunk), r#"{"kind":"raw","text":" "}"#);
    }
}
