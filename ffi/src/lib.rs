use serde_json::{json, Value};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use syllabify_fr::letters::{match_letters, presets, render_letters_html, LetterRule, RenderMode};
use syllabify_fr::{phonemes, render_html, render_word_html, syllabify_text, syllables, TextChunk};

// --- JSON helpers via serde_json ---
//
// Échappement RFC 8259-conforme par construction : caractères de contrôle
// U+0000..U+001F sérialisés en `\uXXXX`, `\b`/`\f` en formes courtes, etc.
// Remplace les helpers maison antérieurs qui ne couvraient que `" \ \n \r \t`
// (cf. audit #3 / NOTES-v6.md « hand-rolled JSON »).

fn chunk_to_value(chunk: &TextChunk) -> Value {
    match chunk {
        TextChunk::Word(syls) => json!({ "kind": "word", "syllables": syls }),
        TextChunk::Raw(text) => json!({ "kind": "raw", "text": text }),
        _ => json!({ "kind": "unknown" }),
    }
}

fn chunks_to_json(chunks: &[TextChunk]) -> String {
    let values: Vec<Value> = chunks.iter().map(chunk_to_value).collect();
    // `to_string` ne peut échouer que sur `Map<NonString, _>` ou erreur d'IO
    // — aucun des deux cas n'est possible sur des `Value` construits ici.
    serde_json::to_string(&values).expect("serde_json::to_string never fails for owned Values")
}

fn phonemes_to_json(pairs: &[(String, String)]) -> String {
    let values: Vec<Value> = pairs
        .iter()
        .map(|(code, letters)| json!([code, letters]))
        .collect();
    serde_json::to_string(&values).expect("serde_json::to_string never fails for owned Values")
}

// --- Helpers for safe string conversion ---

unsafe fn c_str_to_rust<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

fn rust_string_to_c(s: String) -> *mut c_char {
    CString::new(s)
        .map(|cs| cs.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

// --- Public C API ---

/// Free a string previously returned by a `syllabify_*` function.
/// Passing NULL is a no-op.
///
/// # Safety
///
/// `ptr` must be NULL or a pointer previously returned by one of the
/// `syllabify_*` functions in this crate. Calling on any other pointer,
/// or calling twice on the same pointer (double-free), is undefined
/// behavior. After this call, `ptr` must not be dereferenced.
#[no_mangle]
pub unsafe extern "C" fn syllabify_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Syllabify a word, returning syllables joined by hyphens.
///
/// Example: `"chocolat"` → `"cho-co-lat"`
///
/// Returns NULL on NULL input or invalid UTF-8.
/// The caller must free the result with `syllabify_free()`.
///
/// # Safety
///
/// `word` must be NULL or point to a NUL-terminated C string valid for
/// the duration of the call. The returned pointer is owned by the caller
/// and must be freed exactly once via `syllabify_free`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_syllables(word: *const c_char) -> *mut c_char {
    let word = match c_str_to_rust(word) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(syllables(word).join("-"))
}

/// Syllabify full text, returning a JSON array of chunk objects.
///
/// Each chunk is either `{"kind":"word","syllables":["cho","co","lat"]}` or
/// `{"kind":"raw","text":" "}`.
///
/// Returns NULL on NULL input.
/// The caller must free the result with `syllabify_free()`.
///
/// # Safety
///
/// `text` must be NULL or point to a NUL-terminated C string valid for
/// the duration of the call. The returned pointer is owned by the caller
/// and must be freed exactly once via `syllabify_free`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_text_json(text: *const c_char) -> *mut c_char {
    let text = match c_str_to_rust(text) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(chunks_to_json(&syllabify_text(text)))
}

/// Get phonemes for a word as a JSON array of `[code, letters]` pairs.
///
/// Example: `"chat"` → `[["c","ch"],["V","a"],["c","t"]]`
///
/// Returns NULL on NULL input.
/// The caller must free the result with `syllabify_free()`.
///
/// # Safety
///
/// `word` must be NULL or point to a NUL-terminated C string valid for
/// the duration of the call. The returned pointer is owned by the caller
/// and must be freed exactly once via `syllabify_free`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_phonemes(word: *const c_char) -> *mut c_char {
    let word = match c_str_to_rust(word) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(phonemes_to_json(&phonemes(word)))
}

/// Render HTML for a single word with `<span class="syl-a/b">` syllable spans.
///
/// Returns NULL on NULL input.
/// The caller must free the result with `syllabify_free()`.
///
/// # Safety
///
/// `word` must be NULL or point to a NUL-terminated C string valid for
/// the duration of the call. The returned pointer is owned by the caller
/// and must be freed exactly once via `syllabify_free`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_render_word_html(word: *const c_char) -> *mut c_char {
    let word = match c_str_to_rust(word) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(render_word_html(word))
}

/// Render HTML for full text with syllable spans and liaison markers.
///
/// Returns NULL on NULL input.
/// The caller must free the result with `syllabify_free()`.
///
/// # Safety
///
/// `text` must be NULL or point to a NUL-terminated C string valid for
/// the duration of the call. The returned pointer is owned by the caller
/// and must be freed exactly once via `syllabify_free`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_render_html(text: *const c_char) -> *mut c_char {
    let text = match c_str_to_rust(text) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(render_html(text))
}

fn preset_rules(name: &str) -> Vec<LetterRule> {
    match name {
        "bdpq" => presets::bdpq(),
        "mnu" => presets::mnu(),
        "pir-pri" | "pir_pri" => presets::pir_pri(),
        _ => Vec::new(),
    }
}

fn parse_mode(mode: Option<&str>) -> RenderMode {
    match mode {
        Some("classes") => RenderMode::Classes,
        _ => RenderMode::Inline,
    }
}

/// Highlight confusable letters in `word` using a named preset.
///
/// `preset` accepts `"bdpq"`, `"mnu"`, or `"pir-pri"`.
/// `mode` accepts `"inline"` or `"classes"` (NULL = inline default).
/// On unknown preset the word is returned HTML-escaped without spans.
///
/// Returns NULL on NULL `word` or `preset`, or on invalid UTF-8.
/// The caller must free the result with `syllabify_free()`.
///
/// # Safety
///
/// `word` and `preset` must be NULL or point to NUL-terminated C strings
/// valid for the duration of the call. `mode` may additionally be NULL
/// (= inline default). The returned pointer is owned by the caller and
/// must be freed exactly once via `syllabify_free`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_highlight_letters(
    word: *const c_char,
    preset: *const c_char,
    mode: *const c_char,
) -> *mut c_char {
    let word = match c_str_to_rust(word) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let preset = match c_str_to_rust(preset) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let mode = c_str_to_rust(mode);
    let rules = preset_rules(preset);
    let spans = match_letters(word, &rules);
    rust_string_to_c(render_letters_html(word, &spans, &rules, parse_mode(mode)))
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe fn round_trip(
        f: unsafe extern "C" fn(*const c_char) -> *mut c_char,
        input: &str,
    ) -> String {
        let cstr = CString::new(input).unwrap();
        let ptr = f(cstr.as_ptr());
        assert!(!ptr.is_null(), "unexpected NULL for input {:?}", input);
        let result = CStr::from_ptr(ptr).to_str().unwrap().to_owned();
        syllabify_free(ptr);
        result
    }

    #[test]
    fn test_syllabify_syllables_chocolat() {
        let result = unsafe { round_trip(syllabify_syllables, "chocolat") };
        assert_eq!(result, "cho-co-lat");
    }

    #[test]
    fn test_syllabify_syllables_famille() {
        let result = unsafe { round_trip(syllabify_syllables, "famille") };
        assert_eq!(result, "fa-mi-lle");
    }

    #[test]
    fn test_syllabify_syllables_null_returns_null() {
        let ptr = unsafe { syllabify_syllables(std::ptr::null()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn test_syllabify_text_json_word_chunk() {
        let result = unsafe { round_trip(syllabify_text_json, "le chat dort") };
        assert!(result.starts_with('['), "expected JSON array: {result}");
        assert!(
            result.contains("\"kind\":\"word\""),
            "missing word chunk: {result}"
        );
        assert!(
            result.contains("\"syllables\""),
            "missing syllables: {result}"
        );
    }

    #[test]
    fn test_syllabify_text_json_raw_chunk() {
        let result = unsafe { round_trip(syllabify_text_json, "bonjour !") };
        assert!(
            result.contains("\"kind\":\"raw\""),
            "missing raw chunk: {result}"
        );
    }

    #[test]
    fn test_syllabify_phonemes_json() {
        let result = unsafe { round_trip(syllabify_phonemes, "chat") };
        assert!(result.starts_with('['), "expected JSON array: {result}");
        assert!(result.contains("[["), "expected nested arrays: {result}");
    }

    #[test]
    fn test_syllabify_render_word_html_has_spans() {
        let result = unsafe { round_trip(syllabify_render_word_html, "chocolat") };
        assert!(
            result.contains("syl-a") || result.contains("syl-b"),
            "missing syllable spans: {result}"
        );
    }

    #[test]
    fn test_syllabify_render_html_has_spans() {
        let result = unsafe { round_trip(syllabify_render_html, "les hôtels") };
        assert!(
            result.contains("syl-a") || result.contains("syl-b"),
            "missing syllable spans: {result}"
        );
    }

    #[test]
    fn test_syllabify_free_null_is_noop() {
        // Must not crash or panic
        unsafe { syllabify_free(std::ptr::null_mut()) };
    }

    #[test]
    fn test_highlight_letters_bdpq_inline() {
        let word = CString::new("dépit").unwrap();
        let preset = CString::new("bdpq").unwrap();
        let ptr = unsafe {
            syllabify_highlight_letters(word.as_ptr(), preset.as_ptr(), std::ptr::null())
        };
        assert!(!ptr.is_null());
        let result = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        unsafe { syllabify_free(ptr) };
        assert!(
            result.contains("color:#1e8e3e"),
            "expected d color: {result}"
        );
        assert!(
            result.contains("color:#d93025"),
            "expected p color: {result}"
        );
    }

    #[test]
    fn test_highlight_letters_classes_mode() {
        let word = CString::new("bd").unwrap();
        let preset = CString::new("bdpq").unwrap();
        let mode = CString::new("classes").unwrap();
        let ptr =
            unsafe { syllabify_highlight_letters(word.as_ptr(), preset.as_ptr(), mode.as_ptr()) };
        let result = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        unsafe { syllabify_free(ptr) };
        assert!(result.contains("class=\"lc-letter-0\""));
        assert!(result.contains("class=\"lc-letter-1\""));
    }

    #[test]
    fn test_highlight_letters_unknown_preset_returns_word() {
        let word = CString::new("bonjour").unwrap();
        let preset = CString::new("nope").unwrap();
        let ptr = unsafe {
            syllabify_highlight_letters(word.as_ptr(), preset.as_ptr(), std::ptr::null())
        };
        let result = unsafe { CStr::from_ptr(ptr).to_str().unwrap().to_owned() };
        unsafe { syllabify_free(ptr) };
        assert_eq!(result, "bonjour");
    }

    /// Audit #3 — la sortie JSON ne doit contenir aucun caractère de contrôle
    /// brut (U+0000..U+001F) après round-trip via `syllabify_text_json`.
    /// Pré-serde_json, ces caractères passaient tels quels → JSON invalide.
    #[test]
    fn test_syllabify_text_json_escapes_control_chars() {
        let probes = [
            "a\u{0001}b",
            "a\u{0007}b", // BEL
            "a\u{0008}b", // BS
            "a\u{000B}b", // VT
            "a\u{000C}b", // FF
            "a\u{001F}b", // US
        ];
        for input in &probes {
            let result = unsafe { round_trip(syllabify_text_json, input) };
            assert!(
                !result.chars().any(|c| (c as u32) < 0x20),
                "raw control char in output for input={input:?}: {result:?}"
            );
            // Round-trip parse — preuve définitive que c'est un JSON valide.
            let _parsed: serde_json::Value = serde_json::from_str(&result)
                .unwrap_or_else(|e| panic!("invalid JSON for input={input:?}: {e} :: {result:?}"));
        }
    }
}
